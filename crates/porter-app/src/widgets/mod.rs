mod binary;
mod header_divider;
mod laser;
mod resizable;
mod spinner;
mod text_wrap;
mod viewport;

pub use viewport::ViewportAction;
pub use viewport::ViewportState;

use std::borrow::Borrow;
use std::borrow::Cow;
use std::ops::RangeInclusive;
use std::time::Duration;

use iced::border::Radius;
use iced::border::rounded;

use iced::widget;
use iced::widget::Button;
use iced::widget::Checkbox;
use iced::widget::Container;
use iced::widget::PickList;
use iced::widget::ProgressBar;
use iced::widget::Scrollable;
use iced::widget::Slider;
use iced::widget::TextInput;
use iced::widget::Tooltip;
use iced::widget::container;
use iced::widget::text;

use iced::Alignment;
use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Rectangle;
use iced::Shadow;
use iced::Vector;

use crate::fonts;
use crate::palette;

/// Styled button widget.
pub fn button<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: widget::button::Catalog + 'a,
    Theme::Class<'a>: From<widget::button::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer,
{
    use widget::button;

    Button::new(content).style(|_, status| {
        let active = button::Style {
            background: None,
            border: Border {
                width: 1.0,
                color: palette::PRIMARY_COLOR,
                ..rounded(4.0)
            },
            shadow: Default::default(),
            text_color: palette::TEXT_COLOR_DEFAULT,
        };

        match status {
            button::Status::Active => active,
            button::Status::Hovered => button::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR_LIGHT_250,
                    ..active.border
                },
                ..active
            },
            button::Status::Pressed => button::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR_DARK_250,
                    ..active.border
                },
                ..active
            },
            button::Status::Disabled => button::Style {
                text_color: palette::TEXT_COLOR_DISABLED,
                border: Border {
                    color: palette::PRIMARY_COLOR.scale_alpha(0.3),
                    ..active.border
                },
                ..active
            },
        }
    })
}

/// Styled icon button.
pub fn icon_button<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: widget::button::Catalog + 'a,
    Theme::Class<'a>: From<widget::button::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer,
{
    use widget::button;

    Button::new(content).padding(0.0).style(|_, status| {
        let active = button::Style {
            text_color: palette::TEXT_COLOR_MUTED,
            ..Default::default()
        };

        match status {
            button::Status::Active => active,
            button::Status::Hovered | button::Status::Pressed => button::Style {
                text_color: palette::TEXT_COLOR_DEFAULT,
                ..active
            },
            _ => active,
        }
    })
}

/// Styled checkbox widget.
pub fn checkbox<'a, Message, Theme, Renderer>(
    label: impl Into<String>,
    is_checked: bool,
) -> Checkbox<'a, Message, Theme, Renderer>
where
    Theme: widget::checkbox::Catalog + 'a,
    Theme::Class<'a>: From<widget::checkbox::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    use widget::checkbox;

    Checkbox::new(label, is_checked)
        .size(20.0)
        .style(|_, status| {
            let active = checkbox::Style {
                background: Background::Color(palette::PRIMARY_COLOR),
                icon_color: palette::TEXT_COLOR_DEFAULT,
                border: rounded(4.0),
                text_color: Some(palette::TEXT_COLOR_DEFAULT),
            };

            match status {
                checkbox::Status::Active { .. } => active,
                checkbox::Status::Hovered { .. } => checkbox::Style {
                    background: Background::Color(palette::PRIMARY_COLOR_LIGHT_250),
                    ..active
                },
                checkbox::Status::Disabled { .. } => checkbox::Style {
                    background: Background::Color(palette::PRIMARY_COLOR_DARK_250),
                    text_color: Some(palette::TEXT_COLOR_DISABLED),
                    ..active
                },
            }
        })
}

/// Styled tooltip widget.
#[allow(dead_code)]
pub fn tooltip<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
    position: widget::tooltip::Position,
) -> Tooltip<'a, Message, Theme, Renderer>
where
    Theme: widget::container::Catalog,
    Theme::Class<'a>: From<widget::container::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    use widget::container;

    Tooltip::new(content, tooltip, position)
        .gap(8.0)
        .snap_within_viewport(true)
        .style(|_| container::Style {
            text_color: Some(palette::TEXT_COLOR_DEFAULT),
            background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_100)),
            border: Border {
                width: 1.0,
                color: palette::BACKGROUND_COLOR_LIGHT_100,
                ..rounded(4.0)
            },
            shadow: Shadow {
                color: palette::BACKGROUND_COLOR_LIGHT_025,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 6.0,
            },
        })
}

