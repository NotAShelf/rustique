use std::collections::{HashMap, HashSet};

use human_format::Formatter;
use iced::widget::{
    Column, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Color, Element, Fill};
use rustique_core::api::api_structs::ModApi;
use rustique_core::version_filter::VersionFilter;

use crate::app::Message;
use crate::widgets::{
    active_tab_style, card_style, danger_btn_style, ghost_btn_style, primary_btn_style,
    status_element,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SortBy {
    #[default]
    Downloads,
    Follows,
    Trending,
    Name,
}

impl SortBy {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Downloads => "Downloads",
            Self::Follows => "Follows",
            Self::Trending => "Trending",
            Self::Name => "Name",
        }
    }
}

pub const PAGE_SIZE: usize = 30;

#[derive(Debug, Clone)]
pub struct BrowseView {
    pub query: String,
    pub all_mods: Vec<ModApi>,
    pub full_mods: Vec<ModApi>,
    pub filtered: Vec<ModApi>,
    pub loading: bool,
    pub status: Option<String>,
    pub sort_by: SortBy,
    pub sort_desc: bool,
    pub page: usize,
    pub favorites: HashSet<String>,
    pub show_favorites_only: bool,
    pub installing: HashSet<String>,
    pub installed_mods: HashMap<String, String>,
    pub confirm_delete: Option<String>,
    pub expanded_mod: Option<String>,
    pub version_filter: VersionFilter,
    pub available_minor_versions: Vec<String>,
}

impl Default for BrowseView {
    fn default() -> Self {
        Self {
            query: String::new(),
            all_mods: Vec::new(),
            full_mods: Vec::new(),
            filtered: Vec::new(),
            loading: false,
            status: None,
            sort_by: SortBy::Downloads,
            sort_desc: true,
            page: 0,
            favorites: HashSet::new(),
            show_favorites_only: false,
            installing: HashSet::new(),
            installed_mods: HashMap::new(),
            confirm_delete: None,
            expanded_mod: None,
            version_filter: VersionFilter::Any,
            available_minor_versions: Vec::new(),
        }
    }
}

impl BrowseView {
    pub fn total_pages(&self) -> usize {
        (self.filtered.len() + PAGE_SIZE - 1) / PAGE_SIZE
    }

    pub fn current_page_mods(&self) -> &[ModApi] {
        let start = self.page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(self.filtered.len());
        &self.filtered[start..end]
    }
}

