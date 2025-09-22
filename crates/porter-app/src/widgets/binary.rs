use std::marker::PhantomData;

use iced::advanced;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse::Cursor;
use iced::advanced::renderer::Quad;
use iced::advanced::text::Alignment;
use iced::advanced::text::Paragraph;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;

use iced::widget::text::LineHeight;
use iced::widget::text::Shaping;
use iced::widget::text::Wrapping;

use iced::alignment::Vertical;

use iced::Background;
use iced::Color;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::Pixels;
use iced::Point;
use iced::Rectangle;
use iced::Size;

/// Hex lookup table.
const HEX_LOOKUP: &[u8; 16] = b"0123456789ABCDEF";

/// Header content.
const HEADER_CONTENT: &str =
    "  Offset   00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F  Decoded text";
/// Offset content.
const OFFSET_CONTENT: &str = " 00000000  ";
/// Hex number content.
const HEX_CONTENT: &str = "00 11 22 33 44 55 66 77 88 99 AA BB CC DD EE FF  ";
/// Text decoded content.
const TEXT_CONTENT: &str = "DEADBEEFDEADBEEF ";

/// Mapping to ANSI printable characters to UTF-8.
const ANSI_TO_UNICODE: [char; 32] = [
    '\u{20AC}', '.', '\u{201A}', '\u{192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{2C6}', '\u{2030}', '\u{160}', '\u{2039}', '\u{152}', '.', '\u{17D}', '.', '.', '\u{2018}',
    '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}', '\u{2DC}', '\u{2122}',
    '\u{161}', '\u{203A}', '\u{153}', '.', '\u{17E}', '\u{178}',
];

/// A binary hex viewer widget.
pub struct Binary<'a, Message, Theme, Renderer> {
    buffer: &'a [u8],
    style: Style,
    size: Option<Pixels>,
    font: Option<Font>,
    _phantom: PhantomData<(Message, Theme, Renderer)>,
}

/// Style for the binary widget.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The background color.
    pub background: Background,
    /// The color of the hex data.
    pub hex_color: Color,
    /// The color of the decoded text data.
    pub text_color: Color,
    /// The color of the offsets.
    pub offset_color: Color,
    /// The color of the header text.
    pub header_color: Color,
}

/// State used by the binary widget.
#[derive(Default)]
struct State {
    text_size: Pixels,
    text_font: Font,
    offset_bounds: Size<f32>,
    hex_bounds: Size<f32>,
    text_bounds: Size<f32>,
    full_width: f32,
}

