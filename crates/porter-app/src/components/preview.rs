use iced::border::Radius;
use iced::border::rounded;

use iced::widget::Column;
use iced::widget::Container;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::stack;
use iced::widget::text;
use iced::widget::text_editor;
use iced::widget::vertical_space;

use iced::Alignment;
use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Task;
use iced::Theme;

use porter_preview::PreviewError;

use crate::AppState;
use crate::AssetPreview;
use crate::Message;
use crate::fonts;
use crate::palette;
use crate::widgets;

use super::ContentMessage;

/// Size of the header in pixels.
const HEADER_HEIGHT: f32 = 30.0;
/// Text to display when not supported.
const TEXT_UNSUPPORTED: &str = "<This asset can't be represented as text>";

/// A list of preview controls to render over the previewer.
const PREVIEW_CONTROLS: &[(&str, &str)] = &[
    ("Toggle Bones:", "[B]"),
    ("Toggle Wireframe:", "[W]"),
    ("Toggle Shaded:", "[M]"),
    ("Toggle Grid:", "[G]"),
    ("Reset View:", "[R]"),
    ("Cycle Image:", "[N]"),
];

/// Preview component handler.
pub struct Preview {
    tab: PreviewTab,
    raw_text: text_editor::Content,
    raw_binary: Option<Vec<u8>>,
    raw_name: String,
    error: bool,
    unsupported: bool,
    viewport_state: widgets::ViewportState,
    scroll_id: scrollable::Id,
}

/// Currently active preview tab.
#[derive(Debug)]
pub enum PreviewTab {
    Viewport,
    Text,
    Binary,
}

/// Messages produced by the preview component.
#[derive(Debug, Clone)]
pub enum PreviewMessage {
    Viewport,
    ViewportAction(widgets::ViewportAction),
    Text,
    TextAction(text_editor::Action),
    Binary,
    Update(AssetPreview),
    Request,
}

impl Preview {
    /// Construct a new preview component.
    pub fn new() -> Self {
        Self {
            tab: PreviewTab::Viewport,
            raw_text: text_editor::Content::new(),
            raw_binary: None,
            raw_name: String::new(),
            error: false,
            unsupported: false,
            viewport_state: widgets::ViewportState::new(),
            scroll_id: scrollable::Id::unique(),
        }
    }

    /// Handles updates for the preview component.
    pub fn update(&mut self, state: &mut AppState, message: PreviewMessage) -> Task<Message> {
        use PreviewMessage::*;

        match message {
            Viewport => self.on_viewport(state),
            ViewportAction(action) => self.on_viewport_action(state, action),
            Text => self.on_text(state),
            TextAction(action) => self.on_text_action(state, action),
            Binary => self.on_binary(state),
            Update(asset) => self.on_preview_update(state, asset),
            Request => self.on_preview_request(state),
        }
    }

