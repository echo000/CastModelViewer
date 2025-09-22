use iced::Color;

use crate::Sort;

/// A column in the virtual list view of assets.
#[derive(Debug, Clone, Copy)]
pub struct AssetColumn {
    pub header: &'static str,
    pub width: f32,
    pub color: Option<Color>,
    pub sort: Option<Sort>,
}

impl AssetColumn {
    /// Constructs a new asset column.
    pub const fn new(
        header: &'static str,
        width: usize,
        color: Option<Color>,
        sort: Option<Sort>,
    ) -> Self {
        Self {
            header,
            width: width as f32,
            color,
            sort,
        }
    }
}