impl<'a, Message, Theme, Renderer> Binary<'a, Message, Theme, Renderer>
where
    Renderer: advanced::text::Renderer,
{
    /// Constructs a new instance of [`Binary`].
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            style: Default::default(),
            size: None,
            font: None,
            _phantom: PhantomData,
        }
    }

    /// Sets the style of this [`Binary`].
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the size of the [`Binary`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`Font`] of the [`Binary`].
    ///
    /// [`Font`]: iced::Font
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Binary<'_, Message, Theme, Renderer>
where
    Renderer: advanced::text::Renderer<Font = Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_mut::<State>();

        let size = self.size.unwrap_or_else(|| renderer.default_size());
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let rows = self.buffer.len().div_ceil(16) + 1; // Plus one for the header, which is fixed to the top.

        if state.text_size == size && state.text_font == font {
            let height = rows * state.hex_bounds.height.ceil() as usize;
            let width =
                state.offset_bounds.width + state.hex_bounds.width + state.text_bounds.width;

            return layout::atomic(limits, Length::Fixed(width), Length::Fixed(height as f32));
        }

        state.text_size = size;
        state.text_font = font;

        let text = advanced::text::Text {
            content: OFFSET_CONTENT,
            size,
            line_height: LineHeight::default(),
            bounds: Size::INFINITY,
            font,
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: Shaping::Basic,
            wrapping: Wrapping::None,
        };

        let paragraph = <Renderer as advanced::text::Renderer>::Paragraph::with_text(text);
        let offset_bounds = paragraph.min_bounds();

        state.offset_bounds = offset_bounds;

        let text = advanced::text::Text {
            content: HEX_CONTENT,
            size,
            line_height: LineHeight::default(),
            bounds: Size::INFINITY,
            font,
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: Shaping::Basic,
            wrapping: Wrapping::None,
        };

        let paragraph = <Renderer as advanced::text::Renderer>::Paragraph::with_text(text);
        let hex_bounds = paragraph.min_bounds();

        state.hex_bounds = hex_bounds;

        let text = advanced::text::Text {
            content: TEXT_CONTENT,
            size,
            line_height: LineHeight::default(),
            bounds: Size::INFINITY,
            font,
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: Shaping::Basic,
            wrapping: Wrapping::None,
        };

        let paragraph = <Renderer as advanced::text::Renderer>::Paragraph::with_text(text);
        let text_bounds = paragraph.min_bounds();

        state.text_bounds = text_bounds;

        let height = rows * state.hex_bounds.height.ceil() as usize;
        let width = state.offset_bounds.width + state.hex_bounds.width + state.text_bounds.width;

        state.full_width = width;

        layout::atomic(limits, Length::Fixed(width), Length::Fixed(height as f32))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let start_y = viewport.y - bounds.y;

        let size = Size::new(state.full_width, bounds.height);
        let position = Point::new(bounds.x, bounds.y + state.hex_bounds.height);

        let bounds = Rectangle::new(position, size);
        let row_height = state.hex_bounds.height.ceil() as usize;

        let skip = (start_y / state.hex_bounds.height.ceil()).floor() as usize;
        let take = (viewport.height / state.hex_bounds.height.ceil()).ceil() as usize + 1; // Plus one for the header, which is fixed to the top.

        let mut buffer = String::with_capacity((16 * 2) + 16);

        for (index, row) in self.buffer.chunks(16).enumerate().skip(skip).take(take) {
            let shift = index * row_height;

            let text = advanced::Text {
                content: format!(" {:08X} ", index * 16),
                size: state.text_size,
                bounds: bounds.size(),
                line_height: LineHeight::default(),
                font: state.text_font,
                align_x: Alignment::Left,
                align_y: Vertical::Top,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            };

            renderer.fill_text(
                text,
                Point {
                    x: bounds.x,
                    y: bounds.y + shift as f32,
                },
                self.style.offset_color,
                bounds,
            );

            buffer.clear();

            for byte in row {
                buffer.push(HEX_LOOKUP[(byte >> 4) as usize] as char);
                buffer.push(HEX_LOOKUP[(byte & 0x0F) as usize] as char);
                buffer.push(' ');
            }

            let text = advanced::Text {
                content: buffer.clone(),
                size: state.text_size,
                bounds: bounds.size(),
                line_height: LineHeight::default(),
                font: state.text_font,
                align_x: Alignment::Left,
                align_y: Vertical::Top,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            };

            renderer.fill_text(
                text,
                Point {
                    x: bounds.x + state.offset_bounds.width,
                    y: bounds.y + shift as f32,
                },
                self.style.hex_color,
                Rectangle::INFINITE,
            );

            buffer.clear();

            for byte in row {
                let character = *byte as char;

                // Standard Ascii + Latin extended codes.
                if character.is_ascii_alphanumeric()
                    || character.is_ascii_punctuation()
                    || 0x20 == *byte
                    || (0xA0u8..=0xFFu8).contains(byte)
                {
                    buffer.push(character);
                } else if 0x80 == *byte
                    || (0x82u8..=0x8Cu8).contains(byte)
                    || 0x8E == *byte
                    || (0x91u8..=0x9Cu8).contains(byte)
                    || (0x9Eu8..=0x9Fu8).contains(byte)
                {
                    buffer.push(ANSI_TO_UNICODE[(*byte - 0x80) as usize]);
                } else {
                    buffer.push('.');
                }
            }

            let text = advanced::Text {
                content: buffer.clone(),
                size: state.text_size,
                bounds: bounds.size(),
                line_height: LineHeight::default(),
                font: state.text_font,
                align_x: Alignment::Left,
                align_y: Vertical::Top,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            };

            renderer.fill_text(
                text,
                Point {
                    x: bounds.x + state.offset_bounds.width + state.hex_bounds.width,
                    y: bounds.y + shift as f32,
                },
                self.style.text_color,
                Rectangle::INFINITE,
            );
        }

        renderer.with_layer(*viewport, |renderer| {
            let position = Point::new(bounds.x, viewport.y);
            let size = Size::new(bounds.width, state.hex_bounds.height);
            let quad_bounds = Rectangle::new(position, size);

            renderer.fill_quad(
                Quad {
                    bounds: quad_bounds,
                    ..Default::default()
                },
                self.style.background,
            );

            let text = advanced::Text {
                content: HEADER_CONTENT.to_owned(),
                size: state.text_size,
                bounds: bounds.size(),
                line_height: LineHeight::default(),
                font: state.text_font,
                align_x: Alignment::Left,
                align_y: Vertical::Top,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            };

            renderer.fill_text(
                text,
                Point {
                    x: bounds.x,
                    y: viewport.y,
                },
                self.style.header_color,
                Rectangle::INFINITE,
            );
        });
    }
}

impl<'a, Message, Theme, Renderer> From<Binary<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn from(value: Binary<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Background::Color(Color::TRANSPARENT),
            hex_color: Color::WHITE,
            text_color: Color::WHITE,
            offset_color: Color::BLACK,
            header_color: Color::BLACK,
        }
    }
}