    /// Handles rendering for the preview component.
    pub fn view(&self, state: &AppState, embedded: bool) -> Element<'_, Message> {
        let header = container(
            row([
                text("Asset Preview")
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_y(Alignment::Center)
                    .into(),
                widgets::icon_button(
                    text("\u{E801}")
                        .size(12.0)
                        .font(fonts::ICON_FONT)
                        .height(Length::Fill)
                        .align_y(Alignment::Center)
                        .shaping(text::Shaping::Advanced),
                )
                .on_press(Message::from(ContentMessage::PreviewWindow))
                .width(Length::Shrink)
                .height(Length::Fill)
                .into(),
                widgets::icon_button(
                    text("\u{E802}")
                        .size(12.0)
                        .font(fonts::ICON_FONT)
                        .height(Length::Fill)
                        .align_y(Alignment::Center)
                        .shaping(text::Shaping::Advanced),
                )
                .on_press(Message::from(ContentMessage::PreviewToggle))
                .width(Length::Shrink)
                .height(Length::Fill)
                .into(),
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([0.0, 8.0])
            .spacing(8.0)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(HEADER_HEIGHT))
        .style(preview_header_style);

        let content = match &self.tab {
            PreviewTab::Viewport => {
                let viewport = Element::from(widgets::viewport(
                    &self.viewport_state,
                    // Keep the viewport state in sync with the external state.
                    |action| Message::from(PreviewMessage::ViewportAction(action)),
                ));

                let mut columns: Column<_> = Column::with_capacity(8)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(2.0);

                if self.error || self.unsupported {
                    columns = columns.push(
                        row([
                            text("Name")
                                .size(16.0)
                                .width(75.0)
                                .font(fonts::MONOSPACE_BOLD_FONT)
                                .color(palette::TEXT_COLOR_INFO)
                                .into(),
                            text(":")
                                .size(16.0)
                                .color(palette::TEXT_COLOR_INFO)
                                .font(fonts::MONOSPACE_BOLD_FONT)
                                .into(),
                            text(if self.error {
                                "<failed to load>"
                            } else {
                                "<not supported for preview>"
                            })
                            .size(16.0)
                            .color(palette::TEXT_COLOR_DEFAULT)
                            .font(fonts::MONOSPACE_BOLD_FONT)
                            .into(),
                        ])
                        .width(Length::Shrink)
                        .padding(2.0)
                        .spacing(8.0),
                    );
                } else {
                    let renderer = self.viewport_state.renderer();

                    for (stat_header, stat_value) in renderer.statistics() {
                        columns = columns.push(
                            row([
                                text(stat_header)
                                    .size(16.0)
                                    .width(75.0)
                                    .color(palette::TEXT_COLOR_INFO)
                                    .font(fonts::MONOSPACE_BOLD_FONT)
                                    .into(),
                                text(":")
                                    .size(16.0)
                                    .color(palette::TEXT_COLOR_INFO)
                                    .font(fonts::MONOSPACE_BOLD_FONT)
                                    .into(),
                                text(stat_value)
                                    .size(16.0)
                                    .color(palette::TEXT_COLOR_DEFAULT)
                                    .font(fonts::MONOSPACE_BOLD_FONT)
                                    .into(),
                            ])
                            .width(Length::Shrink)
                            .padding(2.0)
                            .spacing(8.0),
                        );
                    }
                }

                let columns = container(
                    container(columns)
                        .width(Length::Shrink)
                        .padding(4.0)
                        .style(preview_overlay_style),
                )
                .width(Length::Fill)
                .height(Length::FillPortion(1))
                .padding(4.0);

                let mut controls: Column<_> = Column::with_capacity(PREVIEW_CONTROLS.len())
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(2.0);

                for (control_name, control) in PREVIEW_CONTROLS {
                    controls = controls.push(
                        row([
                            text(*control_name)
                                .size(16.0)
                                .color(palette::TEXT_COLOR_INFO)
                                .font(fonts::MONOSPACE_BOLD_FONT)
                                .into(),
                            text(*control)
                                .size(16.0)
                                .color(palette::TEXT_COLOR_DEFAULT)
                                .font(fonts::MONOSPACE_BOLD_FONT)
                                .into(),
                        ])
                        .width(Length::Shrink)
                        .padding(2.0)
                        .spacing(8.0),
                    );
                }

                let controls = container(
                    container(controls)
                        .width(Length::Shrink)
                        .padding(4.0)
                        .style(preview_overlay_style),
                )
                .align_y(Alignment::End)
                .width(Length::Fill)
                .height(Length::FillPortion(1))
                .padding(4.0);

                let controls = if state.settings.preview_overlay() {
                    controls.into()
                } else {
                    vertical_space().into()
                };

                let overlay = if state.asset_preview_id.is_some() {
                    let loading = container(widgets::spinner())
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center);

                    column([columns.into(), loading.into(), controls])
                        .width(Length::Fill)
                        .height(Length::Fill)
                } else {
                    column([columns.into(), controls])
                        .width(Length::Fill)
                        .height(Length::Fill)
                };

                Element::from(stack([viewport, overlay.into()]))
            }
            PreviewTab::Text => Element::from(
                widgets::scrollable(
                    text_editor(&self.raw_text)
                        .height(Length::Shrink)
                        .size(14.0)
                        .padding(4.0)
                        .font(fonts::MONOSPACE_FONT)
                        .on_action(|action| Message::from(PreviewMessage::TextAction(action)))
                        .style(text_editor_style),
                )
                .id(self.scroll_id.clone())
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new()
                        .width(16.0)
                        .scroller_width(16.0)
                        .spacing(0.0),
                ))
                .width(Length::Fill)
                .height(Length::Fill),
            ),
            PreviewTab::Binary => Element::from(
                widgets::scrollable(widgets::binary(self.raw_binary.as_deref().unwrap_or(&[])))
                    .id(self.scroll_id.clone())
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new()
                            .width(16.0)
                            .scroller_width(16.0)
                            .spacing(0.0),
                    ))
                    .width(Length::Fill)
                    .height(Length::Fill),
            ),
        };

        let footer: Option<Container<_>> =
            if matches!(self.tab, PreviewTab::Binary | PreviewTab::Text) {
                Some(
                    container(
                        container(
                            row([
                                container(
                                    text(format!(
                                        "Name: {}",
                                        if self.error {
                                            "<failed to load>"
                                        } else {
                                            &self.raw_name
                                        }
                                    ))
                                    .width(Length::Shrink)
                                    .height(Length::Shrink)
                                    .color(if self.error {
                                        palette::TEXT_COLOR_WARN
                                    } else {
                                        palette::TEXT_COLOR_DEFAULT
                                    })
                                    .wrapping(text::Wrapping::None),
                                )
                                .clip(true)
                                .width(Length::Fill)
                                .height(Length::Shrink)
                                .into(),
                                text(if matches!(self.tab, PreviewTab::Text) {
                                    format!("Lines: {:?}", self.raw_text.line_count())
                                } else {
                                    format!(
                                        "Size: 0x{:02X}",
                                        self.raw_binary
                                            .as_ref()
                                            .map(|x| x.len())
                                            .unwrap_or_default()
                                    )
                                })
                                .width(Length::Shrink)
                                .height(Length::Shrink)
                                .into(),
                            ])
                            .spacing(8.0),
                        )
                        .padding([2.0, 4.0])
                        .style(preview_footer_style),
                    )
                    .align_y(Alignment::Center)
                    .style(preview_content_style),
                )
            } else {
                None
            };

        let content = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .padding(1.0)
            .style(preview_content_style);

        let tabs = row([
            widgets::tab(
                row([
                    text("\u{F1B2}")
                        .size(16.0)
                        .font(fonts::ICON_FONT)
                        .height(Length::Shrink)
                        .align_y(Alignment::Center)
                        .shaping(text::Shaping::Advanced)
                        .into(),
                    text("Viewport")
                        .height(Length::Shrink)
                        .align_y(Alignment::Center)
                        .into(),
                ])
                .height(Length::Shrink)
                .spacing(8.0),
                matches!(self.tab, PreviewTab::Viewport),
            )
            .width(Length::Shrink)
            .height(Length::Shrink)
            .on_press_maybe(if self.raw_binary.is_none() {
                Some(Message::from(PreviewMessage::Viewport))
            } else {
                None
            })
            .into(),
            widgets::tab(
                row([
                    text("\u{F0F6}")
                        .size(16.0)
                        .font(fonts::ICON_FONT)
                        .height(Length::Shrink)
                        .align_y(Alignment::Center)
                        .shaping(text::Shaping::Advanced)
                        .into(),
                    text("Text")
                        .height(Length::Shrink)
                        .align_y(Alignment::Center)
                        .into(),
                ])
                .height(Length::Shrink)
                .spacing(8.0),
                matches!(self.tab, PreviewTab::Text),
            )
            .width(Length::Shrink)
            .height(Length::Shrink)
            .on_press_maybe(if self.raw_binary.is_some() {
                Some(Message::from(PreviewMessage::Text))
            } else {
                None
            })
            .into(),
            widgets::tab(
                row([
                    text("\u{F1C9}")
                        .size(16.0)
                        .font(fonts::ICON_FONT)
                        .height(Length::Shrink)
                        .align_y(Alignment::Center)
                        .shaping(text::Shaping::Advanced)
                        .into(),
                    text("Binary")
                        .height(Length::Shrink)
                        .align_y(Alignment::Center)
                        .into(),
                ])
                .height(Length::Shrink)
                .spacing(8.0),
                matches!(self.tab, PreviewTab::Binary),
            )
            .width(Length::Shrink)
            .height(Length::Shrink)
            .on_press_maybe(if self.raw_binary.is_some() {
                Some(Message::from(PreviewMessage::Binary))
            } else {
                None
            })
            .into(),
        ])
        .width(Length::Fill)
        .height(Length::Shrink)
        .spacing(4.0);

        let view = if embedded {
            column(
                [Element::from(header), Element::from(content)]
                    .into_iter()
                    .chain(footer.map(Into::into))
                    .chain([Element::from(tabs)]),
            )
            .spacing(1.0)
        } else {
            column(
                [Element::from(content)]
                    .into_iter()
                    .chain(footer.map(Into::into))
                    .chain([Element::from(tabs)]),
            )
            .spacing(1.0)
        };

        container(view)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Occurs when the viewport tab is clicked.
    fn on_viewport(&mut self, _state: &mut AppState) -> Task<Message> {
        if matches!(self.tab, PreviewTab::Viewport) {
            return Task::none();
        }

        self.tab = PreviewTab::Viewport;

        Task::none()
    }

    /// Occurs when the viewport triggers an action.
    fn on_viewport_action(
        &mut self,
        state: &mut AppState,
        action: widgets::ViewportAction,
    ) -> Task<Message> {
        self.viewport_state.perform(
            action,
            state.settings.far_clip() as f32,
            state.settings.preview_controls(),
        );

        Task::none()
    }

    /// Occurs when the text tab is clicked.
    fn on_text(&mut self, _state: &mut AppState) -> Task<Message> {
        if matches!(self.tab, PreviewTab::Text) {
            return Task::none();
        }

        self.tab = PreviewTab::Text;

        Task::none()
    }

    /// Occurs when a text editor action comes in.
    fn on_text_action(
        &mut self,
        _state: &mut AppState,
        action: text_editor::Action,
    ) -> Task<Message> {
        if action.is_edit() {
            return Task::none();
        }

        self.raw_text.perform(action);

        Task::none()
    }

    /// Occurs when the binary tab is clicked.
    fn on_binary(&mut self, _state: &mut AppState) -> Task<Message> {
        if matches!(self.tab, PreviewTab::Binary) {
            return Task::none();
        }

        self.tab = PreviewTab::Binary;

        Task::none()
    }

    /// Occurs when the asset manager has a new asset to preview.
    fn on_preview_update(&mut self, _: &mut AppState, asset: AssetPreview) -> Task<Message> {
        match asset {
            AssetPreview::NotSupported => {
                self.raw_text = text_editor::Content::new();
                self.raw_binary = None;
                self.raw_name = String::new();

                self.error = false;
                self.unsupported = true;
                self.viewport_state.renderer_mut().clear_preview();

                self.tab = PreviewTab::Viewport;
            }
            AssetPreview::PreviewError => {
                self.raw_text = text_editor::Content::new();
                self.raw_binary = None;
                self.raw_name = String::new();

                self.error = true;
                self.unsupported = false;
                self.viewport_state.renderer_mut().clear_preview();

                self.tab = PreviewTab::Viewport;
            }
            AssetPreview::RawFile(name, raw_file) => {
                let text = std::str::from_utf8(&raw_file);

                if let Ok(text) = text {
                    if text
                        .chars()
                        // Ignore control keys that aren't WHITESPACE|UNIT SEPARATOR.
                        .any(|ch| ch.is_control() && !ch.is_whitespace() && ch != '\u{1f}')
                    {
                        self.raw_text = text_editor::Content::with_text(TEXT_UNSUPPORTED);
                        self.tab = PreviewTab::Binary;
                    } else {
                        self.raw_text = text_editor::Content::with_text(text);
                        self.tab = PreviewTab::Text;
                    }
                } else {
                    self.raw_text = text_editor::Content::with_text(TEXT_UNSUPPORTED);
                    self.tab = PreviewTab::Binary;
                }

                self.raw_binary = Some(raw_file);
                self.raw_name = name;

                self.error = false;
                self.unsupported = false;
                self.viewport_state.renderer_mut().clear_preview();

                return scrollable::scroll_to(
                    self.scroll_id.clone(),
                    scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                );
            }
            AssetPreview::Image(name, image) => {
                self.raw_text = text_editor::Content::new();
                self.raw_binary = None;
                self.raw_name = String::new();

                if let Err(e) = self
                    .viewport_state
                    .renderer_mut()
                    .set_preview_image(name, image)
                {
                    if matches!(e, PreviewError::Unsupported) {
                        self.unsupported = true;
                        self.error = false;
                    } else {
                        self.unsupported = false;
                        self.error = true;
                    }
                } else {
                    self.unsupported = false;
                    self.error = false;
                }

                self.tab = PreviewTab::Viewport;
            }
            AssetPreview::Material(name, material) => {
                self.raw_text = text_editor::Content::new();
                self.raw_binary = None;
                self.raw_name = String::new();

                if let Err(e) = self
                    .viewport_state
                    .renderer_mut()
                    .set_preview_material(name, material)
                {
                    if matches!(e, PreviewError::Unsupported) {
                        self.unsupported = true;
                        self.error = false;
                    } else {
                        self.unsupported = false;
                        self.error = true;
                    }
                } else {
                    self.unsupported = false;
                    self.error = false;
                }

                self.tab = PreviewTab::Viewport;
            }
            AssetPreview::Model(name, model, images) => {
                self.raw_text = text_editor::Content::new();
                self.raw_binary = None;
                self.raw_name = String::new();

                let srgb = cfg!(feature = "srgb-preview");

                if let Err(e) = self
                    .viewport_state
                    .renderer_mut()
                    .set_preview_model(name, model, images, srgb)
                {
                    if matches!(e, PreviewError::Unsupported) {
                        self.unsupported = true;
                        self.error = false;
                    } else {
                        self.unsupported = false;
                        self.error = true;
                    }
                } else {
                    self.unsupported = false;
                    self.error = false;
                }

                self.tab = PreviewTab::Viewport;
            }
        }

        Task::none()
    }

    /// Occurs when we want to request a preview asset, but need to check if the previewer is open.
    fn on_preview_request(&mut self, _: &mut AppState) -> Task<Message> {
        Task::done(Message::PreviewRequest)
    }
}

