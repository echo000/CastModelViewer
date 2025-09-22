use std::path::PathBuf;

use iced::keyboard;
use iced::keyboard::Key;
use iced::keyboard::Modifiers;
use iced::keyboard::key::Named;

use iced::widget::Column;
use iced::widget::container;

use iced::window;
use iced::window::Mode;
use iced::window::Position;

use iced::Element;
use iced::Event;
use iced::Length;
use iced::Size;
use iced::Task;

use porter_utils::StringCaseExt;

use rfd::FileDialog;
use rfd::MessageButtons;
use rfd::MessageDialog;
use rfd::MessageLevel;

use crate::AppState;
use crate::Message;
use crate::components::About;
use crate::components::Content;
use crate::components::ContentMessage;
use crate::components::Controls;
use crate::components::ControlsMessage;
use crate::components::Header;
use crate::components::HeaderMessage;
use crate::components::SearchBar;
use crate::components::SearchBarMessage;
use crate::components::Settings;
use crate::components::SettingsMessage;
use crate::components::VirtualListMessage;

/// Main window handler.
pub struct MainWindow {
    pub id: window::Id,
    pub header: Header,
    search_bar: SearchBar,
    content: Content,
    controls: Controls,
    about: About,
    settings: Settings,
}

/// Messages produced by the main window.
#[derive(Debug, Clone)]
pub enum MainMessage {
    UI(Event),
    Show,
    Header(HeaderMessage),
    SearchBar(SearchBarMessage),
    Content(ContentMessage),
    Controls(ControlsMessage),
    Settings(SettingsMessage),
    LoadFile,
    PickExportFolder,
    Warning(String),
}

impl MainWindow {
    /// Creates a new main window.
    pub fn create() -> (Self, Task<window::Id>) {
        let (id, task) = window::open(window::Settings {
            size: Size::new(920.0, 582.0),
            position: Position::Centered,
            min_size: Some(Size::new(920.0, 582.0)),
            visible: false,
            ..Default::default()
        });

        (
            Self {
                id,
                header: Header::new(),
                search_bar: SearchBar::new(),
                content: Content::with_virtual_list(),
                controls: Controls::new(),
                about: About::new(),
                settings: Settings::new(),
            },
            task,
        )
    }

    /// Handles the title of the main window.
    pub fn title(&self, state: &AppState) -> String {
        format!("{} v{}", state.name.to_titlecase(), state.version)
    }

    /// Handles updates for the main window.
    pub fn update(&mut self, state: &mut AppState, message: MainMessage) -> Task<Message> {
        use MainMessage::*;

        match message {
            UI(event) => self.on_ui(state, event),
            Show => self.on_show(state),
            Header(message) => self.header.update(state, message),
            SearchBar(message) => self.search_bar.update(state, message),
            Content(message) => self.content.update(state, message),
            Controls(message) => self.controls.update(state, message),
            Settings(message) => self.settings.update(state, message),
            LoadFile => self.on_load_file(state),
            PickExportFolder => self.on_pick_export_folder(state),
            Warning(message) => self.on_warning(state, message),
        }
    }

    /// Handles rendering the main window.
    pub fn view<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        let mut columns: Column<_> = Column::with_capacity(4);

        columns = columns.push(self.header.view(state));

        if self.header.show_about {
            columns = columns.push(self.about.view(state));
        } else if self.header.show_settings {
            columns = columns.push(self.settings.view(state));
        } else {
            columns = columns
                .push(self.search_bar.view(state))
                .push(self.content.view(state))
                .push(self.controls.view(state));
        }

