#![allow(non_snake_case)]

mod app;
mod components;
mod markdown;
mod model;
mod styles;

fn main() {
    dioxus::launch(app::App);
}