pub fn view(state: &BrowseView) -> Element<'_, Message> {
    let export_btn: Element<'_, Message> = if !state.favorites.is_empty() {
        button(text("Export Favorites").size(13))
            .on_press(Message::ExportFavorites)
            .style(ghost_btn_style)
            .into()
    } else {
        iced::widget::Space::new().into()
    };

    let refresh_btn = button(text("↺ Refresh").size(13))
        .on_press(Message::BrowseRefresh)
        .style(ghost_btn_style);

    let header = row![
        text("Browse Mods").size(22).width(Fill),
        export_btn,
        refresh_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let search_bar = row![
        text_input("Search mods...", &state.query)
            .on_input(Message::BrowseQueryChanged)
            .on_submit(Message::BrowseSearch)
            .width(Fill),
        button("Search").on_press(Message::BrowseSearch),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let version_controls = {
        let version_options: Vec<String> = std::iter::once("All versions".to_string())
            .chain(state.available_minor_versions.iter().cloned())
            .collect();

        let selected_label = match &state.version_filter {
            VersionFilter::Any => "All versions".to_string(),
            VersionFilter::Exact(v) | VersionFilter::AtLeast(v) => v.clone(),
        };

        let current_is_at_least = matches!(state.version_filter, VersionFilter::AtLeast(_));
        let vlist = pick_list(version_options, Some(selected_label), move |s: String| {
            if s == "All versions" {
                Message::BrowseVersionFilterChanged(VersionFilter::Any)
            } else if current_is_at_least {
                Message::BrowseVersionFilterChanged(VersionFilter::AtLeast(s))
            } else {
                Message::BrowseVersionFilterChanged(VersionFilter::Exact(s))
            }
        })
        .width(140);

        let mode_btn: Element<'_, Message> = match &state.version_filter {
            VersionFilter::Any => iced::widget::Space::new().into(),
            VersionFilter::Exact(v) => button(text("=").size(12))
                .on_press(Message::BrowseVersionFilterChanged(VersionFilter::AtLeast(
                    v.clone(),
                )))
                .padding([5, 8])
                .style(active_tab_style)
                .into(),
            VersionFilter::AtLeast(v) => button(text("≥").size(12))
                .on_press(Message::BrowseVersionFilterChanged(VersionFilter::Exact(
                    v.clone(),
                )))
                .padding([5, 8])
                .style(active_tab_style)
                .into(),
        };

        row![
            text("Version:").size(12).color(Color {
                r: 0.55,
                g: 0.55,
                b: 0.55,
                a: 1.0,
            }),
            vlist,
            mode_btn,
        ]
        .spacing(6)
        .align_y(Alignment::Center)
    };

    let sort_controls = {
        let sorts = [
            SortBy::Downloads,
            SortBy::Follows,
            SortBy::Trending,
            SortBy::Name,
        ];
        let sort_btns = sorts
            .into_iter()
            .map(|s| sort_btn(s, &state.sort_by, state.sort_desc));

        let fav_btn: Element<'_, Message> = {
            let label = if state.show_favorites_only {
                "★ Favorites"
            } else {
                "☆ Favorites"
            };
            if state.show_favorites_only {
                button(text(label).size(12))
                    .on_press(Message::ToggleFavoritesFilter)
                    .padding([5, 10])
                    .style(active_tab_style)
                    .into()
            } else {
                button(text(label).size(12))
                    .on_press(Message::ToggleFavoritesFilter)
                    .padding([5, 10])
                    .style(ghost_btn_style)
                    .into()
            }
        };

        let count_label = if state.loading {
            String::new()
        } else {
            match &state.version_filter {
                VersionFilter::Any => format!("{} mods", state.filtered.len()),
                VersionFilter::Exact(v) => format!("{} mods · v{v}", state.filtered.len()),
                VersionFilter::AtLeast(v) => format!("{} mods · v{v}+", state.filtered.len()),
            }
        };

        row(sort_btns.collect::<Vec<_>>())
            .push(fav_btn)
            .push(iced::widget::Space::new().width(Fill))
            .push(text(count_label).size(12).color(Color {
                r: 0.55,
                g: 0.55,
                b: 0.55,
                a: 1.0,
            }))
            .spacing(4)
            .align_y(Alignment::Center)
    };

    let body: Element<'_, Message> = if state.loading {
        container(text("Fetching mod database...").size(15))
            .center(Fill)
            .height(Fill)
            .into()
    } else if state.filtered.is_empty() {
        let msg = if state.show_favorites_only && state.favorites.is_empty() {
            "No favorites yet. Click ☆ on any mod to add it.".to_string()
        } else if !matches!(state.version_filter, VersionFilter::Any) && state.query.is_empty() {
            format!("No mods found for {}.", state.version_filter.label())
        } else if !state.query.is_empty() {
            "No results found.".to_string()
        } else {
            "No mods found.".to_string()
        };
        container(text(msg).size(14))
            .center(Fill)
            .height(Fill)
            .into()
    } else {
        let page_mods = state.current_page_mods();
        let rows: Vec<Element<'_, Message>> = page_mods
            .iter()
            .map(|m| {
                let key = mod_key(m);
                let favorited = state.favorites.contains(key.as_str());
                let installing = state.installing.contains(&key);
                let installed_file = state.installed_mods.get(&key).cloned();
                let pending_delete = state.confirm_delete.is_some()
                    && state.confirm_delete.as_deref() == installed_file.as_deref();
                let expanded = state.expanded_mod.as_deref() == Some(key.as_str());
                browse_row(
                    m,
                    favorited,
                    installing,
                    installed_file,
                    pending_delete,
                    expanded,
                )
            })
            .collect();

        let total = state.total_pages();
        let page_bar = row![
            button(text("← Prev").size(12))
                .on_press_maybe(if state.page > 0 {
                    Some(Message::BrowsePrevPage)
                } else {
                    None
                })
                .style(ghost_btn_style),
            text(format!("Page {} of {}", state.page + 1, total.max(1))).size(13),
            button(text("Next →").size(12))
                .on_press_maybe(if state.page + 1 < total {
                    Some(Message::BrowseNextPage)
                } else {
                    None
                })
                .style(ghost_btn_style),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        column![
            scrollable(Column::with_children(rows).spacing(6)).height(Fill),
            page_bar,
        ]
        .spacing(8)
        .height(Fill)
        .into()
    };

    column![
        header,
        search_bar,
        version_controls,
        sort_controls,
        status_element(state.status.as_deref()),
        body,
    ]
    .spacing(10)
    .padding(16)
    .width(Fill)
    .height(Fill)
    .into()
}

fn sort_btn<'a>(sort: SortBy, current: &'a SortBy, desc: bool) -> Element<'a, Message> {
    let active = &sort == current;
    let dir = if active {
        if desc { " ↓" } else { " ↑" }
    } else {
        ""
    };
    let label = format!("{}{}", sort.label(), dir);
    let msg = if active {
        Message::BrowseSortToggle
    } else {
        Message::BrowseSortChanged(sort.clone())
    };
    if active {
        button(text(label).size(12))
            .on_press(msg)
            .padding([5, 10])
            .style(active_tab_style)
            .into()
    } else {
        button(text(label).size(12))
            .on_press(msg)
            .padding([5, 10])
            .style(ghost_btn_style)
            .into()
    }
}

fn mod_key(m: &ModApi) -> String {
    m.mod_id_strs
        .first()
        .cloned()
        .unwrap_or_else(|| m.mod_id.to_string())
}

