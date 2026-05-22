use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use iced::widget::{column, container, row, rule, text};
use iced::{Element, Fill, Task, Theme};

use rustique_core::api::api_structs::ModApi;
use rustique_core::sync_structs::ModSyncInfo;

use crate::ops::SettingsData;
use crate::views::browse::{BrowseView, SortBy};
use crate::views::installed::{InstalledTab, InstalledView};
use crate::views::settings::SettingsView;
use crate::views::{browse, installed, settings};

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Browse,
    Installed,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Navigate(View),

    // Installed: mods
    InstalledLoaded(Result<HashMap<String, ModSyncInfo>, String>),
    SyncMods,
    SyncDone(Result<HashMap<String, ModSyncInfo>, String>),
    UpdateAll,
    UpdateDone(Result<(), String>),
    DeleteMod(String),
    DeleteDone(Result<String, String>),

    // Installed: tabs + modpacks
    InstalledTabChanged(InstalledTab),
    PacksLoaded(Result<(Vec<String>, Vec<String>), String>),
    EnablePack(String),
    DisablePack(String),
    PackOpDone(Result<String, String>),
    ShowCreatePackForm(bool),
    CreatePackName(String),
    CreatePackId(String),
    CreatePackVersion(String),
    CreatePackSubmit,
    CreatePackDone(Result<String, String>),

    // Browse
    BrowseLoaded(Result<Vec<ModApi>, String>),
    BrowseQueryChanged(String),
    BrowseSearch,
    BrowseSortChanged(SortBy),
    BrowseSortToggle,
    BrowseNextPage,
    BrowsePrevPage,
    InstallMod(String),
    InstallDone(String, Result<String, String>),
    ToggleFavorite(String),
    ToggleFavoritesFilter,
    ExportFavorites,
    ExportDone(Result<String, String>),
    FavoritesLoaded(Result<HashSet<String>, String>),

    // Settings
    SettingsLoaded(Result<SettingsData, String>),
    SettingModDir(String),
    SettingGameDownloadDir(String),
    SettingGameVersion(String),
    SettingZipMods(bool),
    SettingBackupMods(bool),
    SettingBackupModsDir(String),
    SettingNotifyUnzipped(bool),
    SettingCheckUpdates(bool),
    SettingShowExecTime(bool),
    SettingModpackDir(String),
    SaveSettings,
    SettingsSaved(Result<(), String>),
}

