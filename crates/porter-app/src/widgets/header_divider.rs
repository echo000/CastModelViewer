use std::marker::PhantomData;

use iced::advanced;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse::Cursor;
use iced::advanced::mouse::Interaction;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree::State;
use iced::advanced::widget::tree::Tag;

use iced::widget::container;

use iced::Background;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Size;
use iced::mouse;

/// State of the divider.
#[derive(Default, Clone, Copy)]
struct DividerState {
    is_hovered: bool,
    drag_origin: Option<Point>,
}

/// A virtual list header column divider.
pub struct HeaderDivider<'a, Message, Theme, Renderer, D>
where
    Theme: container::Catalog,
{
    width: Length,
    height: Length,
    class: Theme::Class<'a>,
    on_drag: D,
    on_release: Message,
    _phantom: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer, D> HeaderDivider<'a, Message, Theme, Renderer, D>
where
    Message: Clone,
    Theme: container::Catalog,
    D: Fn(f32) -> Message,
{
    /// Constructs a new isntance of [`HeaderDivider`].
    pub fn new(on_drag: D, on_release: Message) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            class: Theme::default(),
            on_drag,
            on_release,
            _phantom: PhantomData,
        }
    }

    /// Sets the width of the [`HeaderDivider`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`HeaderDivider`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the style of the [`HeaderDivider`].
    pub fn style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<container::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as container::StyleFn<'a, Theme>).into();
        self
    }
}

impl<Message, Theme, Renderer, D> Widget<Message, Theme, Renderer>
    for HeaderDivider<'_, Message, Theme, Renderer, D>
where
    Message: Clone,
    Theme: container::Catalog,
    Renderer: advanced::Renderer,
    D: Fn(f32) -> Message,
{
    fn tag(&self) -> Tag {
        Tag::of::<DividerState>()
    }

    fn state(&self) -> State {
        State::new(DividerState::default())
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
        layout: advanced::Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state: &mut DividerState = tree.state.downcast_mut();

        let bounds = layout.bounds();
        let bounds = Rectangle {
            x: bounds.x - 5.0,
            y: bounds.y,
            width: bounds.width + 10.0,
            height: bounds.height,
        };

        let position_over = cursor.position_over(bounds);
        let is_hovered = position_over.is_some();

        if state.is_hovered != is_hovered {
            state.is_hovered = is_hovered;
            // Right now, cursors don't change unless a redraw has been scheduled.
            // mouse_interaction is not called, so updates won't happen.
            shell.request_redraw();
        }

        if let Event::Mouse(event) = event {
            match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(origin) = position_over {
                        state.drag_origin = Some(origin);
                        shell.capture_event();
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.drag_origin.take().is_some() {
                        shell.publish(self.on_release.clone());
                        shell.capture_event();
                    }
                }
                mouse::Event::CursorMoved { position } => {
                    if let Some(origin) = &mut state.drag_origin {
                        let shift = (*position - *origin).x;

                        *origin = *position;

                        shell.publish((self.on_drag)(shift));
                        shell.capture_event();
                    }
                }
                _ => {
                    // Nothing.
                }
            }
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let style = theme.style(&self.class);

        renderer.fill_quad(
            advanced::renderer::Quad {
                bounds: layout.bounds(),
                border: style.border,
                shadow: style.shadow,
            },
            style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: advanced::Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> Interaction {
        let state: &DividerState = tree.state.downcast_ref();

        if state.drag_origin.is_some() || state.is_hovered {
            Interaction::ResizingHorizontally
        } else {
            Interaction::None
        }
    }
}

impl<'a, Message, Theme, Renderer, D> From<HeaderDivider<'a, Message, Theme, Renderer, D>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: container::Catalog + 'a,
    Renderer: advanced::Renderer + 'a,
    D: Fn(f32) -> Message + 'a,
{
    fn from(
        value: HeaderDivider<'a, Message, Theme, Renderer, D>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(value)
    }
}
