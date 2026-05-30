#![allow(clippy::collapsible_if, clippy::manual_div_ceil)]

mod app;
mod ops;
mod views;
mod widgets;

pub fn run() -> iced::Result {
   app::run()
}
