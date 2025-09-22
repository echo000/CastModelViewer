pub(crate) mod components;
pub(crate) mod fonts;
pub(crate) mod panic_hook;
pub(crate) mod strings;
pub(crate) mod system;
pub(crate) mod widgets;

#[cfg(target_os = "windows")]
pub(crate) mod icon_windows;

mod app;
mod app_state;
mod asset_column;
mod asset_manager;
mod asset_preview;
mod asset_status;
mod column_status;
mod controller;
mod executor;
mod icon;
mod message;
mod search;
mod settings;
mod sort;
mod windows;

pub(crate) use app::*;
pub(crate) use asset_column::*;
pub(crate) use executor::*;
pub(crate) use message::*;
pub(crate) use windows::*;

/// Shared application palette and colors for ui elements.
pub mod palette;

pub use app_state::*;
pub use asset_manager::*;
pub use asset_preview::*;
pub use asset_status::*;
pub use column_status::*;
pub use controller::*;
pub use icon::*;
pub use search::*;
pub use settings::*;
pub use sort::*;

/// Re-exported for use in public interfaces.
pub use iced::Color;

/// Initializes a new app state with the given asset manager.
pub fn initialize<A: AssetManager + 'static>(asset_manager: A) -> AppState {
    AppState::new(asset_manager)
}
