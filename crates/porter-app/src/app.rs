use std::path::PathBuf;

use iced::theme::Palette;

use iced::window;

use iced::Element;
use iced::Event;
use iced::Subscription;
use iced::Task;
use iced::Theme;

use crate::AppState;
use crate::AssetPreview;
use crate::ColumnStatus;
use crate::Controller;
use crate::MainMessage;
use crate::MainWindow;
use crate::Message;
use crate::PreviewWindow;
use crate::PreviewWindowMessage;
use crate::SplashMessage;
use crate::SplashWindow;
use crate::components::ContentMessage;
use crate::components::HeaderMessage;
use crate::components::PreviewMessage;
use crate::components::SearchBarMessage;
use crate::palette;

/// Entry point for the iced application.
pub struct App {
    state: AppState,
    main_window: MainWindow,
    splash_window: SplashWindow,
    preview_window: Option<PreviewWindow>,
}

impl App {
    /// Constructs a new app entry point.
    pub fn new(state: AppState) -> (Self, Task<Message>) {
        let (main_window, main_window_task) = MainWindow::create();
        let (splash_window, splash_window_task) = SplashWindow::create();

        let task = Task::batch([main_window_task, splash_window_task]).discard();

        let ui = Self {
            state,
            main_window,
            splash_window,
            preview_window: None,
        };

        (ui, task)
    }

    /// Provides the title for the given window.
    pub fn title(&self, id: window::Id) -> String {
        if id == self.main_window.id {
            self.main_window.title(&self.state)
        } else if id == self.splash_window.id {
            self.splash_window.title(&self.state)
        } else if self
            .preview_window
            .as_ref()
            .map(|window| window.id)
            .is_some_and(|x| x == id)
        {
            self.preview_window
                .as_ref()
                .map(|window| window.title(&self.state))
                .unwrap_or_else(|| String::from("<unset>"))
        } else {
            String::from("<unset>")
        }
    }

    /// Handles updating the app state.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        use Message::*;

