use iced::widget::{Column, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Fill, Length};
use lithic_core::sync::structs::ModSyncInfo;
use lithic_core::version::filter::minor_version;

use crate::app::Message;
use crate::widgets::{active_tab_style, card_style, danger_btn_style, ghost_btn_style, status_element};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum InstalledTab {
    #[default]
    Mods,
    Modpacks,
}

#[derive(Debug, Default, Clone)]
pub struct InstalledView {
    pub tab: InstalledTab,
    pub mods: Vec<ModSyncInfo>,
    pub packs: Vec<String>,
    pub enabled_packs: Vec<String>,
    pub loading: bool,
    pub status: Option<String>,
    pub show_create_form: bool,
    pub create_name: String,
    pub create_id: String,
    pub create_version: String,
    pub confirm_delete: Option<String>,
    pub expanded_mod: Option<String>,
    pub search: String,
}

pub fn view<'a>(state: &'a InstalledView, pinned_game_version: &str) -> Element<'a, Message> {
    let header = row![
        text("Installed").size(22).width(Fill),
        button("Sync").on_press(Message::SyncMods),
        button("Update All").on_press(Message::UpdateAll),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let tab_bar = row![
        tab_btn(
            "Mods",
            state.tab == InstalledTab::Mods,
            Message::InstalledTabChanged(InstalledTab::Mods)
        ),
        tab_btn(
            "Modpacks",
            state.tab == InstalledTab::Modpacks,
            Message::InstalledTabChanged(InstalledTab::Modpacks)
        ),
    ]
    .spacing(4);

    let body: Element<'_, Message> = if state.loading {
        container(text("Loading...").size(15))
            .center(Fill)
            .height(Fill)
            .into()
    } else {
        match state.tab {
            InstalledTab::Mods => mods_body(state, pinned_game_version),
            InstalledTab::Modpacks => packs_body(state),
        }
    };

    column![header, tab_bar, status_element(state.status.as_deref()), body]
        .spacing(10)
        .padding(16)
        .width(Fill)
        .height(Fill)
        .into()
}

fn tab_btn(label: &str, active: bool, msg: Message) -> Element<'_, Message> {
    if active {
        button(text(label).size(13))
            .padding([6, 14])
            .on_press(msg)
            .style(active_tab_style)
            .into()
    } else {
        button(text(label).size(13))
            .padding([6, 14])
            .on_press(msg)
            .style(ghost_btn_style)
            .into()
    }
}

fn mods_body<'a>(state: &'a InstalledView, pinned_game_version: &str) -> Element<'a, Message> {
    if state.mods.is_empty() {
        return container(
            column![
                text("No mods installed").size(16),
                text("Configure your mods directory in Settings, then click Sync.")
                    .size(13)
                    .color(Color {
                        r: 0.55,
                        g: 0.55,
                        b: 0.55,
                        a: 1.0
                    }),
            ]
            .spacing(6)
            .align_x(Alignment::Center),
        )
        .center(Fill)
        .height(Fill)
        .into();
    }

    let search_bar = text_input("Filter installed mods...", &state.search)
        .on_input(Message::InstalledSearchChanged)
        .width(Fill);

    let q = state.search.to_lowercase();
    let displayed: Vec<&ModSyncInfo> = state
        .mods
        .iter()
        .filter(|m| q.is_empty() || m.mod_name.to_lowercase().contains(&q))
        .collect();

    let pinned_minor = minor_version(pinned_game_version);
    let rows: Vec<Element<'_, Message>> = displayed
        .iter()
        .map(|m| {
            let pending = state.confirm_delete.as_deref() == Some(m.file_name.as_ref());
            let expanded = state.expanded_mod.as_deref() == Some(m.file_name.as_ref());
            mod_row(m, pending, expanded, pinned_minor.as_deref())
        })
        .collect();

    let list: Element<'_, Message> = if rows.is_empty() {
        container(text("No mods match the filter.").size(13))
            .center(Fill)
            .height(Fill)
            .into()
    } else {
        scrollable(Column::with_children(rows).spacing(6))
            .height(Fill)
            .into()
    };

    column![search_bar, list].spacing(8).height(Fill).into()
}

