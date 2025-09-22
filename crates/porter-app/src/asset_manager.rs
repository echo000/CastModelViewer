use std::path::PathBuf;

use iced::Color;

use crate::ColumnStatus;
use crate::Controller;
use crate::Icon;
use crate::SearchTerm;
use crate::Settings;

/// Asset manager handles loading, exporting and management of game assets.
pub trait AssetManager: Send + Sync + 'static {
    /// Whether or not the asset manager supports loading game files on disk.
    fn supports_files(&self) -> bool {
        false
    }

    /// Whether or not the asset manager supports loading from the game directly.
    fn supports_games(&self) -> bool {
        false
    }

    /// Gets information about the specific asset, in the form of column data.
    fn assets_info(&self, index: usize) -> Vec<(String, Option<Color>)>;

    /// The number of visible assets, whether they are search results, or just loaded.
    fn assets_visible(&self) -> usize;

    /// The total number of assets loaded.
    fn assets_total(&self) -> usize;

    /// Whether or not there are visible assets.
    fn assets_empty(&self) -> bool {
        self.assets_visible() == 0
    }

    /// Search for assets, or reset the search term.
    fn search(&self, term: Option<SearchTerm>);

    /// Sort assets based on column status, returns the new column sort statuses.
    fn sort(&self, column: Option<usize>, statuses: Vec<ColumnStatus>) -> Vec<ColumnStatus> {
        let _ = column;
        let _ = statuses;

        Vec::new()
    }

    /// Loads one or more given files.
    fn load_files(&self, settings: Settings, files: Vec<PathBuf>) -> Result<(), String>;

    /// Loads a running game instance.
    fn load_game(&self, settings: Settings) -> Result<(), String>;

    /// Optional icon to display as an indicator on the main window.
    ///
    /// This icon is refreshed in two different ways:
    /// - Once when the application initially loads.
    /// - Whenever a call to `load_files` or `load_game` finishes.
    fn display_icon(&self) -> Option<Icon> {
        None
    }

    /// Request one or more assets be exported.
    fn export(&self, settings: Settings, assets: Vec<usize>, controller: Controller);

    /// Cancels an active export.
    fn export_cancel(&self);

    /// Request the given assets data for preview, optionally forcing a raw file preview.
    fn preview(
        &self,
        settings: Settings,
        asset: usize,
        raw: bool,
        request_id: u64,
        controller: Controller,
    );
}
