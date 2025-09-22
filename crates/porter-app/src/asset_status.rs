use std::fmt;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use iced::Color;

use crate::palette;

/// The status of an asset.
#[repr(transparent)]
#[derive(Debug)]
pub struct AssetStatus {
    status: AtomicUsize,
}

impl AssetStatus {
    /// The asset was loaded.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const LOADED: Self = Self {
        status: AtomicUsize::new(0),
    };

    /// The asset was exported.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const EXPORTED: Self = Self {
        status: AtomicUsize::new(1),
    };

    /// The asset had an error.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const ERROR: Self = Self {
        status: AtomicUsize::new(2),
    };

    /// The asset is a placeholder.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const PLACEHOLDER: Self = Self {
        status: AtomicUsize::new(3),
    };

    /// The asset is exporting.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const EXPORTING: Self = Self {
        status: AtomicUsize::new(4),
    };

    /// The asset is not supported yet.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const NOT_SUPPORTED: Self = Self {
        status: AtomicUsize::new(5),
    };

    /// Returns true if the status matches.
    #[inline]
    pub fn is(&self, status: Self) -> bool {
        self.status.load(Ordering::Relaxed) == status.status.load(Ordering::Relaxed)
    }

    /// Returns true if the asset is available for export/preview.
    #[inline]
    pub fn is_available(&self) -> bool {
        !self.is(Self::PLACEHOLDER) && !self.is(Self::NOT_SUPPORTED)
    }

    /// Sets the asset status.
    #[inline]
    pub fn set(&self, status: Self) {
        self.status
            .store(status.status.load(Ordering::Relaxed), Ordering::Relaxed)
    }

    /// Gets the color for this status.
    pub fn color(&self) -> Color {
        match self.status.load(Ordering::Relaxed) {
            0 => palette::ASSET_STATUS_LOADED,
            1 => palette::ASSET_STATUS_EXPORTED,
            2 => palette::ASSET_STATUS_ERROR,
            3 => palette::ASSET_STATUS_PLACEHOLDER,
            4 => palette::ASSET_STATUS_EXPORTING,
            5 => palette::ASSET_STATUS_NOT_SUPPORTED,
            _ => Color::WHITE,
        }
    }
}

impl fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status.load(Ordering::Relaxed) {
            0 => write!(f, "Loaded"),
            1 => write!(f, "Exported"),
            2 => write!(f, "Error"),
            3 => write!(f, "Placeholder"),
            4 => write!(f, "Exporting..."),
            5 => write!(f, "Not Supported"),
            _ => write!(f, "<unset>"),
        }
    }
}
