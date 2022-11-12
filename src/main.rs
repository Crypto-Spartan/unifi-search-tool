#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod gui;
mod unifi;

use gui::GuiApp;

fn main() {
    let icon = load_icon(r"src\unifi-search.ico");

    let native_options = eframe::NativeOptions{
        initial_window_size: Some(egui::Vec2{x:700., y:180.}),
        max_window_size: Some(egui::Vec2{x:1000., y:170.}),
        min_window_size: Some(egui::Vec2{x:265., y:150.}),
        icon_data: Some(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Unifi Search Tool",
        native_options,
        Box::new(|cc| Box::new(GuiApp::new(cc))),
    );
}


fn load_icon(path: &str) -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}