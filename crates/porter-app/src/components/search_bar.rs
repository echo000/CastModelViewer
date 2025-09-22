use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text_input;

use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::Task;

use crate::AppState;
use crate::Message;
use crate::SearchTerm;
use crate::palette;
use crate::widgets;

use super::VirtualListMessage;

/// The maximum number of assets before search isn't realtime.
const SEARCH_REALTIME_MAX: usize = 500_000;

/// Search bar component handler.
pub struct SearchBar {
    search: String,
    search_id: text_input::Id,
}

/// Messages produced by the search bar component.
#[derive(Debug, Clone)]
pub enum SearchBarMessage {
    Input(String),
    Clear,
    Submit,
    Find,
}

impl SearchBar {
    /// Creates a new search bar component.
    pub fn new() -> Self {
        Self {
            search: String::new(),
            search_id: text_input::Id::unique(),
        }
    }

    /// Handles updates for the search bar component.
    pub fn update(&mut self, state: &mut AppState, message: SearchBarMessage) -> Task<Message> {
        use SearchBarMessage::*;

        match message {
            Input(input) => self.on_search_input(state, input),
            Clear => self.on_search_clear(state),
            Submit => self.on_search_submit(state),
            Find => self.on_search_find(state),
        }
    }

    /// Handles rendering the search bar component.
    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        let mut row = row([widgets::text_input("Search for assets...", &self.search)
            .id(self.search_id.clone())
            .on_input_maybe(if state.is_busy() {
                None
            } else {
                Some(|input| Message::from(SearchBarMessage::Input(input)))
            })
            .on_submit_maybe(if self.search.is_empty() || state.is_busy() {
                None
            } else {
                Some(Message::from(SearchBarMessage::Submit))
            })
            .width(Length::Fixed(350.0))
            .into()]);

        if state.asset_manager.assets_total() > SEARCH_REALTIME_MAX {
            row = row.push(widgets::button("Search").on_press_maybe(
                if self.search.is_empty() || state.is_busy() {
                    None
                } else {
                    Some(Message::from(SearchBarMessage::Submit))
                },
            ));
        }

        row = row.push(widgets::button("Clear").on_press_maybe(
            if self.search.is_empty() || state.is_busy() {
                None
            } else {
                Some(Message::from(SearchBarMessage::Clear))
            },
        ));

        container(
            row.push(
                text(if state.loading {
                    String::from("Loading...")
                } else if self.search.is_empty() {
                    format!("{} assets loaded", state.asset_manager.assets_visible())
                } else {
                    format!(
                        "Showing {} assets out of {} loaded",
                        state.asset_manager.assets_visible(),
                        state.asset_manager.assets_total()
                    )
                })
                .width(Length::Fill)
                .align_x(Alignment::End)
                .color(palette::TEXT_COLOR_SECONDARY),
            )
            .align_y(Alignment::Center)
            .spacing(4.0),
        )
        .width(Length::Fill)
        .height(Length::Shrink)
        .padding(8.0)
        .into()
    }

    /// Modifies the search input.
    fn on_search_input(&mut self, state: &mut AppState, input: String) -> Task<Message> {
        self.search = input;

        if state.asset_manager.assets_total() > SEARCH_REALTIME_MAX && !self.search.is_empty() {
            Task::none()
        } else {
            self.on_search_submit(state)
        }
    }

    /// Clears any search results.
    fn on_search_clear(&mut self, state: &mut AppState) -> Task<Message> {
        self.search = String::new();

        state.assets_selected.clear();

        state.asset_manager.search(None);
        state.reset_item_range();

        Task::done(Message::from(VirtualListMessage::ScrollReset))
    }

    /// Submits the search term to filter assets.
    fn on_search_submit(&mut self, state: &mut AppState) -> Task<Message> {
        if self.search.is_empty() {
            return self.on_search_clear(state);
        }

        state.assets_selected.clear();

        state
            .asset_manager
            .search(Some(SearchTerm::compile(self.search.clone())));
        state.reset_item_range();

        Task::done(Message::from(VirtualListMessage::ScrollReset))
    }

    /// Focuses and selects all search text.
    fn on_search_find(&mut self, _: &mut AppState) -> Task<Message> {
        Task::batch([
            text_input::focus(self.search_id.clone()),
            text_input::select_all(self.search_id.clone()),
        ])
    }
}
