use crate::{
    gui::{
        popup::{PopupWindow, WindowMeta, GuiError},
        {ChannelsGuiThread, ChannelsSearchThread}
    },
    mac_address::validation::text_is_valid_mac,
    unifi::search::{UnifiSearchInfo, find_unifi_device}
};
use std::thread;


#[derive(Debug, Clone, PartialEq)]
enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}


pub(crate) struct GuiApp<'a> {
    font_size_enum: FontSize,
    unifi_search_info: UnifiSearchInfo,
    gui_channels: ChannelsGuiThread,
    popup_window_option: Option<PopupWindow<'a>>,
}


impl<'a> Default for GuiApp<'a> {
    fn default() -> Self {
        let font_size_enum = FontSize::Medium;

        // create flume channels to communicate with the background thread
        let (search_info_tx, search_info_rx) = flume::bounded(1);
        let (signal_tx, signal_rx) = flume::bounded(1);
        let (percentage_tx, percentage_rx) = flume::bounded(1);
        let (device_tx, device_rx) = flume::bounded(1);

        // all of the channel pieces for the GUI thread
        let gui_channels = ChannelsGuiThread {
            search_info_tx,
            signal_tx,
            percentage_rx,
            device_rx,
        };

        // all of the channel pieces for the background thread
        let mut search_thread_channels = ChannelsSearchThread {
            search_info_rx,
            signal_rx,
            percentage_tx,
            device_tx,
        };

        let _ = thread::spawn(move || loop {
            let mut search_info = search_thread_channels.search_info_rx.recv().unwrap();
            let unifi_search_result = find_unifi_device(&mut search_info, &mut search_thread_channels);
            search_thread_channels
                .device_tx
                .send(unifi_search_result)
                .unwrap();
        });

        Self {
            font_size_enum,
            unifi_search_info: UnifiSearchInfo::default(),
            gui_channels,
            popup_window_option: None,
        }
    }
}

impl<'a> GuiApp<'a> {
    /// Called once before the first frame.
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        cc.egui_ctx.set_pixels_per_point(1.5);

        Default::default()
    }
}

impl<'a> eframe::App for GuiApp<'a> {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            font_size_enum,
            unifi_search_info,
            gui_channels,
            popup_window_option,
        } = self;

        let UnifiSearchInfo {
            username,
            password,
            server_url,
            mac_to_search,
            accept_invalid_certs,
        } = unifi_search_info;

        egui::CentralPanel::default().show(ctx, |ui| {

            let ui_scale_num = {
                match font_size_enum {
                    FontSize::Small => 1.25,
                    FontSize::Medium => 1.5,
                    FontSize::Large => 1.75,
                    FontSize::ExtraLarge => 2.
                }
            };
            if ctx.pixels_per_point() > ui_scale_num || ctx.pixels_per_point() < ui_scale_num {
                ctx.set_pixels_per_point(ui_scale_num);
            }
            ui.shrink_width_to_current();
            ui.shrink_height_to_current();


            let main_window_size = ui.available_size();

            // create top menu bar with light/dark buttons & hyperlinks
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                    ui.label(" | ");
                    egui::ComboBox::from_id_source("ComboBox #1")
                        .selected_text("Gui Scaling")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(font_size_enum, FontSize::Small, "Small");
                            ui.selectable_value(font_size_enum, FontSize::Medium, "Medium");
                            ui.selectable_value(font_size_enum, FontSize::Large, "Large");
                            ui.selectable_value(font_size_enum, FontSize::ExtraLarge, "Extra Large");
                        });
                });
                ui.add_space(150.);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("Source Code", "https://github.com/Crypto-Spartan/unifi-search-tool");
                    ui.label(" | ");
                    ui.hyperlink_to("License", "https://github.com/Crypto-Spartan/unifi-search-tool/blob/master/LICENSE");
                });

            });

            // title in main window
            ui.vertical_centered(|ui| {
                ui.strong("Enter Unifi Controller Credentials");
            });

            // use of grid for the input fields for formatting/spacing
            egui::Grid::new("some_unique_id #1").num_columns(2).show(ui, |ui| {
                ui.label("Username");
                ui.add(egui::TextEdit::singleline(username).desired_width(f32::INFINITY));
                ui.end_row();

                ui.label("Password");
                ui.add(egui::TextEdit::singleline(password).password(true).desired_width(f32::INFINITY));
                ui.end_row();

                ui.label("Server URL");
                ui.add(egui::TextEdit::singleline(server_url).desired_width(f32::INFINITY));
                ui.end_row();

                ui.label("MAC Address");
                ui.add(egui::TextEdit::singleline(mac_to_search).desired_width(f32::INFINITY));
                ui.end_row();
            });

            ui.checkbox(accept_invalid_certs, "Accept Invalid HTTPS Certificate");

            // add "Search Unifi" button
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    if ui.button("Search Unifi").clicked() {
                        
                        if username.is_empty()
                        || password.is_empty()
                        || server_url.is_empty()
                        || mac_to_search.is_empty() {
                            *popup_window_option = Some(PopupWindow::Error(
                                GuiError::new_standard(
                                    "Required Fields",
                                    Box::from("Username, Password, Server URL, & MAC Address are all required fields.")
                                )
                            ));
                        } else if !text_is_valid_mac(mac_to_search.as_bytes()) {
                            *popup_window_option = Some(PopupWindow::Error(
                                GuiError::new_standard(
                                    "Invalid MAC Address",
                                    Box::from("MAC Address must be formatted like XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX with hexadecimal characters only.")
                                )
                            ));
                        } else {
                            *popup_window_option = Some(PopupWindow::SearchProgress(0.));
                            gui_channels.search_info_tx.send(
                                UnifiSearchInfo {
                                    username: username.to_string(),
                                    password: password.to_string(),
                                    server_url: server_url.to_string(),
                                    mac_to_search: mac_to_search.replace("-", ":").to_lowercase(),
                                    accept_invalid_certs: *accept_invalid_certs
                                }
                            ).unwrap();
                        }
                    }
                });
            });

            if let Some(popup_window) = popup_window_option.clone() {
                let popup_metadata = {
                    let width = main_window_size.x * 0.7;
                    WindowMeta {
                        ctx,
                        width,
                        default_x_pos: (main_window_size.x / 2.) - (width / 2.),
                        default_y_pos: main_window_size.y * 0.15
                    }
                };
                
                match popup_window {
                    PopupWindow::SearchProgress(percentage) => {
                        PopupWindow::render_search_progress(popup_metadata, popup_window_option, percentage, mac_to_search.as_str(), gui_channels);
                    },
                    PopupWindow::SearchResult(unifi_device) => {
                        PopupWindow::render_search_result(popup_metadata, popup_window_option, unifi_device);
                    },
                    PopupWindow::Error(error) => {
                        PopupWindow::render_error(popup_metadata, popup_window_option, error);
                    },
                    PopupWindow::DisplayCancel => {
                        PopupWindow::render_cancel(popup_metadata, popup_window_option, &mut gui_channels.device_rx);
                    }
                }
                
                //popup_window.render_window(ctx, main_window_size);
            }

            // displays a small warning message in the bottom right corner if not built in release mode
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}