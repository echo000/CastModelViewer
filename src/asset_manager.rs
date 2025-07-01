use porter_threads::ParallelIterator;
use porter_ui::{
    Color, PorterAssetManager, PorterAssetStatus, PorterColorPalette, PorterPreviewAsset,
    PorterSearch, PorterSearchAsset, PorterSettings, PorterUI,
};
use rayon::prelude::*;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::cast_model;

#[derive(Debug)]
pub struct Asset {
    pub name: String,
    pub file_name: PathBuf,
    //pub cast: cast_model::CastNode,
    pub status: PorterAssetStatus,
}

impl Asset {
    pub fn search(&self) -> PorterSearchAsset {
        PorterSearchAsset::new(self.name())
    }

    /// Returns the name of the asset
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the PorterAssetStatus of an Asset
    fn status(&self) -> &PorterAssetStatus {
        &self.status
    }

    fn info(&self) -> String {
        "N/A".to_string()
    }

    /// Returns the color of the asset type
    fn color(&self) -> Color {
        PorterColorPalette::asset_type_model()
    }

    /// Returns the type name kf the asset
    fn type_name(&self) -> &'static str {
        "Model"
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
}

impl PorterAssetManager for AssetManager {
    /// Returns the asset info in the form of the columns to render.
    /// Returns the asset info in the form of the columns to render.
    fn asset_info(&self, index: usize, _columns: usize) -> Vec<(String, Option<Color>)> {
        let search = self.search_assets.read().unwrap();
        let loaded_assets = self.loaded_assets.read().unwrap();

        //Change this
        let asset_index = if let Some(search) = search.as_ref() {
            search.get(index).copied()
        } else {
            Some(index)
        };

        let Some(asset_index) = asset_index else {
            return vec![];
        };

        match loaded_assets.get(asset_index) {
            Some(asset) => vec![
                (asset.name(), None),
                (asset.type_name().to_string(), Some(asset.color())),
                (asset.status().to_string(), Some(asset.status().color())),
                (asset.info(), None),
            ],
            None => vec![],
        }
    }

    /// Returns the number of assets renderable, as in search for, or loaded.
    fn len(&self) -> usize {
        if let Some(indexes) = &*self.search_assets.read().unwrap() {
            indexes.len()
        } else {
            self.loaded_assets.read().unwrap().len()
        }
    }

    /// Returns the total number of assets loaded.
    fn loaded_len(&self) -> usize {
        self.loaded_assets.read().unwrap().len()
    }

    fn search_assets(&self, search: Option<PorterSearch>) {
        let Some(search) = search else {
            *self.search_assets.write().unwrap() = None;
            return;
        };

        let loaded_assets = self.loaded_assets.read().unwrap();

        let results = loaded_assets
            .par_iter()
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

    /// Whether or not load files is supported.
    fn supports_load_files(&self) -> bool {
        true
    }

    /// Whether or not load game is supported.
    fn supports_load_game(&self) -> bool {
        false
    }

    /// Loads one or more given file in async.
    fn on_load_files(&self, _settings: PorterSettings, files: Vec<PathBuf>) -> Result<(), String> {
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
                        status: PorterAssetStatus::loaded(),
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

    /// Exports a game's assets in async.
    fn on_export(&self, _settings: PorterSettings, _assets: Vec<usize>, _ui: PorterUI) {}

    /// Loads a game's asset for previewing.
    fn on_preview(&self, _settings: PorterSettings, asset: usize, request_id: u64, ui: PorterUI) {
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

        let preview = File::open(&asset_ref.file_name).ok().and_then(|mut f| {
            let mut buffer = Vec::new();
            if f.read_to_end(&mut buffer).is_ok() {
                let mut cursor = Cursor::new(&buffer);
                cast_model::load_cast_file(&mut cursor).and_then(|cast| {
                    cast_model::process_model_node(&cast).map(|model| {
                        let images = cast_model::load_model_images(&model, &asset_ref.file_name);
                        PorterPreviewAsset::Model(asset_name, model, images)
                    })
                })
            } else {
                None
            }
        });

        ui.preview(preview, request_id);
    }

    /// Cancels an active export.
    fn cancel_export(&self) {}

    fn on_load_game(&self, _settings: PorterSettings) -> Result<(), String> {
        todo!()
    }
}
