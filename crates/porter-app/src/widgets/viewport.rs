use std::marker::PhantomData;
use std::time::Instant;

use iced::advanced;
use iced::advanced::Layout;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::renderer::Quad;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;

use iced::keyboard;
use iced::keyboard::Key;

use iced::widget::canvas::Image;
use iced::widget::image::Handle;

use iced::window;

use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Size;
use iced::Vector;

use porter_preview::PreviewKeyState;
use porter_preview::PreviewRenderer;

use crate::PreviewControlScheme;
use crate::palette;

/// Preview viewport rendering widget.
pub struct Viewport<'a, Message, Theme, Renderer, A> {
    state: &'a ViewportState,
    on_action: A,
    _phantom: PhantomData<&'a (Message, Theme, Renderer)>,
}

/// Preview viewport global state.
pub struct ViewportState {
    renderer: PreviewRenderer,
    bounds: Rectangle<f32>,
    dirty: Option<Instant>,
    cache: Option<Handle>,
}

/// Actions performed on the viewport state.
#[derive(Debug, Clone)]
pub enum ViewportAction {
    Resized(Rectangle<f32>),
    Cached(Handle, Instant),
    ResetView,
    ToggleGrid,
    ToggleBones,
    ToggleWireframe,
    ToggleShaded,
    CycleMaterial,
    ScrollDelta(f32),
    MouseMove(Vector<f32>, Option<mouse::Button>, keyboard::Modifiers),
}

/// Internal state for events.
struct State {
    mouse_position: Point<f32>,
    mouse_button: Option<mouse::Button>,
    keyboard_modifiers: keyboard::Modifiers,
}

impl<'a, Message, Theme, Renderer, A> Viewport<'a, Message, Theme, Renderer, A>
where
    Message: Clone + 'a,
    Renderer: advanced::image::Renderer<Handle = advanced::image::Handle>,
    A: Fn(ViewportAction) -> Message + 'a,
{
    /// Creates a [`Viewport`] with the given state and action callback.
    pub fn new(state: &'a ViewportState, on_action: A) -> Self {
        Self {
            state,
            on_action,
            _phantom: PhantomData,
        }
    }
}

impl<Message, Theme, Renderer, A> Widget<Message, Theme, Renderer>
    for Viewport<'_, Message, Theme, Renderer, A>
where
    Message: Clone,
    Renderer: advanced::image::Renderer<Handle = advanced::image::Handle>,
    A: Fn(ViewportAction) -> Message,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            mouse_position: Point::ORIGIN,
            mouse_button: None,
            keyboard_modifiers: keyboard::Modifiers::empty(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        layout::atomic(limits, Length::Fill, Length::Fill)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        use ViewportAction::*;

        let new_bounds = layout.bounds();

        if new_bounds != self.state.bounds {
            shell.publish((self.on_action)(Resized(new_bounds)));
            shell.request_redraw();
            return;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                if !modifiers.is_empty() || shell.is_event_captured() {
                    return;
                }

                match key.as_ref() {
                    Key::Character("r") => {
                        shell.publish((self.on_action)(ResetView));
                        shell.capture_event();

                        shell.redraw_request();
                    }
                    Key::Character("g") => {
                        shell.publish((self.on_action)(ToggleGrid));
                        shell.capture_event();

                        shell.redraw_request();
                    }
                    Key::Character("b") => {
                        shell.publish((self.on_action)(ToggleBones));
                        shell.capture_event();

                        shell.redraw_request();
                    }
                    Key::Character("w") => {
                        shell.publish((self.on_action)(ToggleWireframe));
                        shell.capture_event();

                        shell.redraw_request();
                    }
                    Key::Character("m") => {
                        shell.publish((self.on_action)(ToggleShaded));
                        shell.capture_event();

                        shell.redraw_request();
                    }
                    Key::Character("n") => {
                        shell.publish((self.on_action)(CycleMaterial));
                        shell.capture_event();

                        shell.redraw_request();
                    }
                    _ => {
                        // Not used key.
                    }
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                tree.state.downcast_mut::<State>().keyboard_modifiers = *modifiers;
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if !cursor.is_over(layout.bounds()) {
                    return;
                }

                let state = tree.state.downcast_mut::<State>();
                let delta = state.mouse_position - *position;

                shell.publish((self.on_action)(MouseMove(
                    delta,
                    state.mouse_button,
                    state.keyboard_modifiers,
                )));

                shell.capture_event();

                shell.redraw_request();

                state.mouse_position = *position;
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                if cursor.is_over(layout.bounds()) {
                    tree.state.downcast_mut::<State>().mouse_button = Some(*button);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(_)) => {
                tree.state.downcast_mut::<State>().mouse_button = None;
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if cursor.is_over(layout.bounds()) {
                    let delta = match delta {
                        mouse::ScrollDelta::Lines { x: _, y } => y,
                        mouse::ScrollDelta::Pixels { x: _, y } => y,
                    };

                    shell.publish((self.on_action)(ScrollDelta(*delta)));
                    shell.capture_event();

                    shell.redraw_request();
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                if self.state.dirty.is_none() {
                    return;
                }

                let (width, height, pixels) = self.state.renderer.render();
                let cache = Handle::from_rgba(width, height, pixels);

                shell.publish((self.on_action)(Cached(cache, *now)));
            }
            _ => {
                // Not handled event.
            }
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                ..Default::default()
            },
            palette::BACKGROUND_COLOR_DEFAULT,
        );

        let Some(cache) = &self.state.cache else {
            return;
        };

        renderer.draw_image(Image::new(cache.clone()), layout.bounds());
    }
}

