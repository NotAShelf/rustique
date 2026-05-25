#![allow(clippy::collapsible_if, clippy::manual_div_ceil)]

mod app;
mod ops;
mod views;
mod widgets;

fn main() -> iced::Result {
   app::run()
}
