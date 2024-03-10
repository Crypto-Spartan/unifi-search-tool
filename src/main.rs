#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod gui;
mod unifi;

use gui::GuiApp;

fn main() {
    const IMAGE: &[u8] = include_bytes!("unifi-search.ico");
    let icon = load_icon(IMAGE);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800., 280.])
            .with_min_inner_size([500., 170.])
            .with_icon(icon),
        ..Default::default()
    };
    let error = eframe::run_native(
        "Unifi Search Tool",
        native_options,
        Box::new(|cc| Box::new(GuiApp::new(cc))),
    );
    if error.is_err() {
        eprintln!("{}", error.unwrap_err());
    }
}

fn load_icon(image_const: &[u8]) -> egui::viewport::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        //let image = image::open(path)
        let image = image::load_from_memory(image_const)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::viewport::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}