impl<'a, Message, Theme, Renderer, A> From<Viewport<'a, Message, Theme, Renderer, A>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: advanced::image::Renderer<Handle = advanced::image::Handle>,
    A: Fn(ViewportAction) -> Message + 'a,
{
    fn from(value: Viewport<'a, Message, Theme, Renderer, A>) -> Self {
        Element::new(value)
    }
}

impl ViewportState {
    /// Constructs a new viewport state with the given config values.
    pub fn new() -> Self {
        Self {
            renderer: PreviewRenderer::new(),
            bounds: Rectangle::INFINITE,
            dirty: Some(Instant::now()),
            cache: None,
        }
    }

    /// Performs the given action on the viewport state.
    pub fn perform(
        &mut self,
        action: ViewportAction,
        far_clip: f32,
        control_scheme: PreviewControlScheme,
    ) {
        use ViewportAction::*;

        match action {
            Resized(bounds) => {
                self.renderer.resize(bounds.width, bounds.height, far_clip);

                self.bounds = bounds;
                self.dirty = Some(Instant::now());
            }
            Cached(handle, now) => {
                self.cache = Some(handle);

                // Only clear the dirty flag if the cached frame is recent enough.
                if self.dirty.is_some_and(|x| now >= x) {
                    self.dirty = None;
                }
            }
            ResetView => {
                self.renderer.reset_view();
                self.dirty = Some(Instant::now());
            }
            ToggleGrid => {
                self.renderer.toggle_grid();
                self.dirty = Some(Instant::now());
            }
            ToggleBones => {
                self.renderer.toggle_bones();
                self.dirty = Some(Instant::now());
            }
            ToggleWireframe => {
                self.renderer.toggle_wireframe();
                self.dirty = Some(Instant::now());
            }
            ToggleShaded => {
                self.renderer.toggle_shaded();
                self.dirty = Some(Instant::now());
            }
            CycleMaterial => {
                self.renderer.cycle_material();
                self.dirty = Some(Instant::now());
            }
            ScrollDelta(delta) => {
                self.renderer.scroll_delta(delta);
                self.dirty = Some(Instant::now());
            }
            MouseMove(delta, mouse_button, keyboard_modifiers) => {
                self.renderer.mouse_move(
                    (delta.x, delta.y),
                    PreviewKeyState {
                        maya: matches!(control_scheme, PreviewControlScheme::Maya),
                        left: matches!(mouse_button, Some(mouse::Button::Left)),
                        right: matches!(mouse_button, Some(mouse::Button::Right)),
                        middle: matches!(mouse_button, Some(mouse::Button::Middle)),
                        alt: keyboard_modifiers.alt() || keyboard_modifiers.command(),
                        shift: keyboard_modifiers.shift(),
                    },
                );
                self.dirty = Some(Instant::now());
            }
        }
    }

    /// Gets a reference to the renderer used by this viewport.
    pub fn renderer(&self) -> &PreviewRenderer {
        &self.renderer
    }

    /// Get a mutable reference to the renderer used by this viewport.
    pub fn renderer_mut(&mut self) -> &mut PreviewRenderer {
        self.dirty = Some(Instant::now());

        &mut self.renderer
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new()
    }
}
