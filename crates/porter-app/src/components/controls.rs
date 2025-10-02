use iced::widget::container;
use iced::widget::row;
use iced::widget::stack;
use iced::widget::text;

use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::Task;

use crate::AppState;
use crate::MainMessage;
use crate::Message;
use crate::widgets;

/// Controls component handler.
pub struct Controls;

#[derive(Debug, Clone)]
pub enum ControlsMessage {
    LoadGame,
    LoadFile,
    LoadFolder,
    ExportSelected,
    ExportAll,
    ExportCancel,
}

impl Controls {
    /// Creates a new controls component.
    pub fn new() -> Self {
        Self
    }

    /// Handles updates for the controls component.
    pub fn update(&mut self, _state: &mut AppState, message: ControlsMessage) -> Task<Message> {
        use ControlsMessage::*;

        match message {
            LoadGame => Task::done(Message::LoadGame),
            LoadFile => Task::done(Message::from(MainMessage::LoadFile)),
            LoadFolder => Task::done(Message::from(MainMessage::LoadFolder)),
            ExportSelected => Task::done(Message::ExportSelected),
            ExportAll => Task::done(Message::ExportAll),
            ExportCancel => Task::done(Message::ExportCancel),
        }
    }

    /// Handles rendering for the controls component.
    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        let mut row = row(Vec::with_capacity(8))
            .spacing(8.0)
            .align_y(Alignment::Center);

        if state.asset_manager.supports_games() {
            row = row.push(
                widgets::button("Load Game")
                    .padding([6.0, 10.0])
                    .on_press_maybe(if state.is_busy() {
                        None
                    } else {
                        Some(Message::from(ControlsMessage::LoadGame))
                    }),
            );
        }

        if state.asset_manager.supports_files() {
            row = row.push(
                widgets::button("Load File")
                    .padding([6.0, 10.0])
                    .on_press_maybe(if state.is_busy() {
                        None
                    } else {
                        Some(Message::from(ControlsMessage::LoadFile))
                    }),
            );
        }

        if state.asset_manager.supports_directories() {
            row = row.push(
                widgets::button("Load Folder")
                    .padding([6.0, 10.0])
                    .on_press_maybe(if state.is_busy() {
                        None
                    } else {
                        Some(Message::from(ControlsMessage::LoadFolder))
                    }),
            );
        }

        #[cfg(feature = "raw-files-forcible")]
        {
            row = row
                .push(
                    widgets::button("Export Selected")
                        .padding([6.0, 10.0])
                        .on_press_maybe(if state.assets_selected.is_empty() || state.is_busy() {
                            None
                        } else {
                            Some(Message::from(ControlsMessage::ExportSelected))
                        }),
                )
                .push(
                    widgets::button("Export All")
                        .padding([6.0, 10.0])
                        .on_press_maybe(if state.asset_manager.assets_empty() || state.is_busy() {
                            None
                        } else {
                            Some(Message::from(ControlsMessage::ExportAll))
                        }),
                );
        }

        if state.exporting {
            row = row
                .push(
                    widgets::button(if state.export_canceled {
                        "Canceling..."
                    } else {
                        "Cancel"
                    })
                    .padding([6.0, 10.0])
                    .on_press_maybe(if state.export_canceled {
                        None
                    } else {
                        Some(Message::from(ControlsMessage::ExportCancel))
                    }),
                )
                .push(
                    container(stack([
                        widgets::progress_bar(0.0..=100.0, state.progress as f32)
                            .length(200.0)
                            .girth(32.0)
                            .into(),
                        text(format!("{}%", state.progress))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .into(),
                    ]))
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .align_x(Alignment::End)
                    .align_y(Alignment::Center),
                );
        }

        container(row)
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding([10.0, 8.0])
            .into()
    }
}
