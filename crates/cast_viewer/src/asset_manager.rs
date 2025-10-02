use porter_app::{
    AssetPreview, AssetStatus, Color, Controller, SearchAsset, SearchTerm, Settings,
    palette::ASSET_TYPE_MODEL,
};
use porter_cast::{CastFile, CastId};
use porter_threads::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

use crate::cast_model;

#[derive(Debug)]
pub struct Asset {
    pub name: String,
    pub file_name: PathBuf,
    //pub cast: cast_model::CastNode,
    pub status: AssetStatus,
}

impl Asset {
    pub fn search(&self) -> SearchAsset {
        SearchAsset::new(self.name().to_string())
    }

    /// Returns the name of the asset
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the PorterAssetStatus of an Asset
    fn status(&self) -> &AssetStatus {
        &self.status
    }

    fn info(&self) -> String {
        "N/A".to_string()
    }

    /// Returns the color of the asset type
    fn color(&self) -> Color {
        ASSET_TYPE_MODEL
    }

    /// Returns the type name kf the asset
    fn type_name(&self) -> String {
        "Model".to_string()
    }
}

pub type LoadedAssets = Arc<RwLock<Vec<Asset>>>;

#[derive(Debug)]
pub struct AssetManager {
    search_assets: Arc<RwLock<Option<Vec<usize>>>>,
    loaded_assets: LoadedAssets,
}

impl AssetManager {
    pub fn new() -> Self {
        // Initialize your asset manager as needed
        AssetManager {
            search_assets: Arc::new(RwLock::new(None)),
            loaded_assets: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // TODO: Think of a better way of doing this, currently reads the entire file when initially loading
    //       which is not ideal for large files.
    pub fn ensure_has_model<R: Read>(mut reader: R) -> Result<(), String> {
        let cast = CastFile::read(&mut reader).map_err(|e| format!("Error loading file: {e}"))?;

        let found = cast
            .roots()
            .iter()
            .any(|root| root.children_of_type(CastId::Model).next().is_some());

        if found {
            Ok(())
        } else {
            Err("No model found".to_string())
        }
    }
}

impl porter_app::AssetManager for AssetManager {
    /// Whether or not the asset manager supports loading game files on disk.
    fn supports_files(&self) -> bool {
        true
    }

    fn supports_directories(&self) -> bool {
        true
    }

    /// Gets information about the specific asset, in the form of column data.
    fn assets_info(&self, index: usize) -> Vec<(String, Option<Color>)> {
        let search_lock = self.search_assets.read().unwrap();
        let loaded_assets_lock = self.loaded_assets.read().unwrap();

        // Change this when new porter_lib out
        let asset_index = if let Some(search) = search_lock.as_ref() {
            search.get(index).copied()
        } else {
            Some(index)
        };

        let Some(asset_index) = asset_index else {
            return vec![];
        };

        match loaded_assets_lock.get(asset_index) {
            Some(asset) => vec![
                (asset.name(), None),
                (asset.type_name(), Some(asset.color())),
                (asset.status().to_string(), Some(asset.status().color())),
                (asset.info(), None),
            ],
            None => vec![],
        }
    }

    /// The number of visible assets, whether they are search results, or just loaded.
    fn assets_visible(&self) -> usize {
        let search_lock = self.search_assets.read().unwrap();

        if let Some(indexes) = search_lock.as_ref() {
            indexes.len()
        } else {
            self.loaded_assets.read().unwrap().len()
        }
    }

    /// The total number of assets loaded.
    fn assets_total(&self) -> usize {
        self.loaded_assets.read().unwrap().len()
    }

    fn search(&self, search: Option<SearchTerm>) {
        let Some(search) = search else {
            *self.search_assets.write().unwrap() = None;
            return;
        };

        let loaded_assets = self.loaded_assets.read().unwrap();

        let results = loaded_assets
            .as_slice()
            .into_par_iter()
            .enumerate()
            .filter_map(|(index, asset)| {
                if search.matches(asset.search()) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();

        *self.search_assets.write().unwrap() = Some(results);
    }

    /// Loads one or more given file in async.
    fn load_files(&self, _settings: Settings, files: Vec<PathBuf>) -> Result<(), String> {
        for file_name in &files {
            if let Some(ext) = file_name.extension().and_then(|ext| ext.to_str()) {
                if ext == "cast" {
                    let asset = Asset {
                        name: file_name
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .unwrap_or_default()
                            .to_string(),
                        file_name: file_name.to_path_buf(),
                        status: AssetStatus::LOADED,
                    };

                    // Assign to shared state
                    let mut loaded = self.loaded_assets.write();
                    match loaded.as_mut() {
                        Ok(loaded) => {
                            loaded.push(asset);
                        }
                        Err(_) => {
                            return Err("Failed to acquire write lock on loaded assets".to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Recursively loads all files found under the provided directory.
    fn load_directory(&self, _settings: Settings, directory: PathBuf) -> Result<(), String> {
        if !directory.is_dir() {
            return Err("Provided path is not a directory".to_string());
        }

        let mut discovered = Vec::new();

        for entry in WalkDir::new(&directory).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("cast") {
                    let reader = File::open(path).map_err(|e| format!("Could not open: {e}"))?;

                    // Ensure the file has a model node
                    if Self::ensure_has_model(reader).is_err() {
                        continue;
                    }

                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string();

                    discovered.push(Asset {
                        name,
                        file_name: path.to_path_buf(),
                        status: AssetStatus::LOADED,
                    });
                }
            }
        }

        if discovered.is_empty() {
            return Ok(());
        }

        let mut loaded = self.loaded_assets.write();

        match loaded.as_mut() {
            Ok(loaded) => {
                loaded.extend(discovered);
                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock on loaded assets".to_string()),
        }
    }

    /// Exports a game's assets in async.
    fn export(&self, _settings: Settings, _assets: Vec<usize>, controller: Controller) {
        controller.progress_update(true, 100);
    }

    /// Loads a game's asset for previewing.
    fn preview(
        &self,
        _settings: Settings,
        asset: usize,
        _raw: bool,
        request_id: u64,
        controller: Controller,
    ) {
        let assets_guard = self.loaded_assets.read().unwrap();

        let (asset_name, asset_ref) = {
            let search = self.search_assets.read().unwrap();
            let asset_index = search
                .as_ref()
                .and_then(|s| s.get(asset).copied())
                .unwrap_or(asset);

            let selected_asset = &assets_guard[asset_index];

            let name = PathBuf::from(&selected_asset.name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&selected_asset.name)
                .to_string();

            (name, selected_asset)
        };

        let preview_asset = File::open(&asset_ref.file_name).ok().and_then(|mut f| {
            let mut buffer = Vec::new();
            if f.read_to_end(&mut buffer).is_err() {
                return None;
            }
            let mut cursor = Cursor::new(&buffer);
            let file = CastFile::read(&mut cursor).ok()?;
            let root = file.roots().first()?;
            let model_node = root.children_of_type(CastId::Model).next()?;
            // You must NOT return references into `file` or `model_node` here.
            let model = cast_model::process_model_node(model_node)?;
            let images = cast_model::load_model_images(&model, &asset_ref.file_name);
            Some(AssetPreview::Model(asset_name, model, images))
        });

        if let Some(preview) = preview_asset {
            controller.preview_update(request_id, preview);
        }
    }

    /// Cancels an active export.
    fn export_cancel(&self) {}

    fn load_game(&self, _settings: Settings) -> Result<(), String> {
        todo!()
    }
}
