use std::fmt::{Display, Formatter};

use iced::widget::{button, checkbox, column, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Fill};
use lithic_core::version::filter::minor_version;

use crate::app::Message;
use crate::widgets::{section_card, section_label, status_element};

#[derive(Debug, Default, Clone)]
pub struct SettingsView {
    pub mod_dir: String,
    pub game_download_dir: String,
    pub pinned_game_version: String,
    pub zip_mod_files: bool,
    pub backup_mods: bool,
    pub backup_mods_dir: String,
    pub notify_of_unzipped_mods: bool,
    pub check_for_updates: bool,
    pub show_execution_time: bool,
    pub modpack_dir: String,
    pub theme_mode: ThemeModeOption,
    pub theme_preset: String,
    pub available_theme_presets: Vec<String>,
    pub initial_page: InitialPageOption,
    pub dirty: bool,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeModeOption {
    #[default]
    System,
    Light,
    Dark,
    Preset,
}

impl ThemeModeOption {
    pub const ALL: [Self; 4] = [Self::System, Self::Light, Self::Dark, Self::Preset];

    pub fn from_config(value: &str) -> Self {
        match value {
            "light" => Self::Light,
            "dark" => Self::Dark,
            "preset" => Self::Preset,
            _ => Self::System,
        }
    }

    pub fn as_config(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Preset => "preset",
        }
    }
}

impl Display for ThemeModeOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "System"),
            Self::Light => write!(f, "Light"),
            Self::Dark => write!(f, "Dark"),
            Self::Preset => write!(f, "Preset"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InitialPageOption {
    #[default]
    Browse,
    Installed,
    Instances,
    GameVersions,
    Settings,
}

impl InitialPageOption {
    pub const ALL: [Self; 5] = [
        Self::Browse,
        Self::Installed,
        Self::Instances,
        Self::GameVersions,
        Self::Settings,
    ];

    pub fn from_config(value: &str) -> Self {
        match value {
            "installed" => Self::Installed,
            "instances" => Self::Instances,
            "game_versions" => Self::GameVersions,
            "settings" => Self::Settings,
            _ => Self::Browse,
        }
    }

    pub fn as_config(self) -> &'static str {
        match self {
            Self::Browse => "browse",
            Self::Installed => "installed",
            Self::Instances => "instances",
            Self::GameVersions => "game_versions",
            Self::Settings => "settings",
        }
    }
}

impl Display for InitialPageOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Browse => write!(f, "Browse"),
            Self::Installed => write!(f, "Installed"),
            Self::Instances => write!(f, "Instances"),
            Self::GameVersions => write!(f, "Game Versions"),
            Self::Settings => write!(f, "Settings"),
        }
    }
}

pub fn view(state: &SettingsView) -> Element<'_, Message> {
    let header = row![text("Settings").size(22).width(Fill)].align_y(Alignment::Center);

    let paths_card = section_card(
        column![
            column![
                section_label("MODS DIRECTORY"),
                text_input("/path/to/mods", &state.mod_dir).on_input(Message::SettingModDir),
            ]
            .spacing(6),
            column![
                section_label("GAME DOWNLOAD DIRECTORY"),
                text_input("/path/to/vintage-story", &state.game_download_dir)
                    .on_input(Message::SettingGameDownloadDir),
            ]
            .spacing(6),
            column![
                section_label("MODPACK DIRECTORY"),
                text_input("/path/to/modpacks", &state.modpack_dir)
                    .on_input(Message::SettingModpackDir),
            ]
            .spacing(6),
        ]
        .spacing(16),
    );

    let browse_gate_note: Element<'_, Message> = if state.pinned_game_version.is_empty() {
        iced::widget::Space::new().into()
    } else if let Some(minor) = minor_version(&state.pinned_game_version) {
        text(format!("Browse will filter to v{minor}"))
            .size(11)
            .color(Color {
                r: 0.45,
                g: 0.75,
                b: 0.50,
                a: 1.0,
            })
            .into()
    } else {
        text("Invalid version format. Try a valid format, e.g., 1.20.0")
            .size(11)
            .color(Color {
                r: 0.75,
                g: 0.40,
                b: 0.40,
                a: 1.0,
            })
            .into()
    };

    let game_card = section_card(
        column![
            section_label("GAME VERSION"),
            column![
                section_label("PINNED GAME VERSION  (leave empty for latest)"),
                text_input("e.g. 1.20.0", &state.pinned_game_version)
                    .on_input(Message::SettingGameVersion),
                browse_gate_note,
            ]
            .spacing(6),
        ]
        .spacing(16),
    );

    let updates_card = section_card(
        column![
            section_label("UPDATES"),
            checkbox(state.check_for_updates)
                .label("Check for Lithic updates on launch")
                .on_toggle(Message::SettingCheckUpdates),
        ]
        .spacing(12),
    );

    let app_card = section_card(
        column![
            section_label("APPEARANCE"),
            row![
                text("Theme").size(12).width(100),
                pick_list(
                    ThemeModeOption::ALL,
                    Some(state.theme_mode),
                    Message::SettingThemeMode
                )
                .width(180),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Preset").size(12).width(100),
                pick_list(
                    state.available_theme_presets.as_slice(),
                    if state.theme_preset.is_empty() {
                        None
                    } else {
                        Some(state.theme_preset.clone())
                    },
                    Message::SettingThemePreset
                )
                .placeholder("Select a built-in preset")
                .width(260),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Startup Page").size(12).width(100),
                pick_list(
                    InitialPageOption::ALL,
                    Some(state.initial_page),
                    Message::SettingInitialPage
                )
                .width(180),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    );

    let files_card = section_card(
        column![
            section_label("MOD FILES"),
            checkbox(state.zip_mod_files)
                .label("Store mods as zip files")
                .on_toggle(Message::SettingZipMods),
            checkbox(state.notify_of_unzipped_mods)
                .label("Notify when mods are stored unzipped")
                .on_toggle(Message::SettingNotifyUnzipped),
            checkbox(state.show_execution_time)
                .label("Show execution time (CLI)")
                .on_toggle(Message::SettingShowExecTime),
        ]
        .spacing(12),
    );

    let backup_card = section_card(
        column![
            section_label("BACKUP"),
            checkbox(state.backup_mods)
                .label("Back up mods before updating")
                .on_toggle(Message::SettingBackupMods),
            {
                let backup_dir: Element<'_, Message> = if state.backup_mods {
                    column![
                        section_label("BACKUP DIRECTORY"),
                        text_input("/path/to/backups", &state.backup_mods_dir)
                            .on_input(Message::SettingBackupModsDir),
                    ]
                    .spacing(6)
                    .into()
                } else {
                    iced::widget::Space::new().into()
                };
                backup_dir
            },
        ]
        .spacing(12),
    );

    let save_label = if state.dirty { "Save *" } else { "Save" };
    let save_btn: Element<'_, Message> = if state.dirty {
        button(save_label).on_press(Message::SaveSettings).into()
    } else {
        button(save_label).into()
    };

    let footer = row![iced::widget::Space::new().width(Fill), save_btn].align_y(Alignment::Center);

    scrollable(
        column![
            header,
            status_element(state.status.as_deref()),
            paths_card,
            game_card,
            app_card,
            updates_card,
            files_card,
            backup_card,
            footer,
        ]
        .spacing(12)
        .padding(16)
        .width(Fill),
    )
    .height(Fill)
    .into()
}
