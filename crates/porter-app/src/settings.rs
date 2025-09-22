use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use bincode::Decode;
use bincode::Encode;

use directories::ProjectDirs;
use directories::UserDirs;

use bitflags::bitflags;

use porter_animation::AnimationFileType;
use porter_audio::AudioFileType;
use porter_model::ModelFileType;
use porter_texture::ImageFileType;

#[derive(Debug, Decode, Encode, Clone, Copy)]
struct LoadSettings(u32);

#[derive(Debug, Decode, Encode, Clone, Copy)]
struct ModelSettings(u32);

#[derive(Debug, Decode, Encode, Clone, Copy)]
struct AnimSettings(u32);

#[derive(Debug, Decode, Encode, Clone, Copy)]
struct AudioSettings(u32);

bitflags! {
    impl LoadSettings: u32 {
        const LOAD_MODELS = 1 << 0;
        const LOAD_IMAGES = 1 << 1;
        const LOAD_MATERIALS = 1 << 2;
        const LOAD_ANIMATIONS = 1 << 3;
        const LOAD_SOUNDS = 1 << 4;
        const LOAD_RAW_FILES = 1 << 5;
        const LOAD_FORCE_RAW_FILES = 1 << 6;
    }
}

bitflags! {
    impl ModelSettings: u32 {
        const EXPORT_OBJ = 1 << 0;
        const EXPORT_SMD = 1 << 1;
        const EXPORT_XNA_LARA = 1 << 2;
        const EXPORT_XMODEL_EXPORT = 1 << 3;
        const EXPORT_SEMODEL_REMOVED = 1 << 4;
        const EXPORT_CAST = 1 << 5;
        const EXPORT_MAYA = 1 << 6;
        const EXPORT_FBX = 1 << 7;
    }
}

bitflags! {
    impl AnimSettings: u32 {
        const EXPORT_SEANIM_REMOVED = 1 << 0;
        const EXPORT_CAST = 1 << 1;
    }
}

bitflags! {
    impl AudioSettings: u32 {
        const EXPORT_WAV = 1 << 0;
        const EXPORT_FLAC = 1 << 2;
    }
}

/// Options for processing normal maps through the converter.
#[derive(Debug, Decode, Encode, Clone, Copy)]
pub enum ImageNormalMapProcessing {
    None,
    OpenGl,
    DirectX,
}

/// Control scheme for preview viewport.
#[derive(Debug, Decode, Encode, Clone, Copy)]
pub enum PreviewControlScheme {
    Maya,
    Blender,
}

/// Global application settings.
#[derive(Debug, Decode, Encode, Clone)]
pub struct Settings {
    version: u32,
    load_settings: LoadSettings,
    model_settings: ModelSettings,
    anim_settings: AnimSettings,
    audio_settings: AudioSettings,
    image_file_type: ImageFileType,
    image_normal_map_processing: ImageNormalMapProcessing,
    output_directory: Option<PathBuf>,
    preview_controls: PreviewControlScheme,
    preview_overlay: bool,
    auto_scale: bool,
    far_clip: u32,
    preview_window: bool,
    custom_scale: Option<f32>,
}

impl Settings {
    /// Loads the settings from the disk at the given path, or returns new ones.
    pub fn load<S: Into<String>>(name: S) -> Settings {
        let Some(project_directory) = ProjectDirs::from("com", "DTZxPorter", "GameTools") else {
            return Default::default();
        };

        std::fs::read(
            project_directory
                .config_dir()
                .join(name.into().to_lowercase())
                .with_extension("dat"),
        )
        .map_or(Default::default(), |buffer| {
            let config = bincode::config::standard();

            bincode::decode_from_slice(&buffer, config)
                .unwrap_or_default()
                .0
        })
    }

