use iced::widget::{button, container, row, text};
use iced::{Alignment, Border, Color, Element, Fill, Length, Theme};

use crate::app::Message;

pub fn card_style(_theme: &Theme) -> iced::widget::container::Style {
   iced::widget::container::Style {
      background: Some(
         Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.04,
         }
         .into(),
      ),
      border: Border {
         color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.09,
         },
         width: 1.0,
         radius: 6.0.into(),
      },
      ..Default::default()
   }
}

pub fn danger_btn_style(_: &Theme, _: iced::widget::button::Status) -> iced::widget::button::Style {
   iced::widget::button::Style {
      background: Some(
         Color {
            r: 0.72,
            g: 0.14,
            b: 0.14,
            a: 1.0,
         }
         .into(),
      ),
      text_color: Color::WHITE,
      border: Border {
         radius: 4.0.into(),
         ..Default::default()
      },
      ..Default::default()
   }
}

pub fn ghost_btn_style(theme: &Theme, _: iced::widget::button::Status) -> iced::widget::button::Style {
   iced::widget::button::Style {
      background: None,
      text_color: theme.extended_palette().background.base.text,
      border: Border {
         color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.15,
         },
         width: 1.0,
         radius: 4.0.into(),
      },
      ..Default::default()
   }
}

pub fn primary_btn_style(_: &Theme, _: iced::widget::button::Status) -> iced::widget::button::Style {
   iced::widget::button::Style {
      background: Some(
         Color {
            r: 0.36,
            g: 0.58,
            b: 0.96,
            a: 1.0,
         }
         .into(),
      ),
      text_color: Color::WHITE,
      border: Border {
         radius: 4.0.into(),
         ..Default::default()
      },
      ..Default::default()
   }
}

pub fn active_tab_style(_: &Theme, _: iced::widget::button::Status) -> iced::widget::button::Style {
   iced::widget::button::Style {
      background: Some(
         Color {
            r: 0.36,
            g: 0.58,
            b: 0.96,
            a: 1.0,
         }
         .into(),
      ),
      text_color: Color::WHITE,
      border: Border {
         radius: 4.0.into(),
         ..Default::default()
      },
      ..Default::default()
   }
}

pub fn status_element<'a>(msg: Option<&'a str>) -> Element<'a, Message> {
   match msg {
      None => iced::widget::Space::new().height(16).into(),
      Some(m) => {
         let low = m.to_lowercase();
         let color = if low.contains("error") || low.contains("fail") {
            Color {
               r: 0.96,
               g: 0.26,
               b: 0.21,
               a: 1.0,
            }
         } else if low.contains("complete")
            || low.contains("install")
            || low.contains("delet")
            || low.contains("enabl")
            || low.contains("disabl")
            || low.contains("export")
            || low.contains("saved")
         {
            Color {
               r: 0.30,
               g: 0.69,
               b: 0.31,
               a: 1.0,
            }
         } else {
            Color {
               r: 0.60,
               g: 0.60,
               b: 0.60,
               a: 1.0,
            }
         };
         text(m).size(13).color(color).into()
      }
   }
}

/// Nav button with a fixed-size accent pip inside the button content.
/// Avoids the `height(Fill)` layout issue of the previous approach.
pub fn nav_button(label: &str, active: bool, msg: Message) -> Element<'_, Message> {
   const ACCENT: Color = Color {
      r: 0.36,
      g: 0.58,
      b: 0.96,
      a: 1.0,
   };
   const TRANSPARENT: Color = Color {
      r: 0.0,
      g: 0.0,
      b: 0.0,
      a: 0.0,
   };

   let pip_color = if active { ACCENT } else { TRANSPARENT };

   let pip = container("")
      .width(3)
      .height(14)
      .style(move |_: &Theme| iced::widget::container::Style {
         background: Some(pip_color.into()),
         border: Border {
            radius: 2.0.into(),
            ..Default::default()
         },
         ..Default::default()
      });

   button(
      row![pip, text(label).size(14)]
         .spacing(10)
         .align_y(Alignment::Center),
   )
   .width(Fill)
   .padding([10, 14])
   .on_press(msg)
   .style(move |theme: &Theme, _| iced::widget::button::Style {
      background: if active {
         Some(
            Color {
               r: 1.0,
               g: 1.0,
               b: 1.0,
               a: 0.08,
            }
            .into(),
         )
      } else {
         None
      },
      text_color: if active {
         Color::WHITE
      } else {
         theme.extended_palette().background.base.text
      },
      border: Border::default(),
      ..Default::default()
   })
   .into()
}

/// Section label, all-caps small text used as form field labels.
pub fn section_label(s: impl Into<String>) -> Element<'static, Message> {
   text(s.into())
      .size(11)
      .color(Color {
         r: 0.55,
         g: 0.55,
         b: 0.55,
         a: 1.0,
      })
      .into()
}

/// A card container with subtle background + border, used to group settings sections.
pub fn section_card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
   container(content.into())
      .padding(16)
      .width(Length::Fill)
      .style(|_: &Theme| iced::widget::container::Style {
         background: Some(
            Color {
               r: 1.0,
               g: 1.0,
               b: 1.0,
               a: 0.04,
            }
            .into(),
         ),
         border: Border {
            color: Color {
               r: 1.0,
               g: 1.0,
               b: 1.0,
               a: 0.09,
            },
            width: 1.0,
            radius: 6.0.into(),
         },
         ..Default::default()
      })
      .into()
}
