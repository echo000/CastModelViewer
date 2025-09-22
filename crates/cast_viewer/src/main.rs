#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
mod asset_manager;
mod cast_model;
use porter_app::palette::*;

fn main() {
    porter_app::initialize(asset_manager::AssetManager::new())
        .version(env!("CARGO_PKG_VERSION"))
        .name("Cast Viewer")
        .description("Cast Model Viewer")
        .column("Name", 350, None, None)
        .column("Type", 100, None, None)
        .column("Status", 150, None, None)
        .column("Info", 250, Some(TEXT_COLOR_SECONDARY), None)
        .file_filter("Cast Models (*.cast)", vec!["cast"])
        .run();
}