        container(columns)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Occurs when a ui event has fired.
    fn on_ui(&mut self, state: &mut AppState, event: Event) -> Task<Message> {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                self.on_key_pressed(state, key)
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifier_keys)) => {
                self.on_modifiers_changed(state, modifier_keys)
            }
            Event::Window(window::Event::Opened { .. }) => self.on_opened(),
            Event::Window(window::Event::Closed) => self.on_closed(),
            Event::Window(window::Event::FileDropped(path)) => self.on_file_dropped(state, path),
            _ => Task::none(),
        }
    }

    /// Occurs when a key has been pressed.
    fn on_key_pressed(&mut self, state: &mut AppState, key: Key) -> Task<Message> {
        match key.as_ref() {
            Key::Character("c") => {
                if state.modifier_keys.command() {
                    self.on_copy_text(state)
                } else {
                    Task::none()
                }
            }
            Key::Character("e") => {
                if state.assets_selected.is_empty() {
                    Task::none()
                } else {
                    Task::done(Message::ExportSelected)
                }
            }
            Key::Character("p") => Task::done(Message::PreviewToggle),
            Key::Character("f") => {
                if state.modifier_keys.command() {
                    Task::done(Message::from(SearchBarMessage::Find))
                } else {
                    Task::none()
                }
            }
            Key::Named(Named::ArrowUp) => Task::done(Message::from(VirtualListMessage::MoveUp)),
            Key::Named(Named::ArrowDown) => Task::done(Message::from(VirtualListMessage::MoveDown)),
            Key::Named(Named::PageUp) => Task::done(Message::from(VirtualListMessage::PageUp)),
            Key::Named(Named::PageDown) => Task::done(Message::from(VirtualListMessage::PageDown)),
            _ => Task::none(),
        }
    }

    /// Occurs when the modifier keys change.
    fn on_modifiers_changed(
        &mut self,
        state: &mut AppState,
        modifier_keys: Modifiers,
    ) -> Task<Message> {
        state.modifier_keys = modifier_keys;

        Task::none()
    }

    /// Occurs when the window has opened.
    fn on_opened(&mut self) -> Task<Message> {
        Task::done(Message::WindowOpened(self.id))
    }

    /// Occurs when the window is closed.
    fn on_closed(&mut self) -> Task<Message> {
        iced::exit()
    }

    /// Occurs when a file has been dropped onto the window.
    fn on_file_dropped(&mut self, state: &mut AppState, path: PathBuf) -> Task<Message> {
        if state.is_busy() {
            return Task::none();
        }

        if state.files_dropped.is_empty() {
            state.controller.load_files_dropped();
        }

        state.files_dropped.push(path);

        Task::none()
    }

    /// Shows the main window.
    fn on_show(&mut self, state: &mut AppState) -> Task<Message> {
        let icon = state.asset_manager.display_icon();

        #[cfg(feature = "start-preview")]
        {
            Task::batch([
                window::set_mode(self.id, Mode::Windowed),
                Task::done(Message::PreviewToggle),
                Task::done(Message::from(HeaderMessage::UpdateIcon(icon))),
            ])
        }

        #[cfg(not(feature = "start-preview"))]
        {
            Task::batch([
                window::set_mode(self.id, Mode::Windowed),
                Task::done(Message::from(HeaderMessage::UpdateIcon(icon))),
            ])
        }
    }

    /// Attempts to select files to load.
    fn on_load_file(&mut self, state: &mut AppState) -> Task<Message> {
        if state.is_busy() {
            return Task::none();
        }

        let mut file_dialog = FileDialog::new();

        for (name, extensions) in &state.file_filters {
            file_dialog = file_dialog.add_filter(*name, extensions);
        }

        let controller = state.controller.clone();

        let title = format!("{} | Select game files to load", state.name.to_titlecase());

        window::run_with_handle(self.id, move |handle| {
            let file_dialog = file_dialog.set_parent(&handle).set_title(title);

            let dialog = move || {
                if cfg!(feature = "multi-file") {
                    if let Some(files) = file_dialog.pick_files() {
                        controller.load_files(files);
                    }
                } else if let Some(file) = file_dialog.pick_file() {
                    controller.load_files(vec![file]);
                }
            };

            #[cfg(target_os = "windows")]
            std::thread::spawn(dialog);

            #[cfg(not(target_os = "windows"))]
            dialog();

            Message::Noop
        })
    }

    /// Occurs when the user wants to pick a new export folder.
    fn on_pick_export_folder(&mut self, state: &mut AppState) -> Task<Message> {
        let mut settings = state.settings.clone();

        let title = format!("{} | Select an export directory", state.name.to_titlecase());

        window::run_with_handle(self.id, move |handle| {
            let path = FileDialog::new()
                .set_directory(settings.output_directory())
                .set_parent(&handle)
                .set_title(title)
                .pick_folder();

            if let Some(path) = path {
                settings.set_output_directory(path);

                Message::from(SettingsMessage::Save(settings))
            } else {
                Message::Noop
            }
        })
    }

    /// Show a warning to the user.
    fn on_warning(&mut self, state: &mut AppState, message: String) -> Task<Message> {
        let title = state.name.to_titlecase();

        window::run_with_handle(self.id, move |handle| {
            let dialog = MessageDialog::new()
                .set_title(title)
                .set_description(message)
                .set_level(MessageLevel::Warning)
                .set_buttons(MessageButtons::Ok)
                .set_parent(&handle);

            let dialog = move || {
                dialog.show();
            };

            #[cfg(target_os = "windows")]
            std::thread::spawn(dialog);

            #[cfg(not(target_os = "windows"))]
            dialog();

            Message::Noop
        })
    }

    /// Copy the selected assets to the clipboard.
    fn on_copy_text(&mut self, state: &mut AppState) -> Task<Message> {
        if state.is_busy() || state.assets_selected.is_empty() {
            return Task::none();
        }

        let buffer = state
            .assets_selected
            .iter()
            .copied()
            .map(|index| state.asset_manager.assets_info(index))
            .map(|mut info| info.remove(0).0)
            .collect::<Vec<_>>()
            .join("\n");

        iced::clipboard::write(buffer)
    }
}
