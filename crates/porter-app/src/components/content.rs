use iced::widget::container;
use iced::widget::row;

use iced::Element;
use iced::Length;
use iced::Task;

use crate::AppState;
use crate::Message;

use super::Preview;
use super::PreviewMessage;
use super::VirtualList;
use super::VirtualListMessage;

/// Content component handler.
pub struct Content {
    preview: Option<Preview>,
    virtual_list: Option<VirtualList>,
}

/// Messages produced by content component.
#[derive(Debug, Clone)]
pub enum ContentMessage {
    Preview(PreviewMessage),
    PreviewToggle,
    PreviewWindow,
    VirtualList(VirtualListMessage),
}

impl Content {
    /// Creates a new content component with a virtual list component.
    pub fn with_virtual_list() -> Self {
        Self {
            preview: None,
            virtual_list: Some(VirtualList::new()),
        }
    }

    /// Creates a new content component with a preview component.
    pub fn with_preview() -> Self {
        Self {
            preview: Some(Preview::new()),
            virtual_list: None,
        }
    }

    /// Handles updates for the content component.
    pub fn update(&mut self, state: &mut AppState, message: ContentMessage) -> Task<Message> {
        use ContentMessage::*;

        match message {
            Preview(message) => self
                .preview
                .as_mut()
                .map(|x| x.update(state, message))
                .unwrap_or(Task::none()),
            PreviewToggle => self.on_preview_toggle(state),
            PreviewWindow => self.on_preview_window(state),
            VirtualList(message) => self
                .virtual_list
                .as_mut()
                .map(|x| x.update(state, message))
                .unwrap_or(Task::none()),
        }
    }

    /// Handles rendering for the content component.
    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        match (&self.preview, &self.virtual_list) {
            (Some(preview), Some(virtual_list)) => {
                container(row([virtual_list.view(state), preview.view(state, true)]).spacing(4.0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding([0.0, 8.0])
                    .into()
            }
            (None, Some(virtual_list)) => container(row([virtual_list.view(state)]).spacing(4.0))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([0.0, 8.0])
                .into(),
            (Some(preview), None) => container(row([preview.view(state, false)]))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(8.0)
                .into(),
            (None, None) => container(row([]))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(8.0)
                .into(),
        }
    }

    /// Occurs when the user toggles the preview.
    fn on_preview_toggle(&mut self, _: &mut AppState) -> Task<Message> {
        if self.preview.is_some() {
            self.preview = None;
            Task::none()
        } else {
            self.preview = Some(Preview::new());
            Task::done(Message::PreviewRequest)
        }
    }

    /// Occurs when the user wants to expand preview to a new window.
    fn on_preview_window(&mut self, _: &mut AppState) -> Task<Message> {
        self.preview = None;

        Task::done(Message::PreviewWindowCreate)
    }
}
