use iced::widget::{Column, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Fill, Length};
use lithic_core::api::structs::ModApi;
use lithic_core::instance::{GameVersionInstall, InstanceConfig};

use crate::app::Message;
use crate::views::browse::SortBy;
use crate::widgets::{
   active_tab_style, card_style, danger_btn_style, ghost_btn_style, primary_btn_style, status_element,
};

const MOD_PICKER_PAGE_SIZE: usize = 12;

#[derive(Debug, Clone, Default)]
pub struct InstancesView {
   pub instances: Vec<InstanceConfig>,
   pub game_versions: Vec<GameVersionInstall>,
   pub available_mods: Vec<ModApi>,
   pub selected_mod_ids: Vec<String>,
   pub mod_search: String,
   pub show_mod_picker: bool,
   pub mod_sort_by: SortBy,
   pub mod_sort_desc: bool,
   pub mod_page: usize,
   pub active_instance_id: String,
   pub loading: bool,
   pub status: Option<String>,
   pub form_id: String,
   pub form_name: String,
   pub form_data_dir: String,
   pub form_mods_dir: String,
   pub form_game_version_id: String,
   pub form_start_params: String,
   pub form_env_vars: String,
}

pub fn view(state: &InstancesView) -> Element<'_, Message> {
   let header = row![
      text("Instances").size(22).width(Fill),
      button("Launch Active")
         .on_press(Message::LaunchActiveInstance)
         .style(primary_btn_style),
   ]
   .spacing(8)
   .align_y(Alignment::Center);

   let active_summary = state
      .instances
      .iter()
      .find(|i| i.id == state.active_instance_id)
      .map(|i| format!("Active: {} - {} - {}", i.name, i.game_version_id, i.mods_dir))
      .unwrap_or_else(|| "No active instance selected".to_string());

   let version_buttons: Vec<Element<'_, Message>> = state
      .game_versions
      .iter()
      .map(|gv| {
         button(text(gv.version.as_str()).size(12))
            .on_press(Message::InstanceFormGameVersionId(gv.id.clone()))
            .style(if state.form_game_version_id == gv.id {
               primary_btn_style
            } else {
               ghost_btn_style
            })
            .into()
      })
      .collect();

   let basics = column![
      text_input("id", &state.form_id).on_input(Message::InstanceFormId),
      text_input("name", &state.form_name).on_input(Message::InstanceFormName),
      text_input("game version id", &state.form_game_version_id).on_input(Message::InstanceFormGameVersionId),
      row(version_buttons).spacing(4),
   ]
   .spacing(6)
   .width(Fill);

   let paths = column![
      row![
         text_input("data dir", &state.form_data_dir)
            .on_input(Message::InstanceFormDataDir)
            .width(Fill),
         button("Browse")
            .on_press(Message::PickInstanceDataDir)
            .style(ghost_btn_style),
      ]
      .spacing(6),
      row![
         text_input("mods dir", &state.form_mods_dir)
            .on_input(Message::InstanceFormModsDir)
            .width(Fill),
         button("Browse")
            .on_press(Message::PickInstanceModsDir)
            .style(ghost_btn_style),
      ]
      .spacing(6),
      text_input("start params", &state.form_start_params).on_input(Message::InstanceFormStartParams),
      text_input("env vars (K=V,K2=V2)", &state.form_env_vars).on_input(Message::InstanceFormEnvVars),
   ]
   .spacing(6)
   .width(Fill);

   let form = container(
      column![
         text("Create / Update Instance").size(14),
         row![basics, paths].spacing(12),
         row![
            button("Save Instance")
               .on_press(Message::SaveInstance)
               .style(primary_btn_style),
            button("Reload")
               .on_press(Message::ReloadInstances)
               .style(ghost_btn_style),
            button("Clear")
               .on_press(Message::ClearInstanceForm)
               .style(ghost_btn_style),
         ]
         .spacing(8),
      ]
      .spacing(6),
   )
   .padding(12)
   .style(card_style);

   let selected_mod_rows = selected_mod_rows(state);

   let mod_picker = container(
      column![
         row![
            text("Instance Mods").size(14).width(Fill),
            text(format!("{} selected", state.selected_mod_ids.len())).size(12),
         ]
         .align_y(Alignment::Center),
         row![
            button("Browse Mods")
               .on_press(Message::OpenInstanceModPicker)
               .style(primary_btn_style),
         ],
         scrollable(Column::with_children(selected_mod_rows).spacing(6)).height(Length::Fixed(128.0)),
      ]
      .spacing(8),
   )
   .padding(12)
   .style(card_style);

   let mut rows: Vec<Element<'_, Message>> = Vec::new();
   for inst in &state.instances {
      rows.push(
         container(
            row![
               column![
                  text(format!("{} ({})", inst.name, inst.id)).size(14),
                  text(format!("mods: {}", inst.mods_dir)).size(12),
                  text(format!("version: {}", inst.game_version_id)).size(12),
               ]
               .spacing(4)
               .width(Fill),
               button(if state.active_instance_id == inst.id {
                  "Active"
               } else {
                  "Activate"
               })
               .on_press(Message::SelectInstance(inst.id.clone()))
               .style(ghost_btn_style),
               button("Edit")
                  .on_press(Message::EditInstance(inst.id.clone()))
                  .style(ghost_btn_style),
               button("Delete")
                  .on_press(Message::DeleteInstance(inst.id.clone()))
                  .style(danger_btn_style),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
         )
         .padding(10)
         .style(card_style)
         .into(),
      );
   }

   let body: Element<'_, Message> = if state.loading {
      container(text("Loading...")).center(Fill).into()
   } else {
      scrollable(column(rows).spacing(6)).height(Fill).into()
   };

   let base = column![
      header,
      text(active_summary).size(12),
      status_element(state.status.as_deref()),
      form,
      mod_picker,
      body
   ]
   .spacing(10)
   .padding(16)
   .height(Fill);

   if state.show_mod_picker {
      column![base, mod_picker_modal(state)].spacing(0).into()
   } else {
      base.into()
   }
}

