use std::time::Duration;
use std::time::Instant;

use iced::widget::canvas::Frame;
use iced::widget::canvas::Geometry;
use iced::widget::canvas::Path;
use iced::widget::canvas::Program;
use iced::widget::canvas::Stroke;

use iced::widget::Action;

use iced::advanced::mouse::Cursor;

use iced::Color;
use iced::Event;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;

use porter_math::Angles;
use porter_math::Quaternion;
use porter_math::Vector3;

use crate::Message;
use crate::SplashMessage;

/// The size of the cube in pixels.
const CUBE_SIZE: f32 = 50.0;
/// The framerate to redraw the cube.
const CUBE_FRAMERATE: u64 = 60;

/// Cube face color (0|2).
const CUBE_FACES_02: Color = Color::from_rgb8(0x07, 0x7B, 0xB4);
/// Cube face color (1|3).
const CUBE_FACES_13: Color = Color::from_rgb8(0x37, 0xAB, 0xE4);
/// Cube face color (4|5).
const CUBE_FACES_45: Color = Color::from_rgb8(0x27, 0x9B, 0xD4);

/// Color for half strokes.
const STROKE_LINE_COLOR: Color = Color::from_rgba(0.2, 0.2, 0.2, 0.2);
/// Color for full strokes.
const STROKE_FULL_LINE_COLOR: Color = Color::from_rgba(0.2, 0.2, 0.2, 0.5);

/// How long before the splash closes.
const SPLASH_DURATION: Duration = if cfg!(debug_assertions) {
    Duration::from_millis(750)
} else {
    Duration::from_millis(3072)
};

/// Splash screen right side renderer.
pub struct Splash;

/// Splash screen state.
pub struct SplashState {
    started: Instant,
    last: Instant,
}

impl Program<Message> for Splash {
    type State = SplashState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<Action<Message>> {
        use iced::window;

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if (*now - state.started) >= SPLASH_DURATION {
                return Some(Action::publish(Message::from(SplashMessage::Close)));
            }

            state.last = *now;

            Some(Action::request_redraw_at(
                *now + Duration::from_millis(1000 / CUBE_FRAMERATE),
            ))
        } else {
            None
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let time = state.last - state.started;
        let delta = (time.as_millis() / (1000 / CUBE_FRAMERATE) as u128) as f32 * 0.96;

        let mut frame = Frame::new(renderer, bounds.size());

        let mut offset_x = 0.0;
        let mut offset_y = 0.0;

        let Size { width, height } = frame.size();

        let stroke = Stroke::default()
            .with_color(STROKE_LINE_COLOR)
            .with_width(1.0);

        let stroke_full = Stroke::default()
            .with_color(STROKE_FULL_LINE_COLOR)
            .with_width(1.0);

        while offset_x <= width || offset_y <= height {
            let stroke = if (((offset_x / 25.0) as u32) % 4) == 3 {
                stroke_full
            } else {
                stroke
            };

            frame.stroke(
                &Path::line(Point::new(offset_x, 0.0), Point::new(offset_x, height)),
                stroke,
            );

            frame.stroke(
                &Path::line(Point::new(0.0, offset_y), Point::new(width, offset_y)),
                stroke,
            );

            offset_x += 25.0;
            offset_y += 25.0;
        }

        let center = bounds.center();

        let rotation_x =
            Quaternion::from_axis_rotation(Vector3::new(1.0, 0.0, 0.0), delta, Angles::Degrees)
                .to_4x4();

        let rotation_z =
            Quaternion::from_axis_rotation(Vector3::new(0.0, 0.0, 1.0), delta, Angles::Degrees)
                .to_4x4();

        let mut vertices = [
            Vector3::new(-CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE),
            Vector3::new(CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE),
            Vector3::new(CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE),
            Vector3::new(-CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE),
            Vector3::new(-CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE),
            Vector3::new(CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE),
            Vector3::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
            Vector3::new(-CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
        ];

        for vertex in &mut vertices {
            *vertex = vertex.transform(&rotation_z);
            *vertex = vertex.transform(&rotation_x);
            *vertex += Vector3::new(center.x / 2.0, center.y, 0.0);
        }

        let mut faces = [
            ([vertices[0], vertices[1], vertices[5], vertices[4]], 0),
            ([vertices[1], vertices[2], vertices[6], vertices[5]], 1),
            ([vertices[2], vertices[3], vertices[7], vertices[6]], 2),
            ([vertices[3], vertices[0], vertices[4], vertices[7]], 3),
            ([vertices[0], vertices[1], vertices[2], vertices[3]], 4),
            ([vertices[4], vertices[5], vertices[6], vertices[7]], 5),
        ];

        faces.sort_by(|x, y| {
            let x_depth = x.0.iter().map(|x| x.z).sum::<f32>() / 4.0;
            let y_depth = y.0.iter().map(|x| x.z).sum::<f32>() / 4.0;

            x_depth.total_cmp(&y_depth)
        });

        for (vertices, i) in faces {
            let path = Path::new(|b| {
                for vertex in vertices {
                    b.line_to(Point::new(vertex.x, vertex.y));
                }
                b.close();
            });

            let color = match i {
                0 | 2 => CUBE_FACES_02,
                1 | 3 => CUBE_FACES_13,
                _ => CUBE_FACES_45,
            };

            frame.fill(&path, color);
        }

        vec![frame.into_geometry()]
    }
}

impl Default for SplashState {
    fn default() -> Self {
        let now = Instant::now();

        Self {
            started: now,
            last: now,
        }
    }
}
