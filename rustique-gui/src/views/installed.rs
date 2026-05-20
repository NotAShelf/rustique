use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Fill, Length};
use rustique_core::sync_structs::ModSyncInfo;

use crate::app::Message;
use crate::widgets::{
    active_tab_style, card_style, danger_btn_style, ghost_btn_style, status_element,
};

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
}

pub fn view(state: &InstalledView) -> Element<'_, Message> {
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
            InstalledTab::Mods => mods_body(state),
            InstalledTab::Modpacks => packs_body(state),
        }
    };

    column![
        header,
        tab_bar,
        status_element(state.status.as_deref()),
        body
    ]
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

fn mods_body(state: &InstalledView) -> Element<'_, Message> {
    if state.mods.is_empty() {
        container(
            column![
                text("No mods installed").size(16),
                text("Configure your mods directory in Settings, then click Sync.")
                    .size(13)
                    .color(Color {
                        r: 0.55,
                        g: 0.55,
                        b: 0.55,
                        a: 1.0,
                    }),
            ]
            .spacing(6)
            .align_x(Alignment::Center),
        )
        .center(Fill)
        .height(Fill)
        .into()
    } else {
        let rows: Vec<Element<'_, Message>> = state.mods.iter().map(mod_row).collect();
        scrollable(Column::with_children(rows).spacing(6))
            .height(Fill)
            .into()
    }
}

fn packs_body<'a>(state: &'a InstalledView) -> Element<'a, Message> {
    if state.packs.is_empty() && state.enabled_packs.is_empty() {
        container(
            column![
                text("No modpacks installed").size(16),
                text("Use `rustique modpack install <id>` from the CLI to install a modpack.")
                    .size(13)
                    .color(Color {
                        r: 0.55,
                        g: 0.55,
                        b: 0.55,
                        a: 1.0,
                    }),
            ]
            .spacing(6)
            .align_x(Alignment::Center),
        )
        .center(Fill)
        .height(Fill)
        .into()
    } else {
        // Enabled packs first, then disabled (no duplicates)
        let mut rows: Vec<Element<'a, Message>> = Vec::new();
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

fn mod_row(m: &ModSyncInfo) -> Element<'_, Message> {
    let needs_update =
        !m.latest_known_version.is_empty() && m.installed_version != m.latest_known_version;

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

    container(
        row![
            column![
                text(&m.mod_name).size(14),
                text(&m.installed_version).size(12).color(Color {
                    r: 0.55,
                    g: 0.55,
                    b: 0.55,
                    a: 1.0,
                }),
            ]
            .spacing(2)
            .width(Fill),
            update_badge,
            button(text("Delete").size(13))
                .on_press(Message::DeleteMod(m.file_name.clone()))
                .style(danger_btn_style),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
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
