use std::marker::PhantomData;
use std::time::Duration;
use std::time::Instant;

use iced::advanced;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse::Cursor;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;

use iced::border::radius;
use iced::gradient::Linear;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Gradient;
use iced::Length;
use iced::Rectangle;
use iced::Size;
use iced::window;

use crate::palette;

/// Maximum safe floating point integer.
const MAX_SAFE_FLOAT: f32 = (1 << f32::MANTISSA_DIGITS) as f32;

/// Refresh interval.
const REFRESH_INTERVAL: Duration = Duration::from_millis(33);

/// Color for the gradient base.
const COLOR_BASE: Color = Color::from_rgb8(0x23, 0xCE, 0x6B);
/// Color for the gradient line.
const COLOR_LINE: Color = Color::from_rgb8(0x12, 0x69, 0x36);

/// A laser effect animated container widget.
pub struct Laser<'a, Message, Theme, Renderer> {
    width: Length,
    height: Length,
    radius: f32,
    padding: f32,
    _phantom: PhantomData<&'a (Message, Theme, Renderer)>,
}

/// State for the laser widget.
struct State {
    start: Instant,
    delta: f32,
    focused: bool,
}

impl<Message, Theme, Renderer> Laser<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: advanced::Renderer,
{
    /// Constructs a new isntance of [`Laser`].
    pub fn new() -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            radius: 0.0,
            padding: 1.0,
            _phantom: PhantomData,
        }
    }

    /// Sets the width of the [`Laser`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Laser`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the border radius of the [`Laser`] widget.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the padding of the [`Laser`] widget.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Laser<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            start: Instant::now(),
            delta: 0.0,
            focused: true,
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: advanced::Layout<'_>,
        _cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::Focused) = event {
            state.focused = true;
            shell.request_redraw();
        } else if let Event::Window(window::Event::Unfocused) = event {
            state.focused = false;
        }

        if !state.focused {
            return;
        }

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let distance = now.duration_since(state.start);
            let distance_f32 = distance.as_secs_f32();

            if distance_f32 > MAX_SAFE_FLOAT {
                state.start = *now;
                state.delta = 0.0;
            } else {
                state.delta = distance_f32;
            }

            if state.focused {
                shell.request_redraw_at(*now + REFRESH_INTERVAL);
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let angle = (state.delta / 5.0) * std::f32::consts::TAU;
        let angle = angle % std::f32::consts::TAU;

        let background = Background::Gradient(Gradient::Linear(
            Linear::new(angle)
                .add_stop(0.0, COLOR_BASE)
                .add_stop(0.5, COLOR_LINE)
                .add_stop(1.0, COLOR_BASE),
        ));

        renderer.fill_quad(
            advanced::renderer::Quad {
                bounds: layout.bounds(),
                border: Border {
                    radius: radius(self.radius),
                    ..Default::default()
                },
                shadow: Default::default(),
            },
            background,
        );

        renderer.fill_quad(
            advanced::renderer::Quad {
                bounds: layout.bounds().shrink(self.padding),
                border: Border {
                    radius: radius(self.radius - 2.0),
                    ..Default::default()
                },
                shadow: Default::default(),
            },
            Background::Color(palette::BACKGROUND_COLOR_LIGHT_050),
        );
    }
}

impl<Message, Theme, Renderer> Default for Laser<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: advanced::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> From<Laser<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(value: Laser<'a, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(value)
    }
}
