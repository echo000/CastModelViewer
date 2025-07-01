#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
mod asset_manager;
mod cast_model;
use porter_ui::PorterColorPalette;

fn main() {
    porter_ui::create_main(asset_manager::AssetManager::new())
        .version("0.0.1")
        .name("Cast Viewer")
        .description("Cast Model Viewer")
        .column("Name", 350, None)
        .column("Type", 100, None)
        .column("Status", 150, None)
        .column("Info", 250, Some(PorterColorPalette::asset_info()))
        .file_filter("Cast Models (*.cast)", vec!["cast"])
        .images_enabled(false)
        .raw_files_enabled(false)
        .animations_enabled(false)
        .materials_enabled(false)
        .raw_files_forcable(false)
        .multi_file(true)
        .normal_map_converter(false)
        .run();
}