        match message {
            Noop => self.on_noop(),
            UI(event, id) => self.on_ui(event, id),
            WindowOpened(id) => self.on_window_opened(id),
            Controller(controller) => self.on_controller(controller),
            Splash(message) => self.splash_window.update(message),
            Main(message) => self.main_window.update(&mut self.state, message),
            PreviewProxy(message) => self.on_preview_proxy(message),
            PreviewWindow(message) => self
                .preview_window
                .as_mut()
                .map(|window| window.update(&mut self.state, message))
                .unwrap_or(Task::none()),
            PreviewWindowCreate => self.on_preview_window_create(),
            PreviewWindowClosed => self.on_preview_window_closed(),
            PreviewToggle => self.on_preview_toggle(),
            PreviewRequest => self.on_preview_request(),
            LoadUpdate(result) => self.on_load_update(result),
            ProgressUpdate(finished, progress) => self.on_progress_update(finished, progress),
            PreviewUpdate(request_id, asset) => self.on_preview_update(request_id, asset),
            ExportSelected => self.on_export_selected(),
            ExportAll => self.on_export_all(),
            ExportCancel => self.on_export_cancel(),
            LoadFiles(files) => self.on_load_files(files),
            LoadFilesDropped => self.on_load_files_dropped(),
            LoadGame => self.on_load_game(),
            Sort(index) => self.on_sort(index),
            CheckReload => self.on_check_reload(),
        }
    }

    /// Custom theme defaults.
    pub fn theme(&self, _: window::Id) -> Theme {
        Theme::custom(
            String::from("Porter"),
            Palette {
                background: palette::BACKGROUND_COLOR_DEFAULT,
                text: palette::TEXT_COLOR_DEFAULT,
                primary: palette::PRIMARY_COLOR,
                ..Palette::LIGHT
            },
        )
    }

    /// Handles global and controller events.
    pub fn subscription(&self) -> Subscription<Message> {
        use iced::event;
        use iced::event::Status;
        use iced::stream;

        use iced::futures::SinkExt;
        use iced::futures::StreamExt;
        use iced::futures::channel::mpsc;

        /// Filters out events that aren't necessary for the global listener.
        #[inline(always)]
        fn filter_event(event: Event, id: window::Id) -> Option<Message> {
            // Whenever we need to listen to a new global event, add it here.
            // This prevents thrashing the main message queue with unnecessary events.
            if matches!(event, Event::Keyboard(_))
                || matches!(event, Event::Window(window::Event::Closed))
                || matches!(event, Event::Window(window::Event::Opened { .. }))
                || matches!(event, Event::Window(window::Event::FileDropped(_)))
            {
                return Some(Message::UI(event, id));
            }

            None
        }

        let events = event::listen_with(|event, status, id| match status {
            Status::Ignored => filter_event(event, id),
            Status::Captured => None,
        });

        let controller = Subscription::run(|| {
            stream::channel(100, |mut output: mpsc::Sender<Message>| async move {
                let (tx, mut rx) = mpsc::unbounded::<Message>();

                output
                    .send(Message::Controller(Controller::with_channel(tx)))
                    .await
                    .expect("Failed to initialize controller!");

                loop {
                    while let Some(message) = rx.next().await {
                        let result = output.send(message).await;

                        debug_assert!(result.is_ok());
                    }
                }
            })
        });

        Subscription::batch([events, controller])
    }

    /// Handles rendering a given window.
    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        if id == self.main_window.id {
            self.main_window.view(&self.state)
        } else if id == self.splash_window.id {
            self.splash_window.view(&self.state)
        } else if self
            .preview_window
            .as_ref()
            .map(|window| window.id)
            .is_some_and(|x| x == id)
        {
            self.preview_window
                .as_ref()
                .map(|window| window.view(&self.state))
                .unwrap_or_else(|| iced::widget::text("<unset>").into())
        } else {
            iced::widget::text("<unset>").into()
        }
    }

    /// Occurs when nothing should happen.
    fn on_noop(&mut self) -> Task<Message> {
        Task::none()
    }

    /// Occurs when a ui event has triggered for a given window.
    fn on_ui(&mut self, event: Event, id: window::Id) -> Task<Message> {
        if id == self.main_window.id {
            self.main_window
                .update(&mut self.state, MainMessage::UI(event))
        } else if id == self.splash_window.id {
            self.splash_window.update(SplashMessage::UI(event))
        } else if self
            .preview_window
            .as_ref()
            .map(|window| window.id)
            .is_some_and(|x| x == id)
        {
            self.preview_window
                .as_mut()
                .map(|window| window.update(&mut self.state, PreviewWindowMessage::UI(event)))
                .unwrap_or_else(Task::none)
        } else {
            Task::none()
        }
    }

    /// Occurs when a window opens.
    fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Foundation::*;
            use windows_sys::Win32::UI::WindowsAndMessaging::*;

            use raw_window_handle::RawWindowHandle;

            use crate::icon_windows;

            iced::window::run_with_handle(id, |handle| {
                let icon = icon_windows::windows_icon();

                if let RawWindowHandle::Win32(handle) = handle.window.as_raw() {
                    unsafe {
                        PostMessageW(
                            handle.hwnd.get() as _,
                            WM_SETICON,
                            ICON_BIG as WPARAM,
                            icon as LPARAM,
                        )
                    };
                    unsafe {
                        PostMessageW(
                            handle.hwnd.get() as _,
                            WM_SETICON,
                            ICON_SMALL as WPARAM,
                            icon as LPARAM,
                        )
                    };
                }

                Message::Noop
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = id;

            Task::none()
        }
    }

    /// Occurs when the global controller is initialized.
    fn on_controller(&mut self, controller: Controller) -> Task<Message> {
        self.state.controller = controller;

        Task::none()
    }

    /// Occurs when progress has been made by the asset manager.
    fn on_progress_update(&mut self, finished: bool, progress: u32) -> Task<Message> {
        if finished {
            self.state.loading = false;
            self.state.exporting = false;
            self.state.export_canceled = false;
            self.state.progress = 0;

            return self.on_check_reload();
        } else {
            self.state.progress = progress.clamp(0, 100);
        }

        Task::none()
    }

    /// Occurs when a preview request has been completed by the asset manager.
    fn on_preview_update(&mut self, request_id: u64, asset: AssetPreview) -> Task<Message> {
        if self.state.asset_preview_id.is_none()
            || self.state.asset_preview_id.is_some_and(|x| x != request_id)
        {
            return Task::none();
        }

        self.state.asset_preview_id = None;

        self.on_preview_proxy(PreviewMessage::Update(asset))
    }

    /// Occurs when a load request has been completed by the asset manager.
    fn on_load_update(&mut self, result: Result<(), String>) -> Task<Message> {
        self.state.loading = false;
        self.state.progress = 0;
        self.state.reset_item_range();

        let icon = self.state.asset_manager.display_icon();

        if let Err(e) = result {
            self.state.last_load = None;

            Task::batch([
                Task::done(Message::from(MainMessage::Warning(e))),
                Task::done(Message::from(HeaderMessage::UpdateIcon(icon))),
            ])
        } else {
            Task::batch([
                Task::done(Message::from(SearchBarMessage::Submit)),
                Task::done(Message::from(HeaderMessage::UpdateIcon(icon))),
                Task::done(Message::Sort(None)),
                self.on_check_reload(),
            ])
        }
    }

    /// Occurs when a preview message needs to be proxied to the specific window it exists on.
    fn on_preview_proxy(&mut self, message: PreviewMessage) -> Task<Message> {
        if let Some(preview_window) = &mut self.preview_window {
            use ContentMessage::*;
            use PreviewWindowMessage::*;

            preview_window.update(&mut self.state, Content(Preview(message)))
        } else {
            use ContentMessage::*;
            use MainMessage::*;

            self.main_window
                .update(&mut self.state, Content(Preview(message)))
        }
    }

    /// Occurs when the user wants to expand the preview to a new window.
    fn on_preview_window_create(&mut self) -> Task<Message> {
        let (preview_window, preview_window_task) = PreviewWindow::create();

        self.preview_window = Some(preview_window);

        Task::batch([
            preview_window_task.discard(),
            Task::done(Message::PreviewRequest),
        ])
    }

    /// Occurs when the user closes the preview window.
    fn on_preview_window_closed(&mut self) -> Task<Message> {
        self.preview_window = None;

        Task::none()
    }

    /// Occurs when the user wants to toggle the preview window.
    fn on_preview_toggle(&mut self) -> Task<Message> {
        if self.preview_window.is_some() {
            return Task::none();
        }

        if self.state.settings.preview_window() {
            self.on_preview_window_create()
        } else {
            Task::done(Message::from(ContentMessage::PreviewToggle))
        }
    }

    /// Occurs when we need to request a new preview asset from the asset manager.
    fn on_preview_request(&mut self) -> Task<Message> {
        let Some(index) = self.state.assets_selected.first().cloned() else {
            return Task::none();
        };

        if self.state.asset_manager.assets_empty() {
            return Task::none();
        }

        let manager = self.state.asset_manager.clone();
        let controller = self.state.controller.clone();
        let settings = self.state.settings.clone();
        let request_id = self
            .state
            .asset_preview_id
            .map(|id| id + 1)
            .unwrap_or_default();

        self.state.asset_preview_id = Some(request_id);

        let raw = self.state.modifier_keys.alt();

        porter_threads::spawn(move || {
            manager.preview(settings, index, raw, request_id, controller);
        });

        Task::none()
    }

    /// Occurs when the user requests to export selected assets.
    fn on_export_selected(&mut self) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let manager = self.state.asset_manager.clone();
        let controller = self.state.controller.clone();
        let settings = self.state.settings.clone();
        let assets: Vec<usize> = self.state.assets_selected.iter().copied().collect();

        self.state.exporting = true;
        self.state.export_canceled = false;
        self.state.progress = 0;

        porter_threads::spawn(move || {
            manager.export(settings, assets, controller);
        });

        Task::none()
    }

    /// Occurs when the user requests to export all assets.
    fn on_export_all(&mut self) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let manager = self.state.asset_manager.clone();
        let controller = self.state.controller.clone();
        let settings = self.state.settings.clone();
        let assets: Vec<usize> = (0..self.state.asset_manager.assets_visible()).collect();

        self.state.exporting = true;
        self.state.export_canceled = false;
        self.state.progress = 0;

        porter_threads::spawn(move || {
            manager.export(settings, assets, controller);
        });

        Task::none()
    }

    /// Occurs when the user requests to cancel an export.
    fn on_export_cancel(&mut self) -> Task<Message> {
        if self.state.export_canceled {
            return Task::none();
        }

        self.state.export_canceled = true;
        self.state.asset_manager.export_cancel();

        Task::none()
    }

    /// Occurs when the user requests to load some files.
    fn on_load_files(&mut self, files: Vec<PathBuf>) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let manager = self.state.asset_manager.clone();
        let controller = self.state.controller.clone();
        let settings = self.state.settings.clone();

        self.state.loading = true;
        self.state.progress = 0;
        self.state.last_load = Some(files.clone());
        self.state.assets_selected.clear();

        porter_threads::spawn(move || {
            controller.load_update(manager.load_files(settings, files));
        });

        Task::none()
    }

    /// Occurs when the last file has been received from the drop queue.
    fn on_load_files_dropped(&mut self) -> Task<Message> {
        let mut files = std::mem::take(&mut self.state.files_dropped);

        // We need to only take the files which match a filter in the application.
        // Then load those files only, to prevent issues with the asset manager.
        let mut target_extension: Option<String> = None;

        'outer: for filter in &self.state.file_filters {
            for file in &files {
                if let Some(extension) = file.extension() {
                    let extension = extension.to_string_lossy().into_owned();

                    if filter.1.iter().any(|&x| x == extension) {
                        target_extension = Some(extension);
                        break 'outer;
                    }
                }
            }
        }

        let Some(target_extension) = target_extension else {
            return Task::none();
        };

        let files_to_load: Vec<_> = files
            .drain(..)
            .filter(|file| {
                let Some(extension) = file.extension() else {
                    return false;
                };

                target_extension == extension.to_string_lossy()
            })
            .take(if cfg!(feature = "multi-file") {
                usize::MAX
            } else {
                1
            })
            .collect();

        if !files_to_load.is_empty() {
            return self.on_load_files(files_to_load);
        }

        Task::none()
    }

    /// Occurs when the user requests to load a game.
    fn on_load_game(&mut self) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let manager = self.state.asset_manager.clone();
        let controller = self.state.controller.clone();
        let settings = self.state.settings.clone();

        self.state.loading = true;
        self.state.progress = 0;
        self.state.last_load = Some(Vec::new());
        self.state.assets_selected.clear();

        porter_threads::spawn(move || {
            controller.load_update(manager.load_game(settings));
        });

        Task::none()
    }

    /// Occurs when assets should be sorted, or a column has changed.
    fn on_sort(&mut self, index: Option<usize>) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let statuses: Vec<_> = self
            .state
            .asset_columns
            .iter()
            .enumerate()
            .map(|(index, column)| ColumnStatus::new(index, column.sort.unwrap_or_default()))
            .collect();

        for status in self.state.asset_manager.sort(index, statuses) {
            if let Some(column) = self.state.asset_columns.get_mut(status.index)
                && column.sort.is_some()
            {
                column.sort = Some(status.sort);
            }
        }

        Task::none()
    }

    /// Occurs when we need to check if a reload is required.
    fn on_check_reload(&mut self) -> Task<Message> {
        if !self.state.reload_required
            || self.state.is_busy()
            || self.main_window.header.show_settings
        {
            return Task::none();
        }

        self.state.reload_required = false;

        if let Some(last_load) = self.state.last_load.take() {
            if last_load.is_empty() {
                return self.on_load_game();
            } else {
                return self.on_load_files(last_load);
            }
        }

        Task::none()
    }
}