fn packs_body<'a>(state: &'a InstalledView) -> Element<'a, Message> {
    let create_section: Element<'a, Message> = if state.show_create_form {
        let can_submit = !state.create_name.is_empty() && !state.create_id.is_empty();
        container(
            column![
                text("Create Modpack from Installed Mods").size(14),
                row![
                    text("Name").size(12).width(70),
                    text_input("e.g. My Pack", &state.create_name)
                        .on_input(Message::CreatePackName)
                        .width(Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text("ID").size(12).width(70),
                    text_input("e.g. mypack", &state.create_id)
                        .on_input(Message::CreatePackId)
                        .width(Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text("Version").size(12).width(70),
                    text_input("e.g. 1.0.0", &state.create_version)
                        .on_input(Message::CreatePackVersion)
                        .width(Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    button(text("Create").size(13)).on_press_maybe(if can_submit {
                        Some(Message::CreatePackSubmit)
                    } else {
                        None
                    }),
                    button(text("Cancel").size(13))
                        .on_press(Message::ShowCreatePackForm(false))
                        .style(ghost_btn_style),
                ]
                .spacing(8),
            ]
            .spacing(8),
        )
        .padding(12)
        .style(card_style)
        .into()
    } else {
        button(text("+ Create Modpack").size(13))
            .on_press(Message::ShowCreatePackForm(true))
            .style(ghost_btn_style)
            .into()
    };

    if state.packs.is_empty() && state.enabled_packs.is_empty() {
        column![
            create_section,
            container(
                column![
                    text("No modpacks installed").size(16),
                    text("Create one above from your installed mods, or install via the CLI.")
                        .size(13)
                        .color(Color {
                            r: 0.55,
                            g: 0.55,
                            b: 0.55,
                            a: 1.0
                        }),
                ]
                .spacing(6)
                .align_x(Alignment::Center),
            )
            .center(Fill)
            .height(Fill),
        ]
        .spacing(10)
        .height(Fill)
        .into()
    } else {
        let mut rows: Vec<Element<'a, Message>> = vec![create_section];
        for id in &state.enabled_packs {
            rows.push(pack_row(id, true));
        }
        for id in &state.packs {
            if !state.enabled_packs.contains(id) {
                rows.push(pack_row(id, false));
            }
        }
        scrollable(Column::with_children(rows).spacing(6))
            .height(Fill)
            .into()
    }
}

fn mod_row<'a>(
    m: &'a ModSyncInfo,
    pending_delete: bool,
    expanded: bool,
    pinned_minor: Option<&str>,
) -> Element<'a, Message> {
    let needs_update = !m.latest_known_version.is_empty() && m.installed_version != m.latest_known_version;

    let update_badge: Element<'_, Message> = if needs_update {
        container(
            text(format!("↑ {}", m.latest_known_version))
                .size(11)
                .color(Color::WHITE),
        )
        .padding([3, 8])
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(
                Color {
                    r: 0.20,
                    g: 0.55,
                    b: 0.25,
                    a: 1.0,
                }
                .into(),
            ),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        iced::widget::Space::new().into()
    };

    let delete_area: Element<'_, Message> = if pending_delete {
        row![
            text("Delete?").size(12).color(Color {
                r: 0.85,
                g: 0.35,
                b: 0.35,
                a: 1.0
            }),
            button(text("Yes").size(12))
                .on_press(Message::DeleteMod(m.file_name.to_string()))
                .style(danger_btn_style),
            button(text("No").size(12))
                .on_press(Message::CancelDelete)
                .style(ghost_btn_style),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
    } else {
        button(text("Delete").size(13))
            .on_press(Message::RequestDelete(m.file_name.to_string()))
            .style(danger_btn_style)
            .into()
    };

    let expand_icon = if expanded { "▼" } else { "▶" };
    let expand_btn = button(text(expand_icon).size(10).color(Color {
        r: 0.50,
        g: 0.50,
        b: 0.50,
        a: 1.0,
    }))
    .on_press(Message::ToggleInstalledDetail(m.file_name.to_string()))
    .style(|_: &iced::Theme, _| iced::widget::button::Style {
        background: None,
        ..Default::default()
    })
    .padding([2, 4]);

    let compat_chip: Element<'_, Message> = match pinned_minor {
        None => iced::widget::Space::new().into(),
        Some(minor) => {
            let compatible = m
                .game_versions
                .iter()
                .any(|v| minor_version(v).as_deref() == Some(minor));
            let unknown = m.game_versions.is_empty();

            let (symbol, bg) = if unknown {
                (
                    "?",
                    Color {
                        r: 0.30,
                        g: 0.30,
                        b: 0.35,
                        a: 1.0,
                    },
                )
            } else if compatible {
                (
                    "✓",
                    Color {
                        r: 0.15,
                        g: 0.50,
                        b: 0.20,
                        a: 1.0,
                    },
                )
            } else {
                (
                    "✗",
                    Color {
                        r: 0.55,
                        g: 0.18,
                        b: 0.18,
                        a: 1.0,
                    },
                )
            };

            container(text(symbol).size(11).color(Color::WHITE))
                .padding([3, 7])
                .style(move |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(bg.into()),
                    border: iced::Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        }
    };

    let main_row: Element<'_, Message> = row![
        expand_btn,
        column![
            text(&m.mod_name).size(14),
            text(m.installed_version.to_string()).size(12).color(Color {
                r: 0.55,
                g: 0.55,
                b: 0.55,
                a: 1.0
            }),
        ]
        .spacing(2)
        .width(Fill),
        compat_chip,
        update_badge,
        delete_area,
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into();

    let body: Element<'_, Message> = if expanded {
        let versions_str = if m.game_versions.is_empty() {
            String::new()
        } else {
            format!("Game versions: {}", m.game_versions.join(", "))
        };
        let changelog_preview = if m.latest_changelog.is_empty() {
            String::new()
        } else {
            let preview: String = m.latest_changelog.chars().take(200).collect();
            if m.latest_changelog.len() > 200 {
                format!("{preview}…")
            } else {
                preview
            }
        };

        let detail_items: Vec<Element<'_, Message>> = [versions_str, changelog_preview]
            .into_iter()
            .filter(|s| !s.is_empty())
            .map(|s| {
                text(s)
                    .size(11)
                    .color(Color {
                        r: 0.50,
                        g: 0.50,
                        b: 0.50,
                        a: 1.0,
                    })
                    .into()
            })
            .collect();

        if detail_items.is_empty() {
            main_row
        } else {
            column![
                main_row,
                container(Column::with_children(detail_items).spacing(3)).padding([4, 12]),
            ]
            .spacing(0)
            .into()
        }
    } else {
        main_row
    };

    container(body)
        .padding([10, 12])
        .width(Length::Fill)
        .style(card_style)
        .into()
}

fn pack_row(id: &str, enabled: bool) -> Element<'_, Message> {
    let badge: Element<'_, Message> = if enabled {
        container(text("ACTIVE").size(11).color(Color::WHITE))
            .padding([3, 8])
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(
                    Color {
                        r: 0.20,
                        g: 0.55,
                        b: 0.25,
                        a: 1.0,
                    }
                    .into(),
                ),
                border: iced::Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    } else {
        container(text("inactive").size(11).color(Color {
            r: 0.50,
            g: 0.50,
            b: 0.50,
            a: 1.0,
        }))
        .padding([3, 8])
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.06,
                }
                .into(),
            ),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    };

    let toggle: Element<'_, Message> = if enabled {
        button(text("Disable").size(13))
            .on_press(Message::DisablePack(id.to_string()))
            .style(ghost_btn_style)
            .into()
    } else {
        button(text("Enable").size(13))
            .on_press(Message::EnablePack(id.to_string()))
            .into()
    };

    container(
        row![text(id).size(14).width(Fill), badge, toggle,]
            .spacing(10)
            .align_y(Alignment::Center),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .style(card_style)
    .into()
}