/// Styled pick list widget.
pub fn pick_list<'a, T, L, V, Message, Theme, Renderer>(
    options: L,
    selected: Option<V>,
    on_select: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone,
    Theme: widget::pick_list::Catalog + widget::overlay::menu::Catalog,
    <Theme as iced::widget::pick_list::Catalog>::Class<'a>:
        From<widget::pick_list::StyleFn<'a, Theme>>,
    <Theme as widget::overlay::menu::Catalog>::Class<'a>:
        From<widget::overlay::menu::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    use widget::overlay::menu;
    use widget::pick_list;

    PickList::new(options, selected, on_select)
        .style(|_, status| {
            let active = pick_list::Style {
                text_color: palette::TEXT_COLOR_DEFAULT,
                placeholder_color: palette::TEXT_COLOR_SECONDARY,
                handle_color: palette::PRIMARY_COLOR,
                background: Background::Color(palette::BACKGROUND_COLOR_DEFAULT),
                border: Border {
                    width: 1.0,
                    color: palette::PRIMARY_COLOR,
                    ..rounded(4.0)
                },
            };

            match status {
                pick_list::Status::Active => active,
                pick_list::Status::Hovered => pick_list::Style {
                    handle_color: palette::PRIMARY_COLOR_LIGHT_250,
                    border: Border {
                        color: palette::PRIMARY_COLOR_LIGHT_250,
                        ..active.border
                    },
                    ..active
                },
                pick_list::Status::Opened { .. } => pick_list::Style {
                    handle_color: palette::PRIMARY_COLOR_DARK_250,
                    border: Border {
                        color: palette::PRIMARY_COLOR_DARK_250,
                        ..active.border
                    },
                    ..active
                },
            }
        })
        .menu_style(|_| menu::Style {
            background: Background::Color(palette::BACKGROUND_COLOR_LIGHT_050),
            border: Border {
                width: 1.0,
                color: palette::PRIMARY_COLOR,
                ..rounded(4.0)
            },
            text_color: palette::TEXT_COLOR_SECONDARY,
            selected_text_color: palette::TEXT_COLOR_DEFAULT,
            selected_background: Background::Color(palette::PRIMARY_COLOR),
        })
}

/// Styled slider widget.
pub fn slider<'a, T, Message, Theme, F>(
    range: RangeInclusive<T>,
    value: T,
    on_change: F,
) -> Slider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + PartialOrd,
    Message: Clone,
    Theme: widget::slider::Catalog + 'a,
    Theme::Class<'a>: From<widget::slider::StyleFn<'a, Theme>>,
    F: Fn(T) -> Message + 'a,
{
    use widget::slider;

    Slider::new(range, value, on_change)
        .height(22.0)
        .style(|_, status| {
            let active_handle = slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 8,
                    border_radius: Radius::new(4.0),
                },
                background: Background::Color(palette::PRIMARY_COLOR),
                border_color: palette::PRIMARY_COLOR,
                border_width: 1.0,
            };

            let active = slider::Style {
                rail: slider::Rail {
                    backgrounds: (
                        Background::Color(palette::PRIMARY_COLOR),
                        Background::Color(Color::WHITE),
                    ),
                    width: 4.0,
                    border: rounded(2.0),
                },
                handle: active_handle,
            };

            match status {
                slider::Status::Active => active,
                slider::Status::Hovered => slider::Style {
                    handle: slider::Handle {
                        background: Background::Color(palette::PRIMARY_COLOR_LIGHT_250),
                        border_color: palette::PRIMARY_COLOR_LIGHT_250,
                        ..active.handle
                    },
                    ..active
                },
                slider::Status::Dragged => slider::Style {
                    handle: slider::Handle {
                        background: Background::Color(palette::PRIMARY_COLOR_DARK_250),
                        border_color: palette::PRIMARY_COLOR_DARK_250,
                        ..active.handle
                    },
                    ..active
                },
            }
        })
}