/// Style for the preview header.
fn preview_header_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_100)),
        border: Border {
            width: 1.0,
            color: palette::BACKGROUND_COLOR_LIGHT_100,
            radius: Radius::new(0.0).top_left(4.0).top_right(4.0),
        },
        ..Default::default()
    }
}

/// Style for the content header.
fn preview_content_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
        ..Default::default()
    }
}

/// Style for the overlay container.
fn preview_overlay_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            palette::BACKGROUND_COLOR_DEFAULT.scale_alpha(0.75),
        )),
        border: rounded(4.0),
        ..Default::default()
    }
}

/// Style for the footer container.
fn preview_footer_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_100)),
        border: Border {
            color: palette::BACKGROUND_COLOR_LIGHT_100,
            width: 1.0,
            radius: Radius::new(0.0).top_left(4.0).top_right(4.0),
        },
        ..Default::default()
    }
}

/// Style for the text preview.
fn text_editor_style(_: &Theme, _: text_editor::Status) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(palette::BACKGROUND_COLOR_LIGHT_050),
        border: rounded(0.0),
        icon: palette::TEXT_COLOR_DEFAULT,
        placeholder: palette::TEXT_COLOR_DEFAULT,
        value: palette::TEXT_COLOR_DEFAULT,
        selection: palette::PRIMARY_COLOR,
    }
}
