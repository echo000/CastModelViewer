use iced::window;
use iced::window::Position;
use iced::window::Settings;
use iced::window::settings::PlatformSpecific;

use iced::widget::canvas;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::vertical_space;

use iced::Alignment;
use iced::Background;
use iced::Border;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Size;
use iced::Task;
use iced::Theme;

use porter_utils::StringCaseExt;

use crate::AppState;
use crate::MainMessage;
use crate::Message;
use crate::components::Splash;
use crate::fonts;
use crate::palette;
use crate::strings;
use crate::system;
use crate::widgets;

/// Splash screen window handler.
pub struct SplashWindow {
    pub id: window::Id,
    pub open: bool,
}

/// Messages produced by the splash window.
#[derive(Debug, Clone)]
pub enum SplashMessage {
    UI(Event),
    Close,
    Website,
}

impl SplashWindow {
    /// Creates a new splash window.
    pub fn create() -> (Self, Task<window::Id>) {
        let (id, task) = window::open(Settings {
            size: Size::new(865.0, 570.0),
            min_size: Some(Size::new(865.0, 570.0)),
            position: Position::Centered,
            decorations: false,
            resizable: false,
            platform_specific: PlatformSpecific {
                #[cfg(target_os = "windows")]
                skip_taskbar: true,
                #[cfg(target_os = "windows")]
                drag_and_drop: false,
                ..Default::default()
            },
            ..Default::default()
        });

        (Self { id, open: true }, task)
    }

    /// Handles the title of the splash screen.
    pub fn title(&self, state: &AppState) -> String {
        format!("{} v{}", state.name.to_titlecase(), state.version)
    }

    /// Handles updates for the splash screen.
    pub fn update(&mut self, message: SplashMessage) -> Task<Message> {
        use SplashMessage::*;

        match message {
            UI(event) => self.on_ui(event),
            Close => self.on_close(),
            Website => self.on_website(),
        }
    }

    /// Handles rendering the splash screen.
    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        use SplashMessage::*;

        let splash = row([
            container(
                column([
                    vertical_space().height(20.0).into(),
                    text(state.name.to_uppercase())
                        .size(32.0)
                        .font(fonts::TITLE_FONT)
                        .into(),
                    text(state.description).into(),
                    vertical_space().height(42.0).into(),
                    text(format!("Version {}", state.version)).into(),
                    row([
                        text("Developed by:").into(),
                        text("DTZxPorter").color(palette::TEXT_COLOR_PORTER).into(),
                    ])
                    .spacing(4.0)
                    .into(),
                    widgets::link(strings::PORTER_SITE_URL)
                        .on_press(Message::from(Website))
                        .into(),
                    container(column([
                        text(strings::PORTER_DISCLAIMER)
                            .size(14.0)
                            .color(palette::TEXT_COLOR_MUTED)
                            .into(),
                        vertical_space().height(10.0).into(),
                        text(strings::PORTER_COPYRIGHT).into(),
                        vertical_space().height(20.0).into(),
                    ]))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_y(Alignment::End)
                    .into(),
                ])
                .padding([0.0, 16.0])
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .style(splash_left_style)
            .into(),
            canvas(Splash)
                .width(Length::FillPortion(2))
                .height(Length::Fill)
                .into(),
        ]);

        container(splash)
            .padding(1.0)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(splash_background_style)
            .into()
    }

    /// Occurs when a ui event has fired.
    fn on_ui(&mut self, event: Event) -> Task<Message> {
        match event {
            Event::Window(window::Event::Opened { .. }) => self.on_opened(),
            Event::Window(window::Event::Closed) => self.on_closed(),
            _ => Task::none(),
        }
    }

    /// Occurs when the window has opened.
    fn on_opened(&mut self) -> Task<Message> {
        Task::done(Message::WindowOpened(self.id))
    }

    /// Occurs when the window has closed.
    fn on_closed(&mut self) -> Task<Message> {
        if self.open {
            iced::exit()
        } else {
            Task::none()
        }
    }

    /// Closes the splash window if it's still open.
    fn on_close(&mut self) -> Task<Message> {
        if self.open {
            self.open = false;

            Task::batch([
                window::close(self.id),
                Task::done(Message::from(MainMessage::Show)),
            ])
        } else {
            Task::none()
        }
    }

    /// Opens the website url.
    fn on_website(&mut self) -> Task<Message> {
        system::open_url(strings::PORTER_SITE_URL);

        Task::none()
    }
}

/// Style for the splash left column background.
fn splash_left_style(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(palette::TEXT_COLOR_DEFAULT),
        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_025)),
        ..Default::default()
    }
}

/// Style for the splash background and borders.
fn splash_background_style(_: &Theme) -> container::Style {
    container::Style {
        text_color: None,
        border: Border {
            color: palette::PRIMARY_COLOR,
            width: 1.0,
            ..Default::default()
        },
        background: Some(Background::Color(palette::BACKGROUND_COLOR_DEFAULT)),
        ..Default::default()
    }
}
