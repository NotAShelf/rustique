use std::collections::{HashMap, HashSet};

use crate::widgets::danger_btn_style;
use human_format::Formatter;
use iced::widget::{Column, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Fill};
use rustique_core::api::api_structs::ModApi;

use crate::app::Message;
use crate::widgets::{active_tab_style, card_style, ghost_btn_style, status_element};

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
}

impl Default for BrowseView {
    fn default() -> Self {
        Self {
            query: String::new(),
            all_mods: Vec::new(),
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

    let header = row![text("Browse Mods").size(22).width(Fill), export_btn,]
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

    // Sort + filter controls
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
            format!("{} mods", state.filtered.len())
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
            "No favorites yet. Click ☆ on any mod to add it."
        } else if !state.query.is_empty() {
            "No results found."
        } else {
            "No mods found."
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
                browse_row(m, favorited, installing, installed_file)
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
) -> Element<'static, Message> {
    let name = m.name.clone().unwrap_or_else(|| m.mod_id.to_string());
    let author = m.author.clone().unwrap_or_default();
    let summary = m.summary.clone().unwrap_or_default();
    let mod_id_str = mod_key(m);
    let fav_key = mod_id_str.clone();
    let downloads = m.downloads;
    let follows = m.follows;
    let dl_label = format_count(downloads);
    let fol_label = format_count(follows);

    let fav_icon = if favorited { "★" } else { "☆" };
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

    let action_btn: Element<'static, Message> = if installing {
        button(text("Installing...").size(13))
            .style(ghost_btn_style)
            .into()
    } else if let Some(file_name) = installed_file {
        button(text("Uninstall").size(13))
            .on_press(Message::DeleteMod(file_name))
            .style(danger_btn_style)
            .into()
    } else {
        button(text("Install").size(13))
            .on_press(Message::InstallMod(mod_id_str))
            .into()
    };

    container(
        row![
            button(text(fav_icon).size(16).color(fav_color))
                .on_press(Message::ToggleFavorite(fav_key))
                .style(|_: &iced::Theme, _| iced::widget::button::Style {
                    background: None,
                    ..Default::default()
                })
                .padding([0, 4]),
            column![
                text(name).size(15),
                row![
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
                .align_y(Alignment::Center),
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
        .align_y(Alignment::Center),
    )
    .padding([10, 12])
    .style(card_style)
    .into()
}

fn format_count(n: i64) -> String {
    if n < 1000 {
        return n.to_string();
    }
    Formatter::new().with_separator("").format(n as f64)
}
