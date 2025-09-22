use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use iced::advanced;
use iced::advanced::Renderer;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse::Cursor;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;

use iced::window;

use iced::widget::canvas;
use iced::widget::canvas::Cache;

use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Radians;
use iced::Rectangle;
use iced::Size;
use iced::Theme;
use iced::Vector;

use lyon_algorithms::measure::PathMeasurements;
use lyon_algorithms::measure::SampleType;
use lyon_algorithms::path::Path;

/// Min angle.
const MIN_RADIANS: f32 = std::f32::consts::PI / 8.0;
/// Max wrap angle.
const WRAP_RADIANS: f32 = 2.0 * std::f32::consts::PI - std::f32::consts::PI / 4.0;
/// Rotation speed.
const BASE_ROTATION_SPEED: u32 = u32::MAX / 80;

/// Cubic bezier easing for spinner.
fn easing() -> &'static Easing {
    static EASING: OnceLock<Easing> = OnceLock::new();

    EASING.get_or_init(|| {
        let mut builder = Path::builder();

        builder.begin(lyon_algorithms::geom::point(0.0, 0.0));

        builder.cubic_bezier_to(
            lyon_algorithms::geom::point(0.2, 0.0),
            lyon_algorithms::geom::point(0.0, 1.0),
            lyon_algorithms::geom::point(1.0, 1.0),
        );

        builder.line_to(lyon_algorithms::geom::point(1.0, 1.0));
        builder.end(false);

        let path = builder.build();
        let measurments = PathMeasurements::from_path(&path, 0.0);

        Easing { path, measurments }
    })
}

/// Easing for spinner animation.
struct Easing {
    path: Path,
    measurments: PathMeasurements,
}

/// Animation state.
#[derive(Clone, Copy)]
enum Animation {
    Expanding {
        start: Instant,
        progress: f32,
        rotation: u32,
        last: Instant,
    },
    Contracting {
        start: Instant,
        progress: f32,
        rotation: u32,
        last: Instant,
    },
}

/// State for the spinner.
struct State {
    animation: Animation,
    cache: Cache,
}

/// A spinning progress indicator widget.
pub struct Spinner<'a> {
    size: f32,
    bar_height: f32,
    style: Style,
    easing: &'a Easing,
    cycle_duration: Duration,
    rotation_duration: Duration,
}

/// Style for the spinner widget.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The track color of the progress indicator.
    pub track_color: Color,
    /// The bar color of the progress indicator.
    pub bar_color: Color,
}

impl Spinner<'_> {
    /// Creates a new [`Spinner`] widget.
    pub fn new() -> Self {
        Self {
            size: 40.0,
            bar_height: 4.0,
            style: Default::default(),
            easing: easing(),
            cycle_duration: Duration::from_millis(600),
            rotation_duration: Duration::from_secs(2),
        }
    }

    /// Sets the size of the [`Spinner`].
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the bar height of the [`Spinner`].
    pub fn bar_height(mut self, bar_height: f32) -> Self {
        self.bar_height = bar_height;
        self
    }

    /// Sets the style of this [`Spinner`].
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the cycle duration of this [`Spinner`].
    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration / 2;
        self
    }

    /// Sets the base rotation duration of this [`Spinner`]. This is the duration that a full
    /// rotation would take if the cycle rotation were set to 0.0 (no expanding or contracting)
    pub fn rotation_duration(mut self, duration: Duration) -> Self {
        self.rotation_duration = duration;
        self
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Spinner<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.size), Length::Fixed(self.size))
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        layout::atomic(limits, self.size, self.size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        _layout: layout::Layout<'_>,
        _cursor: Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.step_animation(self.cycle_duration, self.rotation_duration, *now);
            state.cache.clear();

            shell.request_redraw_at(*now + Duration::from_millis(16));
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width() / 2.0 - self.bar_height;
            let track_path = canvas::Path::circle(frame.center(), track_radius);

            frame.stroke(
                &track_path,
                canvas::Stroke::default()
                    .with_color(self.style.track_color)
                    .with_width(self.bar_height),
            );

            let mut builder = canvas::path::Builder::new();

            let start = state.animation.rotation() * 2.0 * std::f32::consts::PI;

            match state.animation {
                Animation::Expanding { progress, .. } => {
                    builder.arc(canvas::path::Arc {
                        center: frame.center(),
                        radius: track_radius,
                        start_angle: Radians(start),
                        end_angle: Radians(
                            start + MIN_RADIANS + WRAP_RADIANS * (self.easing.y_at_x(progress)),
                        ),
                    });
                }
                Animation::Contracting { progress, .. } => {
                    builder.arc(canvas::path::Arc {
                        center: frame.center(),
                        radius: track_radius,
                        start_angle: Radians(start + WRAP_RADIANS * (self.easing.y_at_x(progress))),
                        end_angle: Radians(start + MIN_RADIANS + WRAP_RADIANS),
                    });
                }
            }

            let bar_path = builder.build();

            frame.stroke(
                &bar_path,
                canvas::Stroke::default()
                    .with_color(self.style.bar_color)
                    .with_width(self.bar_height),
            );
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            use iced::advanced::graphics::geometry::Renderer as _;

            renderer.draw_geometry(geometry);
        });
    }
}

