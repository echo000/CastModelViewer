use iced::Element;
use iced::window;
use iced::window::Position;
use iced::window::Settings;

use iced::Event;
use iced::Size;
use iced::Task;

use porter_utils::StringCaseExt;

use crate::AppState;
use crate::Message;
use crate::components::Content;
use crate::components::ContentMessage;

/// Preview window handler.
pub struct PreviewWindow {
    pub id: window::Id,
    content: Content,
}

/// Messages produced by the preview window.
#[derive(Debug, Clone)]
pub enum PreviewWindowMessage {
    UI(Event),
    Content(ContentMessage),
}

impl PreviewWindow {
    /// Creates a new preview window.
    pub fn create() -> (Self, Task<window::Id>) {
        let (id, task) = window::open(Settings {
            size: Size::new(920.0, 582.0),
            position: Position::Centered,
            min_size: Some(Size::new(920.0, 582.0)),
            ..Default::default()
        });

        (
            Self {
                id,
                content: Content::with_preview(),
            },
            task,
        )
    }

    /// Handles the title of the preview window.
    pub fn title(&self, state: &AppState) -> String {
        format!("{} | Asset Preview", state.name.to_titlecase())
    }

    pub fn update(&mut self, state: &mut AppState, message: PreviewWindowMessage) -> Task<Message> {
        use PreviewWindowMessage::*;

        match message {
            UI(event) => self.on_ui(event),
            Content(message) => self.content.update(state, message),
        }
    }

    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        self.content.view(state)
    }

    /// Occurs when a ui event has fired.
    fn on_ui(&mut self, event: Event) -> Task<Message> {
        match event {
            Event::Window(window::Event::Opened { .. }) => self.on_opened(),
            Event::Window(window::Event::Closed) => self.on_closed(),
            _ => Task::none(),
        }
    }

    /// Occurs when the window has opened.
    fn on_opened(&mut self) -> Task<Message> {
        Task::done(Message::WindowOpened(self.id))
    }

    /// Occurs when the user closes the preview window.
    fn on_closed(&mut self) -> Task<Message> {
        Task::done(Message::PreviewWindowClosed)
    }
}