/// Styled list item widget.
pub fn list_item<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    item_index: usize,
    selected: bool,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: widget::button::Catalog + 'a,
    Theme::Class<'a>: From<widget::button::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer,
{
    use widget::button;

    let background_default = if selected {
        palette::PRIMARY_COLOR
    } else if item_index % 2 > 0 {
        palette::BACKGROUND_COLOR_LIGHT_025
    } else {
        palette::BACKGROUND_COLOR_LIGHT_050
    };

    Button::new(content)
        .style(move |_, status| {
            let active = button::Style {
                background: Some(Background::Color(background_default)),
                border: Border {
                    width: 1.0,
                    color: background_default,
                    ..rounded(4.0)
                },
                shadow: Default::default(),
                text_color: palette::TEXT_COLOR_DEFAULT,
            };

            match status {
                button::Status::Active => active,
                button::Status::Hovered | button::Status::Pressed => {
                    if selected {
                        active
                    } else {
                        button::Style {
                            background: Some(Background::Color(
                                palette::PRIMARY_COLOR_DARK_250.scale_alpha(0.5),
                            )),
                            border: Border {
                                color: palette::PRIMARY_COLOR_LIGHT_250.scale_alpha(0.75),
                                ..active.border
                            },
                            ..active
                        }
                    }
                }
                _ => active,
            }
        })
        .padding(0.0)
}

/// Styled text input widget.
pub fn text_input<'a, Message, Theme, Renderer>(
    placeholder: &str,
    value: &str,
) -> TextInput<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: widget::text_input::Catalog + 'a,
    <Theme as iced::widget::text_input::Catalog>::Class<'a>:
        From<widget::text_input::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
{
    use widget::text_input;

    TextInput::new(placeholder, value).style(|_, status| {
        let active = text_input::Style {
            background: Background::Color(palette::BACKGROUND_COLOR_DEFAULT),
            border: Border {
                width: 1.0,
                color: palette::PRIMARY_COLOR,
                ..rounded(4.0)
            },
            icon: palette::TEXT_COLOR_DEFAULT,
            placeholder: palette::TEXT_COLOR_SECONDARY,
            value: palette::TEXT_COLOR_DEFAULT,
            selection: palette::PRIMARY_COLOR,
        };

        match status {
            text_input::Status::Active => active,
            text_input::Status::Hovered | text_input::Status::Focused { .. } => text_input::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR_LIGHT_250,
                    ..active.border
                },
                ..active
            },
            text_input::Status::Disabled => text_input::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR.scale_alpha(0.3),
                    ..active.border
                },
                icon: palette::TEXT_COLOR_DISABLED,
                placeholder: palette::TEXT_COLOR_DISABLED,
                value: palette::TEXT_COLOR_DISABLED,
                ..active
            },
        }
    })
}

/// Styled options as a switch button.
pub fn switch_button<'a, Message, Theme, Renderer>(
    options: impl IntoIterator<Item = (&'a str, Option<Message>, bool)>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: widget::container::Catalog + widget::button::Catalog + widget::text::Catalog + 'a,
    <Theme as iced::widget::container::Catalog>::Class<'a>:
        From<widget::container::StyleFn<'a, Theme>>,
    <Theme as iced::widget::button::Catalog>::Class<'a>: From<widget::button::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
{
    use widget::button;
    use widget::container;
    use widget::horizontal_space;
    use widget::row;

    let mut options: Vec<_> = options
        .into_iter()
        .map(|(text, on_press, selected)| {
            Element::from(
                button(text)
                    .on_press_maybe(on_press)
                    .style(move |_, status| {
                        let active = button::Style {
                            background: if selected {
                                Some(Background::Color(palette::PRIMARY_COLOR))
                            } else {
                                None
                            },
                            border: Border {
                                width: 1.0,
                                color: Color::TRANSPARENT,
                                ..rounded(2.0)
                            },
                            text_color: palette::TEXT_COLOR_DEFAULT,
                            ..Default::default()
                        };

                        match status {
                            button::Status::Active => active,
                            button::Status::Hovered | button::Status::Pressed => button::Style {
                                background: if selected {
                                    Some(Background::Color(palette::PRIMARY_COLOR))
                                } else {
                                    Some(Background::Color(palette::PRIMARY_COLOR.scale_alpha(0.1)))
                                },
                                ..active
                            },
                            _ => active,
                        }
                    }),
            )
        })
        .collect();

    for i in (0..options.len().max(1) - 1).rev() {
        options.insert(
            i + 1,
            Element::from(
                container(
                    horizontal_space()
                        // Ideally, this should be padded to the height of the parent container.
                        .width(1.0)
                        .height(20.0),
                )
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::WHITE.scale_alpha(0.1))),
                    ..Default::default()
                }),
            ),
        );
    }

    container(row(options).align_y(Alignment::Center))
        .style(|_: &Theme| container::Style {
            background: None,
            border: Border {
                width: 1.0,
                color: palette::PRIMARY_COLOR,
                ..rounded(4.0)
            },
            ..Default::default()
        })
        .padding(2.0)
        .width(Length::Shrink)
        .height(Length::Shrink)
}

