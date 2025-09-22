use std::borrow::Cow;
use std::marker::PhantomData;

use iced::advanced;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse::Cursor;
use iced::advanced::text::Alignment;
use iced::advanced::text::Paragraph;
use iced::advanced::widget::Tree;

use iced::widget::text;
use iced::widget::text::LineHeight;
use iced::widget::text::Shaping;
use iced::widget::text::Wrapping;

use iced::alignment::Vertical;

use iced::Color;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::Padding;
use iced::Pixels;
use iced::Point;
use iced::Rectangle;
use iced::Size;

use unicode_segmentation::UnicodeSegmentation;

/// A text widget that handles wrapping with ellipsis.
pub struct TextWrap<'a, Message, Theme, Renderer>
where
    Theme: text::Catalog,
{
    content: Cow<'a, str>,
    size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    padding: Padding,
    align_x: advanced::text::Alignment,
    align_y: Vertical,
    font: Option<Font>,
    shaping: Shaping,
    class: Theme::Class<'a>,
    _phantom: PhantomData<(Message, Renderer)>,
}

impl<'a, Message, Theme, Renderer> TextWrap<'a, Message, Theme, Renderer>
where
    Theme: text::Catalog,
    Renderer: advanced::text::Renderer,
{
    /// Constructs a new instance of [`TextWrap`].
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self {
            content: content.into(),
            size: None,
            line_height: LineHeight::default(),
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::ZERO,
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: Shaping::Basic,
            class: Theme::default(),
            _phantom: PhantomData,
        }
    }

    /// Sets the size of the [`TextWrap`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`TextWrap`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the [`Font`] of the [`TextWrap`].
    ///
    /// [`Font`]: iced::Font
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`TextWrap`].
    pub fn style(mut self, style: impl Fn(&Theme) -> text::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<text::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as text::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the color of the [`TextWrap`].
    pub fn color(self, color: impl Into<Color>) -> Self
    where
        Theme::Class<'a>: From<text::StyleFn<'a, Theme>>,
    {
        let color = color.into();

        self.style(move |_| text::Style { color: Some(color) })
    }

    /// Sets the width of the [`TextWrap`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`TextWrap`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`TextWrap`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the [`Horizontal`] of the [`TextWrap`].
    pub fn align_x(mut self, alignment: impl Into<Alignment>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the [`Vertical`] of the [`TextWrap`].
    pub fn align_y(mut self, alignment: impl Into<Vertical>) -> Self {
        self.align_y = alignment.into();
        self
    }

    /// Sets the [`Shaping`] strategy of the [`TextWrap`].
    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TextWrap<'_, Message, Theme, Renderer>
where
    Theme: text::Catalog,
    Renderer: advanced::text::Renderer<Font = Font>,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds().shrink(self.padding);
        let width = bounds.width;

        let size = self.size.unwrap_or_else(|| renderer.default_size());
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let text = advanced::text::Text {
            content: self.content.as_ref(),
            size,
            line_height: self.line_height,
            bounds: Size::INFINITY,
            font,
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: self.shaping,
            wrapping: Wrapping::None,
        };

        let paragraph = <Renderer as advanced::text::Renderer>::Paragraph::with_text(text);
        let measure_full = paragraph.min_bounds().width;

        let render_str = if measure_full > width {
            let mut index = 0;

            while let Some(position) = paragraph.grapheme_position(0, index) {
                if position.x + size.0 > width {
                    break;
                }
                index += 1;
            }

            if index > 1 {
                index -= 1;
            }

            self.content
                .graphemes(true)
                .take(index)
                .chain(["…"])
                .collect::<Vec<_>>()
                .join("")
        } else {
            self.content.to_string()
        };

        let x = match self.align_x {
            Alignment::Left | Alignment::Default | Alignment::Justified => bounds.x,
            Alignment::Center => bounds.center_x(),
            Alignment::Right => bounds.x + bounds.width,
        };

        let y = match self.align_y {
            Vertical::Top => bounds.y,
            Vertical::Center => bounds.center_y(),
            Vertical::Bottom => bounds.y + bounds.height,
        };

        if width > 0.0 {
            let text = advanced::Text {
                content: render_str,
                size,
                bounds: bounds.size(),
                line_height: self.line_height,
                font,
                align_x: self.align_x,
                align_y: self.align_y,
                shaping: self.shaping,
                wrapping: Wrapping::None,
            };

            renderer.fill_text(
                text,
                Point { x, y },
                theme.style(&self.class).color.unwrap_or(style.text_color),
                *viewport,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<TextWrap<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: text::Catalog + 'a,
    Renderer: advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn from(text: TextWrap<'a, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text)
    }
}
