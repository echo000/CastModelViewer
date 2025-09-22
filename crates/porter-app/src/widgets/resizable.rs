use iced::advanced;
use iced::advanced::Layout;
use iced::advanced::Widget;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree::State;
use iced::advanced::widget::tree::Tag;

use iced::Element;
use iced::Length;
use iced::Rectangle;
use iced::Size;
use iced::Vector;

/// A widget that fires resize events.
pub struct Resizable<'a, Message, Theme, Renderer, R> {
    content: Element<'a, Message, Theme, Renderer>,
    on_resize: R,
}

impl<'a, Message, Theme, Renderer, R> Resizable<'a, Message, Theme, Renderer, R>
where
    R: Fn(Rectangle<f32>) -> Message,
{
    /// Creates a [`Resizable`] with the given content, and resize message.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>, on_resize: R) -> Self {
        Self {
            content: content.into(),
            on_resize,
        }
    }
}

impl<Message, Theme, Renderer, R> Widget<Message, Theme, Renderer>
    for Resizable<'_, Message, Theme, Renderer, R>
where
    Renderer: advanced::Renderer,
    R: Fn(Rectangle<f32>) -> Message,
{
    fn tag(&self) -> Tag {
        Tag::of::<Rectangle<f32>>()
    }

    fn state(&self) -> State {
        State::new(Rectangle::<f32>::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        self.content
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let new_bounds = layout.bounds();
        let old_bounds: &mut Rectangle<f32> = tree.state.downcast_mut();

        if new_bounds != *old_bounds {
            *old_bounds = new_bounds;
            shell.publish((self.on_resize)(new_bounds));
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, translation)
    }
}

impl<'a, Message, Theme, Renderer, R> From<Resizable<'a, Message, Theme, Renderer, R>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + 'a,
    R: Fn(Rectangle<f32>) -> Message + 'a,
{
    fn from(value: Resizable<'a, Message, Theme, Renderer, R>) -> Self {
        Element::new(value)
    }
}