/// Styled button as a link widget.
pub fn link<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: widget::button::Catalog + 'a,
    Theme::Class<'a>: From<widget::button::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer,
{
    use widget::button;

    Button::new(content)
        .padding(0)
        .style(|_, status| match status {
            button::Status::Active | button::Status::Disabled => button::Style {
                text_color: palette::TEXT_COLOR_LINK,
                background: None,
                ..Default::default()
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                text_color: palette::TEXT_COLOR_LINK_HOVER,
                background: None,
                ..Default::default()
            },
        })
}

/// Styled bytton as a tab widget.
pub fn tab<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    selected: bool,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: widget::button::Catalog + 'a,
    Theme::Class<'a>: From<widget::button::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer,
{
    use widget::button;

    Button::new(content)
        .padding([4.0, 8.0])
        .style(move |_, status| match status {
            button::Status::Active | button::Status::Disabled => {
                if selected {
                    button::Style {
                        text_color: palette::TEXT_COLOR_DEFAULT,
                        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
                        border: Border {
                            width: 1.0,
                            color: palette::BACKGROUND_COLOR_LIGHT_050,
                            radius: Radius::new(0.0).bottom_left(8.0).bottom_right(8.0),
                        },
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        text_color: palette::TEXT_COLOR_MUTED,
                        background: None,
                        ..Default::default()
                    }
                }
            }
            button::Status::Hovered | button::Status::Pressed => {
                if selected {
                    button::Style {
                        text_color: palette::TEXT_COLOR_DEFAULT,
                        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
                        border: Border {
                            width: 1.0,
                            color: palette::BACKGROUND_COLOR_LIGHT_050,
                            radius: Radius::new(0.0).bottom_left(8.0).bottom_right(8.0),
                        },
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        text_color: palette::TEXT_COLOR_SECONDARY,
                        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
                        border: Border {
                            width: 1.0,
                            color: palette::BACKGROUND_COLOR_DEFAULT,
                            radius: Radius::new(0.0).bottom_left(8.0).bottom_right(8.0),
                        },
                        ..Default::default()
                    }
                }
            }
        })
}

/// Styled scrollable widget.
pub fn scrollable<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: widget::scrollable::Catalog + 'a,
    Theme::Class<'a>: From<widget::scrollable::StyleFn<'a, Theme>>,
    Renderer: iced::advanced::Renderer,
{
    use widget::scrollable;

    Scrollable::new(content).style(|_, status| {
        let active_rail = scrollable::Rail {
            background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
            border: Border {
                width: 1.0,
                color: palette::BACKGROUND_COLOR_LIGHT_050,
                ..rounded(0.0)
            },
            scroller: scrollable::Scroller {
                color: palette::BACKGROUND_COLOR_LIGHT_100,
                border: Border {
                    width: 1.0,
                    color: palette::BACKGROUND_COLOR_LIGHT_150,
                    ..rounded(2.0)
                },
            },
        };

        let disabled_rail = scrollable::Rail {
            border: Border {
                width: 1.0,
                color: palette::BACKGROUND_COLOR_LIGHT_100,
                ..rounded(2.0)
            },
            ..active_rail
        };

        match status {
            scrollable::Status::Active {
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            } => scrollable::Style {
                container: Default::default(),
                vertical_rail: if is_vertical_scrollbar_disabled {
                    disabled_rail
                } else {
                    active_rail
                },
                horizontal_rail: if is_horizontal_scrollbar_disabled {
                    disabled_rail
                } else {
                    active_rail
                },
                gap: None,
            },
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
                ..
            } => scrollable::Style {
                container: Default::default(),
                vertical_rail: if is_vertical_scrollbar_disabled {
                    disabled_rail
                } else {
                    active_rail
                },
                horizontal_rail: if is_horizontal_scrollbar_disabled {
                    disabled_rail
                } else {
                    active_rail
                },
                gap: None,
            },
            scrollable::Status::Dragged {
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
                ..
            } => scrollable::Style {
                container: Default::default(),
                vertical_rail: if is_vertical_scrollbar_disabled {
                    disabled_rail
                } else {
                    active_rail
                },
                horizontal_rail: if is_horizontal_scrollbar_disabled {
                    disabled_rail
                } else {
                    active_rail
                },
                gap: None,
            },
        }
    })
}