fn selected_mod_rows(state: &InstancesView) -> Vec<Element<'_, Message>> {
   if state.selected_mod_ids.is_empty() {
      return vec![
         container(text("No mods selected for this instance.").size(12))
            .padding(8)
            .style(card_style)
            .into(),
      ];
   }

   state
      .selected_mod_ids
      .iter()
      .map(|id| {
         let name = state
            .available_mods
            .iter()
            .find(|m| mod_key(m) == *id)
            .and_then(|m| m.name.clone())
            .unwrap_or_else(|| id.clone());
         container(
            row![
               column![text(name).size(13), text(id.as_str()).size(11)]
                  .spacing(2)
                  .width(Fill),
               button("Remove")
                  .on_press(Message::ToggleInstanceSelectedMod(id.clone()))
                  .style(danger_btn_style),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
         )
         .padding(8)
         .style(card_style)
         .into()
      })
      .collect()
}

fn mod_picker_modal(state: &InstancesView) -> Element<'_, Message> {
   let mut mods = filtered_mods(state);
   sort_mods(&mut mods, &state.mod_sort_by, state.mod_sort_desc);
   let total_pages = mods.len().div_ceil(MOD_PICKER_PAGE_SIZE).max(1);
   let page = state.mod_page.min(total_pages.saturating_sub(1));
   let start = page * MOD_PICKER_PAGE_SIZE;
   let end = (start + MOD_PICKER_PAGE_SIZE).min(mods.len());

   let rows: Vec<Element<'_, Message>> = mods[start..end]
      .iter()
      .map(|m| {
         let id = mod_key(m);
         let selected = state.selected_mod_ids.contains(&id);
         let name = m.name.clone().unwrap_or_else(|| id.clone());
         let summary = m.summary.clone().unwrap_or_default();
         container(
            row![
               column![
                  text(name).size(14),
                  text(summary).size(12).color(Color {
                     r: 0.65,
                     g: 0.65,
                     b: 0.65,
                     a: 1.0
                  }),
                  text(format!("{} downloads", m.downloads)).size(11),
               ]
               .spacing(3)
               .width(Fill),
               button(if selected { "Remove" } else { "Add to Instance" })
                  .on_press(Message::ToggleInstanceSelectedMod(id))
                  .style(if selected {
                     danger_btn_style
                  } else {
                     primary_btn_style
                  }),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
         )
         .padding(10)
         .style(card_style)
         .into()
      })
      .collect();

   let sorts = [SortBy::Downloads, SortBy::Follows, SortBy::Trending, SortBy::Name];
   let sort_controls = row(
      sorts
         .into_iter()
         .map(|s| picker_sort_btn(s, state))
         .collect::<Vec<_>>(),
   )
   .spacing(4)
   .align_y(Alignment::Center);

   container(
      column![
         row![
            text("Browse Mods for Instance").size(18).width(Fill),
            button("Done")
               .on_press(Message::CloseInstanceModPicker)
               .style(primary_btn_style),
         ]
         .spacing(8)
         .align_y(Alignment::Center),
         text_input("Search mods...", &state.mod_search).on_input(Message::InstanceModSearchChanged),
         sort_controls,
         scrollable(Column::with_children(rows).spacing(6)).height(Length::Fixed(360.0)),
         row![
            button("Prev")
               .on_press_maybe(if page > 0 {
                  Some(Message::InstanceModPrevPage)
               } else {
                  None
               })
               .style(ghost_btn_style),
            text(format!("Page {} of {}", page + 1, total_pages)).size(12),
            button("Next")
               .on_press_maybe(if page + 1 < total_pages {
                  Some(Message::InstanceModNextPage)
               } else {
                  None
               })
               .style(ghost_btn_style),
            text(format!("{} selected", state.selected_mod_ids.len())).size(12),
         ]
         .spacing(8)
         .align_y(Alignment::Center),
      ]
      .spacing(10),
   )
   .padding(16)
   .style(card_style)
   .into()
}

