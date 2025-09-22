use iced::widget::Column;
use iced::widget::row;
use iced::widget::text;
use iced::widget::vertical_space;

use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::Task;

use directories::ProjectDirs;

#[cfg(feature = "raw-files-forcible")]
use porter_model::ModelFileType;
#[cfg(feature = "raw-files-forcible")]
use porter_texture::ImageFileType;

use crate::AppState;
use crate::MainMessage;
use crate::Message;
use crate::PreviewControlScheme;
use crate::palette;
use crate::system;
use crate::widgets;

/// Settings component handler.
pub struct Settings {
    custom_scale: Option<String>,
}

/// Messages produced by the settings component.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Save(crate::Settings),
    PickExportFolder,
    OpenExportFolder,
    OpenConfigFolder,
    ApplyCustomScale,
    ScaleInput(String),
}

impl Settings {
    /// Creates a new settings component.
    pub fn new() -> Self {
        Self { custom_scale: None }
    }

    /// Handles updates for the settings component.
    pub fn update(&mut self, state: &mut AppState, message: SettingsMessage) -> Task<Message> {
        use SettingsMessage::*;

        match message {
            Save(settings) => self.on_save(state, settings),
            PickExportFolder => self.on_pick_export_folder(state),
            OpenExportFolder => self.on_open_export_folder(state),
            OpenConfigFolder => self.on_open_config_folder(state),
            ApplyCustomScale => self.on_apply_custom_scale(state),
            ScaleInput(input) => self.on_scale_input(state, input),
        }
    }

    /// Handles rendering the settings component.
    pub fn view<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        #[cfg(feature = "raw-files-forcible")]
        {
            let model_formats = state.settings.model_file_types();
            let model_format_enabled = |format: ModelFileType| model_formats.contains(&format);
        }

        let save_message =
            |settings: crate::Settings| Message::from(SettingsMessage::Save(settings));

        let mut settings: Column<_> = Column::with_capacity(64);

