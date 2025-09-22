use iced::widget::column;
use iced::widget::container;
use iced::widget::text;
use iced::widget::vertical_space;

use iced::Alignment;
use iced::Element;
use iced::Length;

use crate::AppState;
use crate::Message;
use crate::palette;
use crate::strings;

/// About component handler.
pub struct About;

impl About {
    /// Creates a new about component.
    pub fn new() -> Self {
        Self
    }

    /// Handles rendering for the about component.
    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        container(
            column([
                text(strings::PORTER_THANKS)
                    .size(20.0)
                    .color(palette::TEXT_COLOR_WARN)
                    .into(),
                vertical_space().height(20.0).into(),
                text(strings::PORTER_BUG_REPORT)
                    .size(18.0)
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                vertical_space().height(20.0).into(),
                text(strings::PORTER_DISCLAIMER)
                    .color(palette::TEXT_COLOR_MUTED)
                    .into(),
                text(format!(
                    "\"{}\" {}.",
                    state.name.to_uppercase(),
                    strings::PORTER_COPYRIGHT
                ))
                .color(palette::TEXT_COLOR_MUTED)
                .into(),
            ])
            .spacing(8.0)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
    }
}