    /// Saves the settings to the disk at the given path.
    pub fn save<S: Into<String>>(&self, name: S) {
        let Some(project_directory) = ProjectDirs::from("com", "DTZxPorter", "GameTools") else {
            return;
        };

        let config = bincode::config::standard();

        let Ok(result) = bincode::encode_to_vec(self, config) else {
            return;
        };

        let dirs = std::fs::create_dir_all(project_directory.config_dir());

        debug_assert!(dirs.is_ok());

        let result = std::fs::write(
            project_directory
                .config_dir()
                .join(name.into().to_lowercase())
                .with_extension("dat"),
            result,
        );

        debug_assert!(result.is_ok());
    }

    /// Checks whether or not the new settings requires a reload.
    pub fn reload_required(&self, new_settings: &Self) -> bool {
        if self.load_models() != new_settings.load_models()
            || self.load_animations() != new_settings.load_animations()
            || self.load_images() != new_settings.load_images()
            || self.load_materials() != new_settings.load_materials()
            || self.load_sounds() != new_settings.load_sounds()
            || self.load_raw_files() != new_settings.load_raw_files()
            || self.force_raw_files() != new_settings.force_raw_files()
        {
            return true;
        }

        false
    }

    /// Whether or not to load models.
    pub fn load_models(&self) -> bool {
        self.load_settings.contains(LoadSettings::LOAD_MODELS)
    }

    /// Sets whether or not to load models.
    pub fn set_load_models(&mut self, value: bool) {
        self.load_settings.set(LoadSettings::LOAD_MODELS, value);
    }

    /// Whether or not to load images.
    pub fn load_images(&self) -> bool {
        self.load_settings.contains(LoadSettings::LOAD_IMAGES)
    }

    /// Sets whether or not to load images.
    pub fn set_load_images(&mut self, value: bool) {
        self.load_settings.set(LoadSettings::LOAD_IMAGES, value)
    }

    /// Whether or not to load materials.
    pub fn load_materials(&self) -> bool {
        self.load_settings.contains(LoadSettings::LOAD_MATERIALS)
    }

    /// Sets whether or not to load materials.
    pub fn set_load_materials(&mut self, value: bool) {
        self.load_settings.set(LoadSettings::LOAD_MATERIALS, value)
    }

    /// Whether or not to load animations.
    pub fn load_animations(&self) -> bool {
        self.load_settings.contains(LoadSettings::LOAD_ANIMATIONS)
    }

    /// Sets whether or not to load animations.
    pub fn set_load_animations(&mut self, value: bool) {
        self.load_settings.set(LoadSettings::LOAD_ANIMATIONS, value)
    }

    /// Whether or not to load sounds.
    pub fn load_sounds(&self) -> bool {
        self.load_settings.contains(LoadSettings::LOAD_SOUNDS)
    }

    /// Sets whether or not to load sounds.
    pub fn set_load_sounds(&mut self, value: bool) {
        self.load_settings.set(LoadSettings::LOAD_SOUNDS, value)
    }

    /// Whether or not to load raw files.
    pub fn load_raw_files(&self) -> bool {
        self.load_settings.contains(LoadSettings::LOAD_RAW_FILES)
    }

    /// Sets whether or not to load raw files.
    pub fn set_load_raw_files(&mut self, value: bool) {
        self.load_settings.set(LoadSettings::LOAD_RAW_FILES, value)
    }

    /// Whether or not to force all assets as raw files.
    pub fn force_raw_files(&self) -> bool {
        self.load_settings
            .contains(LoadSettings::LOAD_FORCE_RAW_FILES)
    }

    /// Sets whether or not to force all assets as raw files.
    pub fn set_force_raw_files(&mut self, value: bool) {
        self.load_settings
            .set(LoadSettings::LOAD_FORCE_RAW_FILES, value)
    }