/// Styled progress bar widget.
pub fn progress_bar<'a, Theme>(range: RangeInclusive<f32>, value: f32) -> ProgressBar<'a, Theme>
where
    Theme: widget::progress_bar::Catalog + 'a,
    Theme::Class<'a>: From<widget::progress_bar::StyleFn<'a, Theme>>,
{
    use widget::progress_bar;

    ProgressBar::new(range, value).style(|_| progress_bar::Style {
        background: Background::Color(palette::BACKGROUND_COLOR_LIGHT_050),
        bar: Background::Color(palette::PRIMARY_COLOR),
        border: rounded(4.0),
    })
}

/// Spinner progress indicator widget.
pub fn spinner<'a>() -> spinner::Spinner<'a> {
    spinner::Spinner::new()
        .size(48.0)
        .bar_height(4.0)
        .style(spinner::Style {
            track_color: palette::BACKGROUND_COLOR_DEFAULT,
            bar_color: palette::PRIMARY_COLOR,
        })
        .cycle_duration(Duration::from_secs(2))
        .rotation_duration(Duration::from_secs(2))
}

/// Preview viewport widget.
pub fn viewport<'a, Message, Theme, Renderer, A>(
    state: &'a ViewportState,
    on_action: A,
) -> viewport::Viewport<'a, Message, Theme, Renderer, A>
where
    Message: Clone,
    Renderer: iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>,
    A: Fn(ViewportAction) -> Message + 'a,
{
    viewport::Viewport::new(state, on_action)
}

/// Binary hex viewer widget.
pub fn binary<Message, Theme, Renderer>(
    buffer: &[u8],
) -> binary::Binary<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
{
    binary::Binary::new(buffer)
        .font(fonts::BINARY_FONT)
        .style(binary::Style {
            background: Background::Color(palette::BACKGROUND_COLOR_LIGHT_050),
            hex_color: palette::TEXT_COLOR_DEFAULT,
            text_color: palette::TEXT_COLOR_DEFAULT,
            offset_color: palette::TEXT_COLOR_INFO,
            header_color: palette::TEXT_COLOR_INFO,
        })
}

/// Laser animated background widget.
pub fn laser<'a, Message, Theme, Renderer>() -> laser::Laser<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    laser::Laser::new()
}

/// Resizable widget.
#[allow(dead_code)]
pub fn resizable<'a, Message, Theme, Renderer, R>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    on_resize: R,
) -> resizable::Resizable<'a, Message, Theme, Renderer, R>
where
    R: Fn(Rectangle<f32>) -> Message,
{
    resizable::Resizable::new(content, on_resize)
}

/// Text wrapping with ellipsis widget.
pub fn text_wrap<'a, Message, Theme, Renderer>(
    content: impl Into<Cow<'a, str>>,
) -> text_wrap::TextWrap<'a, Message, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer,
{
    text_wrap::TextWrap::new(content)
}

/// Header divider widget.
pub fn header_divider<'a, Message, Theme, Renderer, D>(
    on_drag: D,
    on_release: Message,
) -> header_divider::HeaderDivider<'a, Message, Theme, Renderer, D>
where
    Message: Clone,
    Theme: container::Catalog,
    D: Fn(f32) -> Message,
{
    header_divider::HeaderDivider::new(on_drag, on_release)
}
