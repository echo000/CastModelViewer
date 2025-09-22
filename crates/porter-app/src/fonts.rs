use iced::font::Family;
use iced::font::Stretch;
use iced::font::Style;
use iced::font::Weight;

use iced::Font;

/// Font used for titles.
pub const TITLE_FONT: Font = Font {
    family: Family::SansSerif,
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Font used for icons.
pub const ICON_FONT: Font = Font {
    family: Family::Name("porter"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Font used for monospace.
pub const MONOSPACE_FONT: Font = Font {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    family: Family::Name("Courier New"),
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    family: Family::Monospace,
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Font used for bold monospace.
pub const MONOSPACE_BOLD_FONT: Font = Font {
    weight: Weight::Bold,
    ..MONOSPACE_FONT
};

/// Font used for binary viewer.
pub const BINARY_FONT: Font = Font {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    family: Family::Name("Courier New"),
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    family: Family::Monospace,
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};
