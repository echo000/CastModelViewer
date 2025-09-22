use std::path::PathBuf;

use iced::window;

use iced::Event;

use crate::AssetPreview;
use crate::Controller;
use crate::MainMessage;
use crate::PreviewWindowMessage;
use crate::SplashMessage;
use crate::components::ContentMessage;
use crate::components::ControlsMessage;
use crate::components::HeaderMessage;
use crate::components::PreviewMessage;
use crate::components::SearchBarMessage;
use crate::components::SettingsMessage;
use crate::components::VirtualListMessage;

#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    UI(Event, window::Id),
    WindowOpened(window::Id),
    Controller(Controller),
    Splash(SplashMessage),
    Main(MainMessage),
    PreviewProxy(PreviewMessage),
    PreviewWindow(PreviewWindowMessage),
    LoadUpdate(Result<(), String>),
    ProgressUpdate(bool, u32),
    PreviewUpdate(u64, AssetPreview),
    PreviewWindowCreate,
    PreviewWindowClosed,
    PreviewToggle,
    PreviewRequest,
    ExportSelected,
    ExportAll,
    ExportCancel,
    LoadFiles(Vec<PathBuf>),
    LoadFilesDropped,
    LoadGame,
    Sort(Option<usize>),
    CheckReload,
}

impl From<SplashMessage> for Message {
    fn from(value: SplashMessage) -> Self {
        Self::Splash(value)
    }
}

impl From<MainMessage> for Message {
    fn from(value: MainMessage) -> Self {
        Self::Main(value)
    }
}

impl From<HeaderMessage> for Message {
    fn from(value: HeaderMessage) -> Self {
        Self::from(MainMessage::Header(value))
    }
}

impl From<SearchBarMessage> for Message {
    fn from(value: SearchBarMessage) -> Self {
        Self::from(MainMessage::SearchBar(value))
    }
}

impl From<ContentMessage> for Message {
    fn from(value: ContentMessage) -> Self {
        Self::from(MainMessage::Content(value))
    }
}

impl From<PreviewMessage> for Message {
    fn from(value: PreviewMessage) -> Self {
        Self::PreviewProxy(value)
    }
}

impl From<VirtualListMessage> for Message {
    fn from(value: VirtualListMessage) -> Self {
        Self::from(MainMessage::Content(ContentMessage::VirtualList(value)))
    }
}

impl From<ControlsMessage> for Message {
    fn from(value: ControlsMessage) -> Self {
        Self::from(MainMessage::Controls(value))
    }
}

impl From<SettingsMessage> for Message {
    fn from(value: SettingsMessage) -> Self {
        Self::from(MainMessage::Settings(value))
    }
}