pub struct App {
    pub current_view: View,
    pub mod_dir: PathBuf,
    pub installed: InstalledView,
    pub browse: BrowseView,
    pub settings: SettingsView,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let _ = rustique_core::config::config_manager::init_config();
        let app = App {
            current_view: View::Browse,
            mod_dir: PathBuf::new(),
            installed: InstalledView {
                loading: true,
                ..Default::default()
            },
            browse: BrowseView {
                loading: true,
                ..Default::default()
            },
            settings: SettingsView::default(),
        };
        let task = Task::batch([
            Task::perform(crate::ops::load_installed(), Message::InstalledLoaded),
            Task::perform(crate::ops::load_settings(), Message::SettingsLoaded),
            Task::perform(crate::ops::load_browse(), Message::BrowseLoaded),
            Task::perform(crate::ops::load_favorites(), Message::FavoritesLoaded),
        ]);
        (app, task)
    }

    pub fn title(&self) -> String {
        "Rustique - Vintage Story Mod Manager".to_string()
    }

    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // --- Navigation ---
            Message::Navigate(v) => {
                self.current_view = v.clone();
                match v {
                    View::Installed
                        if self.installed.tab == InstalledTab::Modpacks
                            && self.installed.packs.is_empty()
                            && self.installed.enabled_packs.is_empty() =>
                    {
                        self.installed.loading = true;
                        Task::perform(crate::ops::load_packs(), Message::PacksLoaded)
                    }
                    _ => Task::none(),
                }
            }

            // --- Installed: mods ---
            Message::InstalledLoaded(Ok(mods)) => {
                self.browse.installed_mods = mods
                    .iter()
                    .map(|(id, info)| (id.clone(), info.file_name.clone()))
                    .collect();
                self.installed.mods = mods.into_values().collect();
                self.installed
                    .mods
                    .sort_by(|a, b| a.mod_name.cmp(&b.mod_name));
                self.installed.loading = false;
                Task::none()
            }
            Message::InstalledLoaded(Err(e)) => {
                self.installed.status = Some(format!("Error: {e}"));
                self.installed.loading = false;
                Task::none()
            }
            Message::SyncMods => {
                self.installed.loading = true;
                self.installed.status = Some("Syncing...".to_string());
                let mod_dir = self.mod_dir.clone();
                Task::perform(crate::ops::sync_mods(mod_dir), Message::SyncDone)
            }
            Message::SyncDone(Ok(mods)) => {
                self.browse.installed_mods = mods
                    .iter()
                    .map(|(id, info)| (id.clone(), info.file_name.clone()))
                    .collect();
                self.installed.mods = mods.into_values().collect();
                self.installed
                    .mods
                    .sort_by(|a, b| a.mod_name.cmp(&b.mod_name));
                self.installed.loading = false;
                self.installed.status = Some("Sync complete.".to_string());
                Task::none()
            }
            Message::SyncDone(Err(e)) => {
                self.installed.loading = false;
                self.installed.status = Some(format!("Sync failed: {e}"));
                Task::none()
            }
            Message::UpdateAll => {
                self.installed.loading = true;
                self.installed.status = Some("Updating all mods...".to_string());
                let mod_dir = self.mod_dir.clone();
                Task::perform(crate::ops::update_all(mod_dir), Message::UpdateDone)
            }
            Message::UpdateDone(Ok(())) => {
                self.installed.loading = false;
                self.installed.status = Some("Update complete.".to_string());
                let mod_dir = self.mod_dir.clone();
                Task::perform(
                    crate::ops::load_installed_from(mod_dir),
                    Message::InstalledLoaded,
                )
            }
            Message::UpdateDone(Err(e)) => {
                self.installed.loading = false;
                self.installed.status = Some(format!("Update failed: {e}"));
                Task::none()
            }
            Message::DeleteMod(file_name) => {
                let mod_dir = self.mod_dir.clone();
                Task::perform(
                    crate::ops::delete_mod(mod_dir, file_name),
                    Message::DeleteDone,
                )
            }
            Message::DeleteDone(Ok(file_name)) => {
                self.installed.mods.retain(|m| m.file_name != file_name);
                self.browse.installed_mods.retain(|_, f| f != &file_name);
                self.installed.status = Some(format!("Deleted {file_name}"));
                Task::none()
            }
            Message::DeleteDone(Err(e)) => {
                self.installed.status = Some(format!("Delete failed: {e}"));
                Task::none()
            }

            // --- Installed: tabs + modpacks ---
            Message::InstalledTabChanged(tab) => {
                self.installed.tab = tab.clone();
                if tab == InstalledTab::Modpacks
                    && self.installed.packs.is_empty()
                    && self.installed.enabled_packs.is_empty()
                {
                    self.installed.loading = true;
                    Task::perform(crate::ops::load_packs(), Message::PacksLoaded)
                } else {
                    Task::none()
                }
            }
            Message::PacksLoaded(Ok((packs, enabled))) => {
                self.installed.packs = packs;
                self.installed.enabled_packs = enabled;
                self.installed.loading = false;
                Task::none()
            }
            Message::PacksLoaded(Err(e)) => {
                self.installed.status = Some(format!("Error: {e}"));
                self.installed.loading = false;
                Task::none()
            }
            Message::EnablePack(id) => {
                Task::perform(crate::ops::enable_pack(id), Message::PackOpDone)
            }
            Message::DisablePack(id) => {
                Task::perform(crate::ops::disable_pack(id), Message::PackOpDone)
            }
            Message::PackOpDone(Ok(msg)) => {
                self.installed.status = Some(msg);
                Task::perform(crate::ops::load_packs(), Message::PacksLoaded)
            }
            Message::PackOpDone(Err(e)) => {
                self.installed.status = Some(format!("Error: {e}"));
                Task::none()
            }
            Message::ShowCreatePackForm(show) => {
                self.installed.show_create_form = show;
                Task::none()
            }
            Message::CreatePackName(v) => {
                self.installed.create_name = v;
                Task::none()
            }
            Message::CreatePackId(v) => {
                self.installed.create_id = v;
                Task::none()
            }
            Message::CreatePackVersion(v) => {
                self.installed.create_version = v;
                Task::none()
            }
            Message::CreatePackSubmit => {
                let mod_dir = self.mod_dir.clone();
                let name = self.installed.create_name.clone();
                let id = self.installed.create_id.clone();
                let version = self.installed.create_version.clone();
                self.installed.loading = true;
                self.installed.show_create_form = false;
                Task::perform(
                    crate::ops::create_pack(mod_dir, name, id, version),
                    Message::CreatePackDone,
                )
            }
            Message::CreatePackDone(Ok(msg)) => {
                self.installed.loading = false;
                self.installed.status = Some(msg);
                self.installed.create_name.clear();
                self.installed.create_id.clear();
                self.installed.create_version.clear();
                Task::perform(crate::ops::load_packs(), Message::PacksLoaded)
            }
            Message::CreatePackDone(Err(e)) => {
                self.installed.loading = false;
                self.installed.status = Some(format!("Create failed: {e}"));
                Task::none()
            }

            // --- Browse ---
            Message::BrowseLoaded(Ok(mods)) => {
                self.browse.all_mods = mods;
                self.browse.loading = false;
                apply_browse_filter(&mut self.browse);
                Task::none()
            }
            Message::BrowseLoaded(Err(e)) => {
                self.browse.status = Some(format!("Failed to load mods: {e}"));
                self.browse.loading = false;
                Task::none()
            }
            Message::BrowseQueryChanged(q) => {
                self.browse.query = q;
                apply_browse_filter(&mut self.browse);
                Task::none()
            }
            Message::BrowseSearch => {
                apply_browse_filter(&mut self.browse);
                Task::none()
            }
            Message::BrowseSortChanged(sort) => {
                self.browse.sort_by = sort;
                apply_browse_filter(&mut self.browse);
                Task::none()
            }
            Message::BrowseSortToggle => {
                self.browse.sort_desc = !self.browse.sort_desc;
                apply_browse_filter(&mut self.browse);
                Task::none()
            }
            Message::BrowseNextPage => {
                if self.browse.page + 1 < self.browse.total_pages() {
                    self.browse.page += 1;
                }
                Task::none()
            }
            Message::BrowsePrevPage => {
                if self.browse.page > 0 {
                    self.browse.page -= 1;
                }
                Task::none()
            }
            Message::InstallMod(mod_id) => {
                self.browse.installing.insert(mod_id.clone());
                let mod_dir = self.mod_dir.clone();
                let id = mod_id.clone();
                Task::perform(crate::ops::install_mod(mod_dir, mod_id), move |r| {
                    Message::InstallDone(id, r)
                })
            }
            Message::InstallDone(mod_id, Ok(name)) => {
                self.browse.installing.remove(&mod_id);
                self.browse.status = Some(format!("Installed {name}"));
                let mod_dir = self.mod_dir.clone();
                Task::perform(
                    crate::ops::load_installed_from(mod_dir),
                    Message::InstalledLoaded,
                )
            }
            Message::InstallDone(mod_id, Err(e)) => {
                self.browse.installing.remove(&mod_id);
                self.browse.status = Some(format!("Install failed: {e}"));
                Task::none()
            }
            Message::ToggleFavorite(key) => {
                if self.browse.favorites.contains(&key) {
                    self.browse.favorites.remove(&key);
                } else {
                    self.browse.favorites.insert(key);
                }
                let favs = self.browse.favorites.clone();
                apply_browse_filter(&mut self.browse);
                Task::perform(crate::ops::save_favorites(favs), |_| {
                    Message::ExportDone(Ok(String::new()))
                })
            }
            Message::ToggleFavoritesFilter => {
                self.browse.show_favorites_only = !self.browse.show_favorites_only;
                apply_browse_filter(&mut self.browse);
                Task::none()
            }
            Message::ExportFavorites => {
                let favs = self.browse.favorites.clone();
                Task::perform(crate::ops::export_favorites(favs), Message::ExportDone)
            }
            Message::ExportDone(Ok(path)) => {
                if !path.is_empty() {
                    self.browse.status = Some(format!("Exported favorites to {path}"));
                }
                Task::none()
            }
            Message::ExportDone(Err(e)) => {
                self.browse.status = Some(format!("Export failed: {e}"));
                Task::none()
            }
            Message::FavoritesLoaded(Ok(favs)) => {
                self.browse.favorites = favs;
                apply_browse_filter(&mut self.browse);
                Task::none()
            }
            Message::FavoritesLoaded(Err(_)) => Task::none(),

            // --- Settings ---
            Message::SettingsLoaded(Ok(data)) => {
                self.settings.mod_dir = data.mod_dir.clone();
                self.settings.game_download_dir = data.game_download_dir;
                self.settings.pinned_game_version = data.pinned_game_version;
                self.settings.zip_mod_files = data.zip_mod_files;
                self.settings.backup_mods = data.backup_mods;
                self.settings.backup_mods_dir = data.backup_mods_dir;
                self.settings.notify_of_unzipped_mods = data.notify_of_unzipped_mods;
                self.settings.check_for_updates = data.check_for_updates;
                self.settings.show_execution_time = data.show_execution_time;
                self.settings.modpack_dir = data.modpack_dir;
                self.settings.dirty = false;
                self.mod_dir = PathBuf::from(data.mod_dir);
                Task::none()
            }
            Message::SettingsLoaded(Err(e)) => {
                tracing::error!("Settings load error: {e}");
                Task::none()
            }
            Message::SettingModDir(v) => {
                self.settings.mod_dir = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingGameDownloadDir(v) => {
                self.settings.game_download_dir = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingGameVersion(v) => {
                self.settings.pinned_game_version = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingZipMods(v) => {
                self.settings.zip_mod_files = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingBackupMods(v) => {
                self.settings.backup_mods = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingBackupModsDir(v) => {
                self.settings.backup_mods_dir = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingNotifyUnzipped(v) => {
                self.settings.notify_of_unzipped_mods = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingCheckUpdates(v) => {
                self.settings.check_for_updates = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingShowExecTime(v) => {
                self.settings.show_execution_time = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SettingModpackDir(v) => {
                self.settings.modpack_dir = v;
                self.settings.dirty = true;
                Task::none()
            }
            Message::SaveSettings => {
                let data = SettingsData {
                    mod_dir: self.settings.mod_dir.clone(),
                    game_download_dir: self.settings.game_download_dir.clone(),
                    pinned_game_version: self.settings.pinned_game_version.clone(),
                    zip_mod_files: self.settings.zip_mod_files,
                    backup_mods: self.settings.backup_mods,
                    backup_mods_dir: self.settings.backup_mods_dir.clone(),
                    notify_of_unzipped_mods: self.settings.notify_of_unzipped_mods,
                    check_for_updates: self.settings.check_for_updates,
                    show_execution_time: self.settings.show_execution_time,
                    modpack_dir: self.settings.modpack_dir.clone(),
                };
                Task::perform(crate::ops::save_settings(data), Message::SettingsSaved)
            }
            Message::SettingsSaved(Ok(())) => {
                self.settings.dirty = false;
                self.settings.status = Some("Settings saved.".to_string());
                self.mod_dir = PathBuf::from(&self.settings.mod_dir);
                Task::none()
            }
            Message::SettingsSaved(Err(e)) => {
                self.settings.status = Some(format!("Save failed: {e}"));
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let nav_items: &[(&str, View)] = &[
            ("Browse", View::Browse),
            ("Installed", View::Installed),
            ("Settings", View::Settings),
        ];

        let nav_buttons: Vec<Element<'_, Message>> = nav_items
            .iter()
            .map(|(label, view)| {
                crate::widgets::nav_button(
                    label,
                    self.current_view == *view,
                    Message::Navigate(view.clone()),
                )
            })
            .collect();

        let sidebar = container(column![
            container(text("Rustique").size(17)).padding(iced::Padding {
                top: 16.0,
                right: 16.0,
                bottom: 12.0,
                left: 20.0
            }),
            rule::horizontal(1),
            column(nav_buttons).spacing(2).padding([4, 0]),
            iced::widget::space::vertical(),
            rule::horizontal(1),
            container(text("v0.5.16 α").size(11).color(iced::Color {
                r: 0.40,
                g: 0.40,
                b: 0.40,
                a: 1.0
            }))
            .padding([8, 16]),
        ])
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(
                iced::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.02,
                }
                .into(),
            ),
            ..Default::default()
        })
        .width(160)
        .height(Fill);

        let content: Element<'_, Message> = match &self.current_view {
            View::Browse => browse::view(&self.browse),
            View::Installed => installed::view(&self.installed),
            View::Settings => settings::view(&self.settings),
        };

        container(row![sidebar, rule::vertical(1), content].spacing(0))
            .width(Fill)
            .height(Fill)
            .into()
    }
}

fn browse_mod_key(m: &ModApi) -> String {
    m.mod_id_strs
        .first()
        .cloned()
        .unwrap_or_else(|| m.mod_id.to_string())
}

fn apply_browse_filter(browse: &mut BrowseView) {
    let base: Vec<ModApi> = if browse.query.trim().is_empty() {
        browse.all_mods.clone()
    } else {
        crate::ops::search_mods(&browse.all_mods, &browse.query)
    };

    let mut filtered: Vec<ModApi> = if browse.show_favorites_only {
        base.into_iter()
            .filter(|m| browse.favorites.contains(&browse_mod_key(m)))
            .collect()
    } else {
        base
    };

    match browse.sort_by {
        SortBy::Downloads => {
            filtered.sort_by(|a, b| {
                if browse.sort_desc {
                    b.downloads.cmp(&a.downloads)
                } else {
                    a.downloads.cmp(&b.downloads)
                }
            });
        }
        SortBy::Follows => {
            filtered.sort_by(|a, b| {
                if browse.sort_desc {
                    b.follows.cmp(&a.follows)
                } else {
                    a.follows.cmp(&b.follows)
                }
            });
        }
        SortBy::Trending => {
            filtered.sort_by(|a, b| {
                if browse.sort_desc {
                    b.trending_points.cmp(&a.trending_points)
                } else {
                    a.trending_points.cmp(&b.trending_points)
                }
            });
        }
        SortBy::Name => {
            filtered.sort_by(|a, b| {
                let an = a.name.as_deref().unwrap_or("");
                let bn = b.name.as_deref().unwrap_or("");
                if browse.sort_desc {
                    bn.cmp(an)
                } else {
                    an.cmp(bn)
                }
            });
        }
    }

    browse.filtered = filtered;
    browse.page = 0;
}

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .window(iced::window::Settings {
            size: iced::Size::new(1100.0, 720.0),
            ..Default::default()
        })
        .theme(Theme::Dark)
        .run()
}