    /// The model file types to export to.
    pub fn model_file_types(&self) -> Vec<ModelFileType> {
        let mut result = Vec::with_capacity(8);

        if self.model_settings.contains(ModelSettings::EXPORT_OBJ) {
            result.push(ModelFileType::Obj);
        }

        if self.model_settings.contains(ModelSettings::EXPORT_SMD) {
            result.push(ModelFileType::Smd);
        }

        if self.model_settings.contains(ModelSettings::EXPORT_XNA_LARA) {
            result.push(ModelFileType::XnaLara);
        }

        if self
            .model_settings
            .contains(ModelSettings::EXPORT_XMODEL_EXPORT)
        {
            result.push(ModelFileType::XModelExport);
        }

        if self.model_settings.contains(ModelSettings::EXPORT_CAST) {
            result.push(ModelFileType::Cast);
        }

        if self.model_settings.contains(ModelSettings::EXPORT_MAYA) {
            result.push(ModelFileType::Maya);
        }

        if self.model_settings.contains(ModelSettings::EXPORT_FBX) {
            result.push(ModelFileType::Fbx);
        }

        result
    }

    /// Sets whether or not a model file type is in use.
    pub fn set_model_file_type(&mut self, file_type: ModelFileType, value: bool) {
        let flag = match file_type {
            ModelFileType::Obj => ModelSettings::EXPORT_OBJ,
            ModelFileType::Smd => ModelSettings::EXPORT_SMD,
            ModelFileType::XnaLara => ModelSettings::EXPORT_XNA_LARA,
            ModelFileType::XModelExport => ModelSettings::EXPORT_XMODEL_EXPORT,
            ModelFileType::Cast => ModelSettings::EXPORT_CAST,
            ModelFileType::Maya => ModelSettings::EXPORT_MAYA,
            ModelFileType::Fbx => ModelSettings::EXPORT_FBX,
        };

        self.model_settings.set(flag, value);
    }

    /// The animation file types to export to.
    pub fn anim_file_types(&self) -> Vec<AnimationFileType> {
        let mut result = Vec::with_capacity(1);

        if self.anim_settings.contains(AnimSettings::EXPORT_CAST) {
            result.push(AnimationFileType::Cast);
        }

        result
    }

    /// Sets whether or not an anim file type is in use.
    pub fn set_anim_file_type(&mut self, file_type: AnimationFileType, value: bool) {
        let flag = match file_type {
            AnimationFileType::Cast => AnimSettings::EXPORT_CAST,
        };

        self.anim_settings.set(flag, value);
    }

    /// The audio file types to export to.
    pub fn audio_file_types(&self) -> Vec<AudioFileType> {
        let mut result = Vec::with_capacity(3);

        if self.audio_settings.contains(AudioSettings::EXPORT_WAV) {
            result.push(AudioFileType::Wav);
        }

        if self.audio_settings.contains(AudioSettings::EXPORT_FLAC) {
            result.push(AudioFileType::Flac);
        }

        result
    }

    /// Sets whether or not an audio file type is in use.
    pub fn set_audio_file_type(&mut self, file_type: AudioFileType, value: bool) {
        let flag = match file_type {
            AudioFileType::Wav => AudioSettings::EXPORT_WAV,
            AudioFileType::Flac => AudioSettings::EXPORT_FLAC,
            AudioFileType::Ogg => {
                // We don't support writing these formats.
                return;
            }
        };

        self.audio_settings.set(flag, value);
    }

    /// The image file type to export to.
    pub fn image_file_type(&self) -> ImageFileType {
        self.image_file_type
    }

    /// Sets the image file type to export to.
    pub fn set_image_file_type(&mut self, file_type: ImageFileType) {
        self.image_file_type = file_type;
    }

    /// The image normal map processing technique.
    pub fn image_normal_map_processing(&self) -> ImageNormalMapProcessing {
        self.image_normal_map_processing
    }

    /// Sets the image normal map processing.
    pub fn set_image_normal_map_processing(&mut self, processing: ImageNormalMapProcessing) {
        self.image_normal_map_processing = processing;
    }

