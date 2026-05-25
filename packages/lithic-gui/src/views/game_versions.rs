use iced::widget::{button, column, container, progress_bar, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Fill};
use lithic_core::instance::GameVersionInstall;

use crate::app::Message;
use crate::ops::GameInstallProgress;
use crate::widgets::{card_style, danger_btn_style, ghost_btn_style, primary_btn_style, status_element};

#[derive(Debug, Clone, Default)]
pub struct GameVersionsView {
   pub versions: Vec<GameVersionInstall>,
   pub loading: bool,
   pub installing: bool,
   pub status: Option<String>,
   pub form_id: String,
   pub form_version: String,
   pub form_path: String,
   pub install_id: String,
   pub install_version: String,
   pub install_dir: String,
   pub install_banner: GameInstallProgress,
   pub show_install_logs: bool,
}

pub fn view(state: &GameVersionsView) -> Element<'_, Message> {
   let header = row![
      text("Game Versions").size(22).width(Fill),
      button("Reload")
         .on_press(Message::ReloadGameVersions)
         .style(ghost_btn_style),
   ]
   .spacing(8)
   .align_y(Alignment::Center);

   let install_banner: Element<'_, Message> = if state.install_banner.active || state.install_banner.done {
      let percent_text = state
         .install_banner
         .percent
         .map(|p| format!("{p}%"))
         .unwrap_or_else(|| "Working".to_string());
      let title = if let Some(error) = &state.install_banner.error {
         format!("Installing Vintage Story - Error: {error}")
      } else if state.install_banner.done {
         "Installing Vintage Story - Complete".to_string()
      } else {
         format!(
            "Installing Vintage Story - {} {percent_text}",
            state.install_banner.stage
         )
      };

      let bar_value = state.install_banner.percent.unwrap_or(0) as f32 / 100.0;
      let logs: Element<'_, Message> = if state.show_install_logs {
         let lines: Vec<Element<'_, Message>> = state
            .install_banner
            .logs
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|line| {
               text(line.as_str())
                  .size(11)
                  .color(Color {
                     r: 0.72,
                     g: 0.72,
                     b: 0.72,
                     a: 1.0,
                  })
                  .into()
            })
            .collect();
         column(lines).spacing(3).into()
      } else {
         iced::widget::Space::new().into()
      };

      container(
         column![
            row![
               text(title).size(14).width(Fill),
               button(if state.show_install_logs {
                  "Hide Logs"
               } else {
                  "View Logs"
               })
               .on_press(Message::ToggleGameInstallLogs)
               .style(ghost_btn_style),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            progress_bar(0.0..=1.0, bar_value),
            logs,
         ]
         .spacing(8),
      )
      .padding(12)
      .style(card_style)
      .into()
   } else {
      iced::widget::Space::new().into()
   };

   let form = container(
      column![
         text("Attach Existing Version").size(14),
         text_input("id", &state.form_id).on_input(Message::GameVersionFormId),
         text_input("version", &state.form_version).on_input(Message::GameVersionFormVersion),
         row![
            text_input("path", &state.form_path)
               .on_input(Message::GameVersionFormPath)
               .width(Fill),
            button("Browse")
               .on_press(Message::PickGameVersionPath)
               .style(ghost_btn_style),
         ]
         .spacing(6),
         button("Save")
            .on_press(Message::SaveGameVersion)
            .style(primary_btn_style),
      ]
      .spacing(6),
   )
   .padding(12)
   .style(card_style);

   let install_button: Element<'_, Message> = if state.installing {
      button("Installing...").style(ghost_btn_style).into()
   } else {
      button("Download and Install")
         .on_press(Message::InstallGameVersion)
         .style(primary_btn_style)
         .into()
   };

   let install_form = container(
      column![
         text("Install Version").size(14),
         text_input("id (defaults to version)", &state.install_id).on_input(Message::GameVersionInstallId),
         text_input("version", &state.install_version).on_input(Message::GameVersionInstallVersion),
         row![
            text_input("install dir (optional)", &state.install_dir)
               .on_input(Message::GameVersionInstallDir)
               .width(Fill),
            button("Browse")
               .on_press(Message::PickGameVersionInstallDir)
               .style(ghost_btn_style),
         ]
         .spacing(6),
         install_button,
      ]
      .spacing(6),
   )
   .padding(12)
   .style(card_style);

   let mut rows: Vec<Element<'_, Message>> = Vec::new();
   for gv in &state.versions {
      rows.push(
         container(
            row![
               column![
                  text(format!("{} ({})", gv.version, gv.id)).size(14),
                  text(gv.path.as_str()).size(12),
               ]
               .spacing(4)
               .width(Fill),
               button("Delete")
                  .on_press(Message::DeleteGameVersion(gv.id.clone()))
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

   column![
      header,
      {
         if state.install_banner.active || state.install_banner.done {
            iced::widget::Space::new().height(16).into()
         } else {
            status_element(state.status.as_deref())
         }
      },
      install_banner,
      install_form,
      form,
      body
   ]
   .spacing(10)
   .padding(16)
   .height(Fill)
   .into()
}