fn picker_sort_btn(sort: SortBy, state: &InstancesView) -> Element<'_, Message> {
   let active = sort == state.mod_sort_by;
   let label = if active {
      format!(
         "{}{}",
         sort.label(),
         if state.mod_sort_desc { " ↓" } else { " ↑" }
      )
   } else {
      sort.label().to_string()
   };
   button(text(label).size(12))
      .on_press(if active {
         Message::InstanceModSortToggle
      } else {
         Message::InstanceModSortChanged(sort)
      })
      .style(if active { active_tab_style } else { ghost_btn_style })
      .into()
}

fn filtered_mods(state: &InstancesView) -> Vec<ModApi> {
   let q = state.mod_search.to_lowercase();
   state
      .available_mods
      .iter()
      .filter(|m| {
         if q.is_empty() {
            true
         } else {
            let name = m.name.as_deref().unwrap_or("");
            name.to_lowercase().contains(&q) || m.mod_id_strs.iter().any(|id| id.to_lowercase().contains(&q))
         }
      })
      .cloned()
      .collect()
}

fn sort_mods(mods: &mut [ModApi], sort: &SortBy, desc: bool) {
   match sort {
      SortBy::Downloads => mods.sort_by(|a, b| ord(a.downloads, b.downloads, desc)),
      SortBy::Follows => mods.sort_by(|a, b| ord(a.follows, b.follows, desc)),
      SortBy::Trending => mods.sort_by(|a, b| ord(a.trending_points, b.trending_points, desc)),
      SortBy::Name => mods.sort_by(|a, b| {
         let an = a.name.as_deref().unwrap_or("");
         let bn = b.name.as_deref().unwrap_or("");
         if desc { bn.cmp(an) } else { an.cmp(bn) }
      }),
   }
}

fn ord(a: i64, b: i64, desc: bool) -> std::cmp::Ordering {
   if desc { b.cmp(&a) } else { a.cmp(&b) }
}

fn mod_key(m: &ModApi) -> String {
   m.mod_id_strs
      .first()
      .cloned()
      .unwrap_or_else(|| m.mod_id.to_string())
}
