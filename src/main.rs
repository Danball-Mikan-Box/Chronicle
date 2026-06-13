#![allow(non_snake_case)]
#![windows_subsystem = "windows"]

mod app;
mod components;
mod fs;
mod markdown;
mod model;
mod styles;

fn main() {
    let icon = {
        let img = image::load_from_memory(include_bytes!("../Chronicle.png"))
            .expect("Failed to load icon")
            .to_rgba8();
        let (w, h) = img.dimensions();
        dioxus_desktop::tao::window::Icon::from_rgba(img.into_raw(), w, h)
            .expect("Failed to create icon")
    };

    // IME-only message helper needed for Chrome/Edge embedded
    // webviews when the title bar is removed.

    let cfg = dioxus_desktop::Config::new().with_window(
        dioxus_desktop::WindowBuilder::new()
            .with_title("Chronicle")
            .with_decorations(false)
            .with_window_icon(Some(icon)),
    );

    dioxus::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(app::App);
}