fn browse_row(
    m: &ModApi,
    favorited: bool,
    installing: bool,
    installed_file: Option<String>,
    pending_delete: bool,
    expanded: bool,
) -> Element<'static, Message> {
    let name = m.name.clone().unwrap_or_else(|| m.mod_id.to_string());
    let author = m.author.clone().unwrap_or_default();
    let summary = m.summary.clone().unwrap_or_default();
    let tags = m.tags.clone();
    let last_released = m.last_released.clone();
    let mod_id_str = mod_key(m);
    let fav_key = mod_id_str.clone();
    let dl_label = format_count(m.downloads);
    let fol_label = format_count(m.follows);

    let fav_color = if favorited {
        Color {
            r: 1.0,
            g: 0.84,
            b: 0.0,
            a: 1.0,
        }
    } else {
        Color {
            r: 0.45,
            g: 0.45,
            b: 0.45,
            a: 1.0,
        }
    };
    let fav_icon = if favorited { "★" } else { "☆" };

    let action_btn: Element<'static, Message> = if pending_delete {
        if let Some(file_name) = installed_file.clone() {
            let fname2 = file_name.clone();
            row![
                text("Delete?").size(12).color(Color {
                    r: 0.85,
                    g: 0.35,
                    b: 0.35,
                    a: 1.0
                }),
                button(text("Yes").size(12))
                    .on_press(Message::DeleteMod(fname2))
                    .style(danger_btn_style),
                button(text("No").size(12))
                    .on_press(Message::CancelDelete)
                    .style(ghost_btn_style),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
        } else {
            iced::widget::Space::new().into()
        }
    } else if installing {
        button(text("Installing...").size(13))
            .style(ghost_btn_style)
            .into()
    } else if let Some(file_name) = installed_file {
        button(text("Uninstall").size(13))
            .on_press(Message::RequestDelete(file_name))
            .style(danger_btn_style)
            .into()
    } else {
        button(text("Install").size(13))
            .on_press(Message::InstallMod(mod_id_str.clone()))
            .style(primary_btn_style)
            .into()
    };

    let expand_key = mod_id_str.clone();
    let expand_icon = if expanded { "▼" } else { "▶" };
    let expand_btn = button(text(expand_icon).size(10).color(Color {
        r: 0.50,
        g: 0.50,
        b: 0.50,
        a: 1.0,
    }))
    .on_press(Message::ToggleBrowseDetail(expand_key))
    .style(|_: &iced::Theme, _| iced::widget::button::Style {
        background: None,
        ..Default::default()
    })
    .padding([2, 4]);

    let meta_row = row![
        text(format!("by {author}")).size(12).color(Color {
            r: 0.55,
            g: 0.55,
            b: 0.55,
            a: 1.0
        }),
        text("·").size(12).color(Color {
            r: 0.35,
            g: 0.35,
            b: 0.35,
            a: 1.0
        }),
        text(format!("{dl_label} ↓")).size(12).color(Color {
            r: 0.55,
            g: 0.55,
            b: 0.55,
            a: 1.0
        }),
        text("·").size(12).color(Color {
            r: 0.35,
            g: 0.35,
            b: 0.35,
            a: 1.0
        }),
        text(format!("{fol_label} ♥")).size(12).color(Color {
            r: 0.55,
            g: 0.55,
            b: 0.55,
            a: 1.0
        }),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let main_row: Element<'static, Message> = row![
        button(text(fav_icon).size(16).color(fav_color))
            .on_press(Message::ToggleFavorite(fav_key))
            .style(|_: &iced::Theme, _| iced::widget::button::Style {
                background: None,
                ..Default::default()
            })
            .padding([0, 4]),
        expand_btn,
        column![
            text(name).size(15),
            meta_row,
            text(summary).size(12).color(Color {
                r: 0.65,
                g: 0.65,
                b: 0.65,
                a: 1.0
            }),
        ]
        .spacing(3)
        .width(Fill),
        action_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into();

    let body: Element<'static, Message> = if expanded {
        let tags_str = if tags.is_empty() {
            String::new()
        } else {
            format!("Tags: {}", tags.join(", "))
        };
        let released_str = last_released
            .map(|d| format!("Released: {d}"))
            .unwrap_or_default();

        let detail_parts: Vec<String> = [tags_str, released_str]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

        if detail_parts.is_empty() {
            main_row
        } else {
            column![
                main_row,
                container(text(detail_parts.join("  ·  ")).size(11).color(Color {
                    r: 0.50,
                    g: 0.50,
                    b: 0.50,
                    a: 1.0
                }))
                .padding([4, 12]),
            ]
            .spacing(0)
            .into()
        }
    } else {
        main_row
    };

    container(body).padding([10, 12]).style(card_style).into()
}

fn format_count(n: i64) -> String {
    if n < 1000 {
        return n.to_string();
    }
    Formatter::new().with_separator("").format(n as f64)
}
