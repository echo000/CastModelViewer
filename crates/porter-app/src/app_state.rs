use std::collections::BTreeSet;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

use iced::advanced::graphics;

use iced::keyboard::Modifiers;

use iced::Color;
use iced::Error;
use iced::Font;
use iced::Pixels;

use porter_threads::initialize_thread_pool;

use porter_utils::StringCaseExt;

use rfd::MessageButtons;
use rfd::MessageDialog;

use crate::App;
use crate::AssetColumn;
use crate::AssetManager;
use crate::Controller;
use crate::Executor;
use crate::Settings;
use crate::Sort;
use crate::palette;
use crate::panic_hook;
use crate::strings;

/// Shared application state information.
pub struct AppState {
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) description: &'static str,
    pub(crate) file_filters: Vec<(&'static str, Vec<&'static str>)>,
    pub(crate) last_load: Option<Vec<PathBuf>>,
    pub(crate) files_dropped: Vec<PathBuf>,
    pub(crate) controller: Controller,
    pub(crate) loading: bool,
    pub(crate) exporting: bool,
    pub(crate) progress: u32,
    pub(crate) export_canceled: bool,
    pub(crate) reload_required: bool,
    pub(crate) modifier_keys: Modifiers,
    pub(crate) settings: Settings,
    pub(crate) item_range: Range<usize>,
    pub(crate) asset_manager: Arc<dyn AssetManager + 'static>,
    pub(crate) asset_columns: Vec<AssetColumn>,
    pub(crate) asset_preview_id: Option<u64>,
    pub(crate) assets_selected: BTreeSet<usize>,
}

impl AppState {
    /// Constructs a new application state with the given asset manager.
    pub(crate) fn new<T: AssetManager + 'static>(asset_manager: T) -> Self {
        AppState {
            name: "<unset>",
            version: "<unset>",
            description: "<unset>",
            file_filters: Vec::new(),
            last_load: None,
            files_dropped: Vec::new(),
            controller: Controller::new(),
            loading: false,
            exporting: false,
            progress: 0,
            export_canceled: false,
            reload_required: false,
            modifier_keys: Modifiers::empty(),
            settings: Settings::default(),
            item_range: 0..0,
            asset_manager: Arc::new(asset_manager),
            asset_columns: Vec::new(),
            asset_preview_id: None,
            assets_selected: BTreeSet::new(),
        }
    }

    /// Whether or not the app is currently loading, exporting, or doing other processing work.
    pub(crate) fn is_busy(&self) -> bool {
        self.loading || self.exporting
    }

    /// Resets the virtual list item range.
    pub(crate) fn reset_item_range(&mut self) {
        self.item_range = 0..50.min(self.asset_manager.assets_visible())
    }

    /// The name of the application. Used for the main window header and settings/crash files.
    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// The version of the program.
    pub const fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }

    /// The description of the program.
    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    /// Adds a column to the asset virtual list.
    pub fn column(
        mut self,
        header: &'static str,
        width: usize,
        color: Option<Color>,
        sort: Option<Sort>,
    ) -> Self {
        self.asset_columns
            .push(AssetColumn::new(header, width, color, sort));
        self
    }

    /// Adds the default columns to the asset virtual list.
    pub fn default_columns(self) -> Self {
        self.column("Name", 350, None, None)
            .column("Type", 115, None, None)
            .column("Status", 150, None, None)
            .column("Info", 250, Some(palette::TEXT_COLOR_SECONDARY), None)
    }

    /// Adds a file filter to the load files dialog.
    pub fn file_filter(mut self, title: &'static str, extensions: Vec<&'static str>) -> Self {
        self.file_filters.push((title, extensions));
        self
    }

    /// Runs the app until the main window is closed.
    pub fn run(mut self) {
        panic_hook::install(self.name, self.version);

        // Load user settings if possible.
        self.settings = Settings::load(self.name);

        // Initialize global rayon thread pool.
        initialize_thread_pool();

        let settings = iced::Settings {
            id: None,
            fonts: Vec::new(),
            default_font: Font::DEFAULT,
            default_text_size: Pixels(16.0),
            antialiasing: true,
            #[cfg(target_os = "windows")]
            vsync: false,
            #[cfg(not(target_os = "windows"))]
            vsync: true,
        };

        let name = self.name;

        let result = iced::daemon(App::title, App::update, App::view)
            .settings(settings)
            .executor::<Executor>()
            .theme(App::theme)
            .subscription(App::subscription)
            .font(include_bytes!("./fonts/porter.ttf"))
            .run_with(move || App::new(self));

        if let Err(error) = result {
            handle_error(name, error);
        }
    }
}

/// Handles errors that occur during initialization.
fn handle_error(name: &'static str, original: Error) {
    let Error::GraphicsCreationFailed(original) = original else {
        std::panic::panic_any(original);
    };

    let error = if let graphics::Error::List(errors) = &original {
        errors.first().unwrap()
    } else {
        &original
    };

    if matches!(
        error,
        graphics::Error::VersionNotSupported
            | graphics::Error::BackendError(_)
            | graphics::Error::NoAvailablePixelFormat
    ) {
        MessageDialog::new()
            .set_title(format!("{} | Graphics Error", name.to_titlecase()))
            .set_description(strings::GRAPHICS_DRIVER_ERROR)
            .set_buttons(MessageButtons::Ok)
            .show();
    } else if matches!(error, graphics::Error::GraphicsAdapterNotFound { .. }) {
        MessageDialog::new()
            .set_title(format!("{} | Graphics Error", name.to_titlecase()))
            .set_description(strings::GRAPHICS_DEVICE_ERROR)
            .set_buttons(MessageButtons::Ok)
            .show();
    }

    std::panic::panic_any(original);
}
