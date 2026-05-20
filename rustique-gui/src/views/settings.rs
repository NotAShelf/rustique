use iced::widget::{button, checkbox, column, row, scrollable, text, text_input};
use iced::{Alignment, Element, Fill};

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
    pub dirty: bool,
    pub status: Option<String>,
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

    let game_card = section_card(
        column![
            section_label("GAME VERSION"),
            column![
                section_label("PINNED GAME VERSION  (leave empty for latest)"),
                text_input("e.g. 1.20.0", &state.pinned_game_version)
                    .on_input(Message::SettingGameVersion),
            ]
            .spacing(6),
        ]
        .spacing(16),
    );

    let updates_card = section_card(
        column![
            section_label("UPDATES"),
            checkbox(state.check_for_updates)
                .label("Check for Rustique updates on launch")
                .on_toggle(Message::SettingCheckUpdates),
        ]
        .spacing(12),
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
