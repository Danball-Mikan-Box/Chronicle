#![allow(non_snake_case)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod components;
mod export;
mod fs;
mod markdown;
mod model;
mod styles;

#[cfg(not(target_os = "android"))]
fn launch_desktop() {
    let icon = {
        let img = image::load_from_memory(include_bytes!("../Chronicle.png"))
            .expect("Failed to load icon")
            .to_rgba8();
        let (w, h) = img.dimensions();
        dioxus_desktop::tao::window::Icon::from_rgba(img.into_raw(), w, h)
            .expect("Failed to create icon")
    };

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

#[cfg(target_os = "android")]
fn launch_mobile() {
    // Mobile launch does not require custom window configuration.
    dioxus::LaunchBuilder::mobile()
        .launch(app::App);
}

fn main() {
    #[cfg(not(target_os = "android"))]
    launch_desktop();

    #[cfg(target_os = "android")]
    launch_mobile();
}