impl<'a, Message> From<Spinner<'a>> for Element<'a, Message, Theme, iced::Renderer> {
    fn from(value: Spinner<'a>) -> Self {
        Self::new(value)
    }
}

impl Easing {
    /// Computes the y value for the given x distance on the curve.
    pub fn y_at_x(&self, x: f32) -> f32 {
        self.measurments
            .create_sampler(&self.path, SampleType::Normalized)
            .sample(x)
            .position()
            .y
    }
}

impl Animation {
    /// Gets the start timestamp.
    fn start(&self) -> Instant {
        match self {
            Self::Expanding { start, .. } | Self::Contracting { start, .. } => *start,
        }
    }

    /// Gets the last updated timestamp.
    fn last(&self) -> Instant {
        match self {
            Self::Expanding { last, .. } | Self::Contracting { last, .. } => *last,
        }
    }

    /// Steps the animation based on the delta time.
    fn step(&self, cycle_duration: Duration, rotation_duration: Duration, now: Instant) -> Self {
        let elapsed = now.duration_since(self.start());
        let additional_rotation = ((now - self.last()).as_secs_f32()
            / rotation_duration.as_secs_f32()
            * (u32::MAX) as f32) as u32;

        match elapsed {
            elapsed if elapsed > cycle_duration => self.next(additional_rotation, now),
            _ => self.with_elapsed(cycle_duration, additional_rotation, elapsed, now),
        }
    }

    /// Computes the next cycle step.
    fn next(&self, additional_rotation: u32, now: Instant) -> Self {
        match self {
            Self::Expanding { rotation, .. } => Self::Contracting {
                start: now,
                progress: 0.0,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
            Self::Contracting { rotation, .. } => Self::Expanding {
                start: now,
                progress: 0.0,
                rotation: rotation.wrapping_add(BASE_ROTATION_SPEED.wrapping_add(
                    ((WRAP_RADIANS / (2.0 * std::f32::consts::PI)) * u32::MAX as f32) as u32,
                )),
                last: now,
            },
        }
    }

    /// Computes the elapsed step.
    fn with_elapsed(
        &self,
        cycle_duration: Duration,
        additional_rotation: u32,
        elapsed: Duration,
        now: Instant,
    ) -> Self {
        let progress = elapsed.as_secs_f32() / cycle_duration.as_secs_f32();

        match self {
            Self::Expanding {
                start, rotation, ..
            } => Self::Expanding {
                start: *start,
                progress,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
            Self::Contracting {
                start, rotation, ..
            } => Self::Contracting {
                start: *start,
                progress,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
        }
    }

    /// Computes the current rotation.
    fn rotation(&self) -> f32 {
        match self {
            Self::Expanding { rotation, .. } | Self::Contracting { rotation, .. } => {
                *rotation as f32 / u32::MAX as f32
            }
        }
    }
}

impl State {
    /// Advances to the next step in the animation.
    pub fn step_animation(
        &mut self,
        cycle_duration: Duration,
        rotation_duration: Duration,
        now: Instant,
    ) {
        self.animation = self.animation.step(cycle_duration, rotation_duration, now);
    }
}

impl Default for Spinner<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self::Expanding {
            start: Instant::now(),
            progress: 0.0,
            rotation: 0,
            last: Instant::now(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            animation: Default::default(),
            cache: Cache::new(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            track_color: Color::TRANSPARENT,
            bar_color: Color::BLACK,
        }
    }
}
