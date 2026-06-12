#![allow(non_snake_case)]
#![windows_subsystem = "windows"]

mod app;
mod components;
mod fs;
mod markdown;
mod model;
mod styles;

fn main() {
    dioxus::launch(app::App);
}
