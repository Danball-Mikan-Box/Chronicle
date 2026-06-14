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
    let icon = (|| -> Option<dioxus_desktop::tao::window::Icon> {
        let img = image::load_from_memory(include_bytes!("../Chronicle.png")).ok()?;
        let small = img.resize_exact(64, 64, image::imageops::FilterType::Lanczos3);
        let rgba = small.to_rgba8();
        let (w, h) = rgba.dimensions();
        dioxus_desktop::tao::window::Icon::from_rgba(rgba.into_raw(), w, h).ok()
    })();

    let cfg = dioxus_desktop::Config::new().with_window(
        dioxus_desktop::WindowBuilder::new()
            .with_title("Chronicle")
            .with_decorations(false)
            .with_window_icon(icon),
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