    /// An output directory used to save assets.
    pub fn output_directory(&self) -> PathBuf {
        if let Some(output_directory) = self.output_directory.clone() {
            return output_directory;
        }

        // We need to make sure that we combine with the exe path.
        // Otherwise we'll have such a bad time, because people are dumb.
        if cfg!(target_os = "windows") {
            if cfg!(debug_assertions) {
                PathBuf::from("./exported_files")
            } else {
                static CURRENT_PATH: OnceLock<PathBuf> = OnceLock::new();

                CURRENT_PATH
                    .get_or_init(|| std::env::current_exe().unwrap_or_default())
                    .parent()
                    .unwrap_or(Path::new("./"))
                    .join("exported_files")
            }
        } else if let Some(user_dirs) = UserDirs::new() {
            match user_dirs.document_dir() {
                Some(path) => path.join("exported_files"),
                None => PathBuf::from("~/Documents/exported_files"),
            }
        } else {
            PathBuf::from("~/Documents/exported_files")
        }
    }

    /// Sets a new output directory.
    pub fn set_output_directory(&mut self, path: PathBuf) {
        self.output_directory = Some(path);
    }

    /// Gets the preview control scheme.
    pub fn preview_controls(&self) -> PreviewControlScheme {
        self.preview_controls
    }

    /// Sets the preview control scheme.
    pub fn set_preview_controls(&mut self, controls: PreviewControlScheme) {
        self.preview_controls = controls;
    }

    /// Whether or not to show the preview overlay hints.
    pub fn preview_overlay(&self) -> bool {
        self.preview_overlay
    }

    /// Sets whether or not to show the preview overlay.
    pub fn set_preview_overlay(&mut self, value: bool) {
        self.preview_overlay = value;
    }

    /// Whether or not to automatically scale models and animations.
    pub fn auto_scale(&self) -> bool {
        self.auto_scale
    }

    /// Sets whether or not to automatically scale models and animations.
    pub fn set_auto_scale(&mut self, value: bool) {
        self.auto_scale = value;
    }

    /// Gets the far clip distance for preview.
    pub fn far_clip(&self) -> u32 {
        self.far_clip.clamp(10000, 1000000)
    }

    /// Sets the far clip distance for preview.
    pub fn set_far_clip(&mut self, far_clip: u32) {
        self.far_clip = far_clip;
    }

    /// Whether or not preview should open in a new window by default.
    pub fn preview_window(&self) -> bool {
        self.preview_window
    }

    /// Sets whether or not preview should open in a new window by default.
    pub fn set_preview_window(&mut self, value: bool) {
        self.preview_window = value;
    }

    /// Gets the custom auto scale value to use.
    pub fn custom_scale(&self) -> Option<f32> {
        self.custom_scale
    }

    /// Sets the custom auto scale value to use.
    pub fn set_custom_scale(&mut self, scale: Option<f32>) {
        self.custom_scale = scale;
    }

    /// Gets the automatic scale factor to use for assets, with the provided default scale.
    pub fn auto_scale_factor(&self, default: f32) -> Option<f32> {
        self.auto_scale()
            .then(|| self.custom_scale().unwrap_or(default))
    }

    /// Update settings and returns a copy.
    pub fn update<F: FnOnce(&mut Self)>(&self, callback: F) -> Self {
        let mut settings = self.clone();

        callback(&mut settings);

        settings
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            load_settings: LoadSettings::all()
                & !LoadSettings::LOAD_RAW_FILES
                & !LoadSettings::LOAD_FORCE_RAW_FILES,
            model_settings: ModelSettings::EXPORT_CAST,
            anim_settings: AnimSettings::EXPORT_CAST,
            audio_settings: AudioSettings::EXPORT_WAV,
            image_file_type: ImageFileType::Png,
            image_normal_map_processing: ImageNormalMapProcessing::OpenGl,
            output_directory: None,
            preview_controls: PreviewControlScheme::Maya,
            preview_overlay: true,
            auto_scale: true,
            far_clip: 10000,
            preview_window: false,
            custom_scale: None,
        }
    }
}
