use iced::Color;

/// Blends a color channel to the target with the given factor.
const fn blend_channel(value: f32, target: f32, factor: f32) -> f32 {
    value * (1.0 - factor) + target * factor
}

/// Lightens a color by the given factor.
pub const fn lighten(color: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);

    Color::from_rgba(
        blend_channel(color.r, 1.0, factor),
        blend_channel(color.g, 1.0, factor),
        blend_channel(color.b, 1.0, factor),
        color.a,
    )
}

/// Darkens a color by the given factor.
pub const fn darken(color: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);

    Color::from_rgba(
        blend_channel(color.r, 0.0, factor),
        blend_channel(color.g, 0.0, factor),
        blend_channel(color.b, 0.0, factor),
        color.a,
    )
}

/// Primary color.
pub const PRIMARY_COLOR: Color = Color::from_rgb8(0x27, 0x9B, 0xD4);
/// Light 0.25 primary color.
pub const PRIMARY_COLOR_LIGHT_250: Color = lighten(PRIMARY_COLOR, 0.25);
/// Dark 0.25 primary color.
pub const PRIMARY_COLOR_DARK_250: Color = darken(PRIMARY_COLOR, 0.25);

/// Default background color.
pub const BACKGROUND_COLOR_DEFAULT: Color = Color::from_rgb8(0x11, 0x11, 0x11);
/// Light 0.025 background color.
pub const BACKGROUND_COLOR_LIGHT_025: Color = lighten(BACKGROUND_COLOR_DEFAULT, 0.025);
/// Light 0.050 background color.
pub const BACKGROUND_COLOR_LIGHT_050: Color = lighten(BACKGROUND_COLOR_DEFAULT, 0.050);
/// Light 0.100 background color.
pub const BACKGROUND_COLOR_LIGHT_100: Color = lighten(BACKGROUND_COLOR_DEFAULT, 0.100);
/// Light 0.150 background color.
pub const BACKGROUND_COLOR_LIGHT_150: Color = lighten(BACKGROUND_COLOR_DEFAULT, 0.150);

/// Default text color.
pub const TEXT_COLOR_DEFAULT: Color = Color::from_rgb8(0xFF, 0xFF, 0xFF);
/// Secondary text color.
pub const TEXT_COLOR_SECONDARY: Color = darken(TEXT_COLOR_DEFAULT, 0.25);
/// Muted text color.
pub const TEXT_COLOR_MUTED: Color = darken(TEXT_COLOR_DEFAULT, 0.5);
/// Porter text color.
pub const TEXT_COLOR_PORTER: Color = Color::from_rgb8(0xEC, 0x34, 0xCA);
/// Info text color.
pub const TEXT_COLOR_INFO: Color = PRIMARY_COLOR;
/// Warn text color.
pub const TEXT_COLOR_WARN: Color = Color::from_rgb8(0xD4, 0xAF, 0x37);
/// Success text color.
pub const TEXT_COLOR_SUCCESS: Color = Color::from_rgb8(0x23, 0xCE, 0x6B);
/// Link text color.
pub const TEXT_COLOR_LINK: Color = TEXT_COLOR_INFO;
/// Link hover text color.
pub const TEXT_COLOR_LINK_HOVER: Color = lighten(TEXT_COLOR_LINK, 0.3);
/// Disabled text color.
pub const TEXT_COLOR_DISABLED: Color = Color::from_rgb8(0x2C, 0x2C, 0x2C);

/// Asset status loaded color.
pub const ASSET_STATUS_LOADED: Color = Color::from_rgb8(0x23, 0xCE, 0x6B);
/// Asset status exported color.
pub const ASSET_STATUS_EXPORTED: Color = Color::from_rgb8(0x21, 0xB8, 0xEB);
/// Asset status error color.
pub const ASSET_STATUS_ERROR: Color = Color::from_rgb8(0xD4, 0xAF, 0x37);
/// Asset status placeholder color.
pub const ASSET_STATUS_PLACEHOLDER: Color = Color::from_rgb8(0xEC, 0x34, 0xCA);
/// Asset status exporting color.
pub const ASSET_STATUS_EXPORTING: Color = Color::from_rgb8(0x90, 0x7A, 0xD6);
/// Asset status not supported color.
pub const ASSET_STATUS_NOT_SUPPORTED: Color = Color::from_rgb8(0xF1, 0xA3, 0x8B);

/// Asset type image color.
pub const ASSET_TYPE_IMAGE: Color = Color::from_rgb8(0xCA, 0x61, 0xC3);
/// Asset type model color.
pub const ASSET_TYPE_MODEL: Color = Color::from_rgb8(0x00, 0x9D, 0xDC);
/// Asset type material color.
pub const ASSET_TYPE_MATERIAL: Color = Color::from_rgb8(0x1B, 0x99, 0x8B);
/// Asset type animation color.
pub const ASSET_TYPE_ANIMATION: Color = Color::from_rgb8(0xDB, 0x50, 0x4A);
/// Asset type sound color.
pub const ASSET_TYPE_SOUND: Color = Color::from_rgb8(0xD8, 0x1E, 0x5B);
/// Asset type raw file color.
pub const ASSET_TYPE_RAW_FILE: Color = Color::from_rgb8(0xFF, 0xFF, 0x00);
/// Asset type world color.
pub const ASSET_TYPE_WORLD: Color = Color::from_rgb8(0x7D, 0x5C, 0xFF);
