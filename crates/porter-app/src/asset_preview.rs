use porter_model::MaterialTextureRefUsage;
use porter_model::Model;

use porter_texture::Image;

/// The result of an assets data to be previewed.
#[derive(Debug, Clone)]
pub enum AssetPreview {
    /// This asset type doesn't support preview.
    NotSupported,
    /// An error occured while previewing this asset.
    PreviewError,
    /// A raw file asset for preview.
    RawFile(String, Vec<u8>),
    /// An image asset for preview.
    Image(String, Image),
    /// A model asset for preview.
    Model(String, Model, Vec<Option<Image>>),
    /// A material asset for preview.
    Material(String, Vec<(MaterialTextureRefUsage, Image)>),
}