        #[cfg(feature = "raw-files-forcible")]
        {
            settings = settings.extend([
                text("Settings - General")
                    .size(20.0)
                    .color(palette::TEXT_COLOR_DEFAULT)
                    .into(),
                vertical_space().height(2.0).into(),
                text("Choose what asset types to load and display:")
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                vertical_space().height(0.0).into(),
                widgets::checkbox("Load Models", state.settings.load_models())
                    .on_toggle(move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_load_models(value)),
                        )
                    })
                    .into(),
            ]);

            #[cfg(feature = "animations")]
            {
                settings = settings.push(
                    widgets::checkbox("Load Animations", state.settings.load_animations())
                        .on_toggle(move |value| {
                            save_message(
                                state
                                    .settings
                                    .update(|settings| settings.set_load_animations(value)),
                            )
                        }),
                );
            }

            #[cfg(feature = "raw-files-forcible")]
            {
                settings = settings.push(
                    widgets::checkbox("Load Images", state.settings.load_images()).on_toggle(
                        move |value| {
                            save_message(
                                state
                                    .settings
                                    .update(|settings| settings.set_load_images(value)),
                            )
                        },
                    ),
                );
            }

            #[cfg(feature = "materials")]
            {
                settings = settings.push(
                    widgets::checkbox("Load Materials", state.settings.load_materials()).on_toggle(
                        move |value| {
                            save_message(
                                state
                                    .settings
                                    .update(|settings| settings.set_load_materials(value)),
                            )
                        },
                    ),
                );
            }

            #[cfg(feature = "sounds")]
            {
                settings = settings.push(
                    widgets::checkbox("Load Sounds", state.settings.load_sounds()).on_toggle(
                        move |value| {
                            save_message(
                                state
                                    .settings
                                    .update(|settings| settings.set_load_sounds(value)),
                            )
                        },
                    ),
                );
            }

            #[cfg(feature = "raw-files")]
            {
                use iced::widget::tooltip::Position;

                settings = settings.push(widgets::tooltip(
                    widgets::checkbox("Load Raw Files", state.settings.load_raw_files()).on_toggle(
                        move |value| {
                            save_message(
                                state
                                    .settings
                                    .update(|settings| settings.set_load_raw_files(value)),
                            )
                        },
                    ),
                    "Loads all files that are exportable as-is",
                    Position::Right,
                ));
            }

            settings = settings.extend([
                vertical_space().height(2.0).into(),
                text("Customize the exported files directory:")
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                vertical_space().height(0.0).into(),
                row(vec![
                    widgets::text_input(
                        "Exported files directory",
                        state.settings.output_directory().to_string_lossy().as_ref(),
                    )
                    .on_input(|_| Message::Noop)
                    .width(500.0)
                    .into(),
                    widgets::button("Browse")
                        .on_press(Message::from(SettingsMessage::PickExportFolder))
                        .into(),
                    widgets::button("Open")
                        .on_press(Message::from(SettingsMessage::OpenExportFolder))
                        .into(),
                ])
                .spacing(4.0)
                .into(),
                vertical_space().height(2.0).into(),
                text("Choose whether or not to automatically scale assets (Recommended):")
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                vertical_space().height(0.0).into(),
                widgets::checkbox("Automatically scale assets", state.settings.auto_scale())
                    .on_toggle(move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_auto_scale(value)),
                        )
                    })
                    .into(),
                vertical_space().height(2.0).into(),
                text("Set a custom asset scale factor:")
                    .color(if state.settings.auto_scale() {
                        palette::TEXT_COLOR_SECONDARY
                    } else {
                        palette::TEXT_COLOR_DISABLED
                    })
                    .into(),
                vertical_space().height(0.0).into(),
                row([
                    widgets::checkbox("Custom scale:", state.settings.custom_scale().is_some())
                        .on_toggle_maybe(if state.settings.auto_scale() {
                            Some(move |value: bool| {
                                save_message(state.settings.update(|settings| {
                                    settings.set_custom_scale(value.then_some(1.0))
                                }))
                            })
                        } else {
                            None
                        })
                        .into(),
                    row([
                        widgets::text_input(
                            "",
                            &self.custom_scale.clone().unwrap_or_else(|| {
                                state
                                    .settings
                                    .custom_scale()
                                    .map(format_custom_scale)
                                    .unwrap_or_else(|| String::from("1.0"))
                            }),
                        )
                        .on_input_maybe(
                            if state.settings.auto_scale()
                                && state.settings.custom_scale().is_some()
                            {
                                Some(|input| Message::from(SettingsMessage::ScaleInput(input)))
                            } else {
                                None
                            },
                        )
                        .width(Length::Fixed(120.0))
                        .into(),
                        widgets::button("Apply")
                            .on_press_maybe(
                                if state.settings.auto_scale()
                                    && state.settings.custom_scale().is_some()
                                {
                                    Some(Message::from(SettingsMessage::ApplyCustomScale))
                                } else {
                                    None
                                },
                            )
                            .into(),
                    ])
                    .spacing(4.0)
                    .align_y(Alignment::Center)
                    .into(),
                ])
                .spacing(8.0)
                .align_y(Alignment::Center)
                .into(),
                vertical_space().height(4.0).into(),
                text("Settings - Models")
                    .size(20.0)
                    .color(palette::TEXT_COLOR_DEFAULT)
                    .into(),
                vertical_space().height(2.0).into(),
                text("Choose what model file types to export to:")
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                vertical_space().height(0.0).into(),
                widgets::checkbox("Cast", model_format_enabled(ModelFileType::Cast))
                    .on_toggle(move |value| {
                        save_message(state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Cast, value)
                        }))
                    })
                    .into(),
                widgets::checkbox("OBJ", model_format_enabled(ModelFileType::Obj))
                    .on_toggle(move |value| {
                        save_message(state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Obj, value)
                        }))
                    })
                    .into(),
                widgets::checkbox("Valve SMD", model_format_enabled(ModelFileType::Smd))
                    .on_toggle(move |value| {
                        save_message(state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Smd, value)
                        }))
                    })
                    .into(),
                widgets::checkbox("XNALara", model_format_enabled(ModelFileType::XnaLara))
                    .on_toggle(move |value| {
                        save_message(state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::XnaLara, value)
                        }))
                    })
                    .into(),
                widgets::checkbox(
                    "CoD XModel",
                    model_format_enabled(ModelFileType::XModelExport),
                )
                .on_toggle(move |value| {
                    save_message(state.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::XModelExport, value)
                    }))
                })
                .into(),
                widgets::checkbox("Autodesk Maya", model_format_enabled(ModelFileType::Maya))
                    .on_toggle(move |value| {
                        save_message(state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Maya, value)
                        }))
                    })
                    .into(),
                widgets::checkbox("FBX", model_format_enabled(ModelFileType::Fbx))
                    .on_toggle(move |value| {
                        save_message(state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Fbx, value)
                        }))
                    })
                    .into(),
                vertical_space().height(4.0).into(),
                text("Settings - Images")
                    .size(20.0)
                    .color(palette::TEXT_COLOR_DEFAULT)
                    .into(),
                vertical_space().height(2.0).into(),
                text("Choose what image file type to export to:")
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                vertical_space().height(0.0).into(),
                widgets::pick_list(
                    vec!["DDS", "PNG", "TIFF", "TGA"],
                    match state.settings.image_file_type() {
                        ImageFileType::Dds => Some("DDS"),
                        ImageFileType::Png => Some("PNG"),
                        ImageFileType::Tiff => Some("TIFF"),
                        ImageFileType::Tga => Some("TGA"),
                    },
                    move |selected| {
                        let format = match selected {
                            "DDS" => ImageFileType::Dds,
                            "PNG" => ImageFileType::Png,
                            "TIFF" => ImageFileType::Tiff,
                            "TGA" => ImageFileType::Tga,
                            _ => ImageFileType::Dds,
                        };

                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_image_file_type(format)),
                        )
                    },
                )
                .width(Length::Fixed(150.0))
                .into(),
                vertical_space().height(2.0).into(),
            ]);

            match state.settings.image_file_type() {
                ImageFileType::Tga => {
                    settings = settings.push(
                        text("(The selected image format may be lossy or take up more space than necessary)")
                            .color(palette::TEXT_COLOR_WARN),
                    );
                }
                ImageFileType::Dds => {
                    settings = settings.push(
                        text("(The selected image format is lossless but may have compatibility issues with some software)")
                            .color(palette::TEXT_COLOR_SUCCESS),
                    );
                }
                _ => {
                    settings = settings.push(
                        text("(The selected image format is lossless and recommended for export)")
                            .color(palette::TEXT_COLOR_SUCCESS),
                    );
                }
            }

            #[cfg(feature = "normal-maps-convertible")]
            {
                use crate::ImageNormalMapProcessing;

                settings = settings.extend([
                    vertical_space().height(2.0).into(),
                    text("Choose a normal map conversion method:")
                        .color(palette::TEXT_COLOR_SECONDARY)
                        .into(),
                    vertical_space().height(0.0).into(),
                    widgets::pick_list(
                        vec!["None", "OpenGL", "DirectX"],
                        match state.settings.image_normal_map_processing() {
                            ImageNormalMapProcessing::None => Some("None"),
                            ImageNormalMapProcessing::OpenGl => Some("OpenGL"),
                            ImageNormalMapProcessing::DirectX => Some("DirectX"),
                        },
                        move |selected| {
                            let format = match selected {
                                "None" => ImageNormalMapProcessing::None,
                                "OpenGL" => ImageNormalMapProcessing::OpenGl,
                                "DirectX" => ImageNormalMapProcessing::DirectX,
                                _ => ImageNormalMapProcessing::None,
                            };

                            save_message(state.settings.update(|settings| {
                                settings.set_image_normal_map_processing(format)
                            }))
                        },
                    )
                    .width(Length::Fixed(150.0))
                    .into(),
                    vertical_space().height(4.0).into(),
                ]);
            }

            #[cfg(not(feature = "normal-maps-convertible"))]
            {
                settings = settings.push(vertical_space().height(4.0));
            }

            #[cfg(feature = "animations")]
            {
                use porter_animation::AnimationFileType;

                let anim_formats = state.settings.anim_file_types();
                let anim_format_enabled =
                    |format: AnimationFileType| anim_formats.contains(&format);

                settings = settings.extend([
                    text("Settings - Animations")
                        .size(20.0)
                        .color(palette::TEXT_COLOR_DEFAULT)
                        .into(),
                    vertical_space().height(2.0).into(),
                    text("Choose what animation file types to export to:")
                        .color(palette::TEXT_COLOR_SECONDARY)
                        .into(),
                    vertical_space().height(0.0).into(),
                    widgets::checkbox("Cast", anim_format_enabled(AnimationFileType::Cast))
                        .on_toggle(move |value| {
                            save_message(state.settings.update(|settings| {
                                settings.set_anim_file_type(AnimationFileType::Cast, value)
                            }))
                        })
                        .into(),
                    vertical_space().height(4.0).into(),
                ]);
            }

            #[cfg(all(feature = "sounds", feature = "sounds-convertible"))]
            {
                use porter_audio::AudioFileType;

                let audio_formats = state.settings.audio_file_types();
                let audio_format_enabled = |format: AudioFileType| audio_formats.contains(&format);

                settings = settings.extend([
                    text("Settings - Audio")
                        .size(20.0)
                        .color(palette::TEXT_COLOR_DEFAULT)
                        .into(),
                    vertical_space().height(2.0).into(),
                    text("Choose what audio file types to export to:")
                        .color(palette::TEXT_COLOR_SECONDARY)
                        .into(),
                    vertical_space().height(0.0).into(),
                    widgets::checkbox("Wav", audio_format_enabled(AudioFileType::Wav))
                        .on_toggle(move |value| {
                            save_message(state.settings.update(|settings| {
                                settings.set_audio_file_type(AudioFileType::Wav, value)
                            }))
                        })
                        .into(),
                    widgets::checkbox("Flac", audio_format_enabled(AudioFileType::Flac))
                        .on_toggle(move |value| {
                            save_message(state.settings.update(|settings| {
                                settings.set_audio_file_type(AudioFileType::Flac, value)
                            }))
                        })
                        .into(),
                    vertical_space().height(4.0).into(),
                ]);
            }
        }

        settings = settings.extend([
            text("Settings - Preview")
                .size(20.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            vertical_space().height(2.0).into(),
            text("Change the preview control scheme:")
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            vertical_space().height(0.0).into(),
            widgets::pick_list(
                vec!["Autodesk Maya", "Blender"],
                match state.settings.preview_controls() {
                    PreviewControlScheme::Maya => Some("Autodesk Maya"),
                    PreviewControlScheme::Blender => Some("Blender"),
                },
                move |selected| {
                    let controls = match selected {
                        "Autodesk Maya" => PreviewControlScheme::Maya,
                        "Blender" => PreviewControlScheme::Blender,
                        _ => PreviewControlScheme::Maya,
                    };

                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_preview_controls(controls)),
                    )
                },
            )
            .width(Length::Fixed(150.0))
            .into(),
            vertical_space().height(2.0).into(),
            text("Choose whether or not to open preview in a separate window:")
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            vertical_space().height(0.0).into(),
            widgets::checkbox("Separate window", state.settings.preview_window())
                .on_toggle(move |value| {
                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_preview_window(value)),
                    )
                })
                .into(),
            vertical_space().height(2.0).into(),
            text("Choose whether or not to show the preview controls overlay:")
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            vertical_space().height(0.0).into(),
            widgets::checkbox("Show controls overlay", state.settings.preview_overlay())
                .on_toggle(move |value| {
                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_preview_overlay(value)),
                    )
                })
                .into(),
            vertical_space().height(2.0).into(),
            text("Set the preview far clip distance (May impact performance):")
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            vertical_space().height(0.0).into(),
            row([
                widgets::slider(10000..=1000000, state.settings.far_clip(), move |value| {
                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_far_clip(value)),
                    )
                })
                .step(10000u32)
                .into(),
                text(state.settings.far_clip().to_string())
                    .width(100.0)
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
            ])
            .width(500.0)
            .spacing(8.0)
            .into(),
            vertical_space().height(4.0).into(),
            text("Settings - Advanced")
                .size(20.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
        ]);

        #[cfg(feature = "raw-files-forcible")]
        {
            use iced::widget::tooltip::Position;

            settings = settings.extend([
                vertical_space().height(2.0).into(),
                text("Choose whether or not to treat all assets as raw files (Not recommended):")
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                vertical_space().height(0.0).into(),
                widgets::tooltip(
                    widgets::checkbox(
                        "Treat all assets as raw files",
                        state.settings.force_raw_files(),
                    )
                    .on_toggle(move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_force_raw_files(value)),
                        )
                    }),
                    "Disables conversion of all supported asset types",
                    Position::Right,
                )
                .into(),
            ]);
        }

        settings = settings.extend([
            vertical_space().height(2.0).into(),
            text("Troubleshooting options:")
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            vertical_space().height(0.0).into(),
            row([
                widgets::button("Reset Settings")
                    .on_press(save_message(crate::Settings::default()))
                    .into(),
                widgets::button("Open Config Folder")
                    .on_press(Message::from(SettingsMessage::OpenConfigFolder))
                    .into(),
            ])
            .align_y(Alignment::Center)
            .spacing(8.0)
            .into(),
        ]);

        widgets::scrollable(
            settings
                .spacing(8.0)
                .padding(16.0)
                .width(Length::Fill)
                .height(Length::Shrink),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Saves settings to state and disk.
    fn on_save(&mut self, state: &mut AppState, settings: crate::Settings) -> Task<Message> {
        if !state.reload_required {
            state.reload_required = state.settings.reload_required(&settings);
        }

        state.settings = settings;
        state.settings.save(state.name);

        self.custom_scale = state.settings.custom_scale().map(format_custom_scale);

        Task::none()
    }

    /// Allows the user to pick a new export folder.
    fn on_pick_export_folder(&mut self, _: &mut AppState) -> Task<Message> {
        Task::done(Message::from(MainMessage::PickExportFolder))
    }

    /// Opens the export folder.
    fn on_open_export_folder(&mut self, state: &mut AppState) -> Task<Message> {
        system::open_folder(state.settings.output_directory());

        Task::none()
    }

    /// Opens the config folder.
    fn on_open_config_folder(&mut self, _: &mut AppState) -> Task<Message> {
        let Some(project_directory) = ProjectDirs::from("com", "DTZxPorter", "GameTools") else {
            return Task::none();
        };

        system::open_folder(project_directory.config_dir());

        Task::none()
    }

    /// Applies the custom user provided scale value.
    fn on_apply_custom_scale(&mut self, state: &mut AppState) -> Task<Message> {
        let Some(custom_scale) = self.custom_scale.take() else {
            return Task::none();
        };

        let Ok(value) = custom_scale.parse::<f32>() else {
            let warning_task = Task::done(Message::from(MainMessage::Warning(String::from(
                "Custom scale value must be a valid floating point number!",
            ))));

            let reset_task = self.on_save(
                state,
                state
                    .settings
                    .update(|settings| settings.set_custom_scale(Some(1.0))),
            );

            return Task::batch([warning_task, reset_task]);
        };

        self.on_save(
            state,
            state
                .settings
                .update(|settings| settings.set_custom_scale(Some(value))),
        )
    }

    /// Occurs when the user enters a custom scale value.
    fn on_scale_input(&mut self, _: &mut AppState, input: String) -> Task<Message> {
        let input = if input.len() > 8 {
            input.chars().take(8).collect()
        } else {
            input
        };

        self.custom_scale = Some(input);

        Task::none()
    }
}

/// Formats a custom scale factor.
fn format_custom_scale(scale: f32) -> String {
    format!("{scale:?}")
}
