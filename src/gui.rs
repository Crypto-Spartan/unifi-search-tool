use std::thread;
use crate::unifi::{UnifiSearchInfo, UnifiDeviceFound, UnifiSearchStatus, UnifiSearchError, DeviceLabel, run_unifi_search};
use flume::{Sender, Receiver};
use fancy_regex::Regex;

#[derive(Debug, Clone, PartialEq)]
enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge
}

#[derive(Debug, Clone, PartialEq)]
struct GuiErrorData {
    title: String,
    desc: String
}

#[derive(Debug, Clone, PartialEq)]
enum GuiError {
    Critical(GuiErrorData),
    Standard(GuiErrorData),
    Info(GuiErrorData)
}

impl GuiError {
    fn new_info(title: String, desc: String) -> Self {
        Self::Info(
            GuiErrorData { title, desc }
        )
    }
    fn new_standard(title: String, desc: String) -> Self {
        Self::Standard(
            GuiErrorData { title, desc }
        )
    }
    fn new_critical(title: String, desc: String) -> Self {
        Self::Critical(
            GuiErrorData { title, desc }
        )
    }
}


#[derive(Debug, PartialEq)]
pub enum ThreadSignal {
    CancelSearch
}

#[derive(Debug, PartialEq)]
enum PopupWindow {
    DisplaySearchProgress(f32),
    DisplaySearchResult(UnifiDeviceFound),
    DisplayError(GuiError),
    DisplayCancel,
    None
}

struct ChannelsForGuiThread {
    search_info_tx: Sender<UnifiSearchInfo>,
    signal_tx: Sender<ThreadSignal>,
    percentage_rx: Receiver<f32>,
    device_rx: Receiver<UnifiSearchStatus>
}

pub struct ChannelsForUnifiThread {
    pub search_info_rx: Receiver<UnifiSearchInfo>,
    pub signal_rx: Receiver<ThreadSignal>,
    pub percentage_tx: Sender<f32>,
    pub device_tx: Sender<UnifiSearchStatus>
}

pub struct GuiApp {
    font_size_enum: FontSize,
    mac_addr_regex: Regex,
    unifi_search_info: UnifiSearchInfo,
    channels_for_gui: ChannelsForGuiThread,
    popup_window: PopupWindow
}

impl Default for GuiApp {
    fn default() -> Self {
        let font_size_enum = FontSize::Medium;

        // create regex to ensure mac addresses are formatted properly
        let mac_addr_regex = Regex::new(r"^(?:\h{2}([-:]))(?:\h{2}\1){4}\h{2}$").unwrap();

        // create flume channels to communicate with the background thread
        let (search_info_tx, search_info_rx) = flume::bounded(1);
        let (signal_tx, signal_rx) = flume::bounded(1);
        let (percentage_tx, percentage_rx) = flume::bounded(1);
        let (device_tx, device_rx) = flume::bounded(1);

        // all of the channel pieces for the GUI thread
        let channels_for_gui = ChannelsForGuiThread {
            search_info_tx,
            signal_tx,
            percentage_rx,
            device_rx
        };

        // all of the channel pieces for the background thread
        let mut channels_for_unifi = ChannelsForUnifiThread {
            search_info_rx,
            signal_rx,
            percentage_tx,
            device_tx
        };

        let _ = thread::spawn(move || {
            loop {
                let mut search_info = channels_for_unifi.search_info_rx.recv().unwrap();
                let unifi_search_status = run_unifi_search(&mut search_info, &mut channels_for_unifi);
                channels_for_unifi.device_tx.send(unifi_search_status).unwrap();
            }
        });

        Self {
            font_size_enum,
            mac_addr_regex,
            unifi_search_info: UnifiSearchInfo::default(),
            channels_for_gui,
            popup_window: PopupWindow::None
        }
    }
}

impl GuiApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        cc.egui_ctx.set_pixels_per_point(1.5);

        Default::default()
    }
}

impl eframe::App for GuiApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            font_size_enum,
            mac_addr_regex,
            unifi_search_info,
            channels_for_gui,
            popup_window
        }  = self;
        let UnifiSearchInfo { username, password, server_url, mac_address, accept_invalid_certs } = unifi_search_info;

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
                ui.add(egui::TextEdit::singleline(mac_address).desired_width(f32::INFINITY));
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
                        || mac_address.is_empty() {
                            *popup_window = PopupWindow::DisplayError(
                                GuiError::new_standard(
                                    "Required Fields".to_string(),
                                    "Username, Password, Server URL, & MAC Address are all required fields.".to_string()
                                )
                            );
                        } else if !mac_addr_regex.is_match(mac_address).unwrap_or(false) {
                            *popup_window = PopupWindow::DisplayError(
                                GuiError::new_standard(
                                    "Invalid MAC Address".to_string(),
                                    "MAC Address must be formatted like XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX with hexadecimal characters only.".to_string()
                                )
                            );
                        } else {
                            *popup_window = PopupWindow::DisplaySearchProgress(0.);
                            channels_for_gui.search_info_tx.send(
                                UnifiSearchInfo {
                                    username: username.to_string(),
                                    password: password.to_string(),
                                    server_url: server_url.to_string(),
                                    mac_address: mac_address.replace("-", ":").to_lowercase(),
                                    accept_invalid_certs: *accept_invalid_certs
                                }
                            ).unwrap();
                        }
                    }
                });
            });

            let width = main_window_size.x*0.7;
            let default_x_pos = (main_window_size.x/2.) - (width/2.);
            //let default_y_pos = main_window_size.y*0.25;
            let default_y_pos = main_window_size.y*0.15;
            match popup_window {
                PopupWindow::DisplaySearchProgress(percentage) => {
                    // create progress bar
                    let progress_bar = egui::widgets::ProgressBar::new(*percentage)
                        .show_percentage()
                        .animate(true);

                    egui::Window::new("Running Unifi Search")
                        .resizable(false)
                        .collapsible(false)
                        .default_width(width)
                        .default_pos((default_x_pos, default_y_pos))
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                    ui.label(format!("Searching for device with MAC Address: {}", mac_address));
                                });
                            });

                            // get percentage value from channel to update the progress bar
                            if let Ok(new_percentage) = channels_for_gui.percentage_rx.try_recv() {
                                *popup_window = PopupWindow::DisplaySearchProgress(new_percentage);
                            }
                            // check channel to see if we have a search result
                            if let Ok(unifi_search_status) = channels_for_gui.device_rx.try_recv() {
                                match unifi_search_status {
                                    UnifiSearchStatus::DeviceFound(unifi_device) => {
                                        *popup_window = PopupWindow::DisplaySearchResult(unifi_device);
                                    },
                                    UnifiSearchStatus::DeviceNotFound => {
                                        *popup_window = PopupWindow::DisplayError(
                                            GuiError::new_info(
                                                "Device Not Found".to_string(),
                                                format!("Unable to find device with MAC Address {}", mac_address)
                                            )
                                        );
                                    },
                                    UnifiSearchStatus::Cancelled => {
                                        *popup_window = PopupWindow::None;
                                    },
                                    UnifiSearchStatus::Error(search_error) => {
                                        match search_error {
                                            UnifiSearchError::Login(error_code) => {
                                                *popup_window = PopupWindow::DisplayError(
                                                    GuiError::new_standard(
                                                        format!("Error {}: Login Failed", error_code),
                                                        format!("Unable to login to {}", server_url)
                                                    )
                                                );
                                            },
                                            UnifiSearchError::APINetwork(error_code) => {
                                                *popup_window = PopupWindow::DisplayError(
                                                    GuiError::new_standard(
                                                        format!("Error {}: Network Error", error_code),
                                                        format!("Unable to reach {}", server_url)
                                                    )
                                                );
                                            },
                                            UnifiSearchError::APIParsing(error_code) => {
                                                *popup_window = PopupWindow::DisplayError(
                                                    GuiError::new_critical(
                                                        format!("Error {}: API Parsing Error", error_code),
                                                        "Error parsing API data".to_string()
                                                    )
                                                );
                                            }
                                        }
                                    }
                                }
                            }

                            ui.add(progress_bar);

                            // cancel button
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                    if ui.button("Cancel").clicked() {
                                        channels_for_gui.signal_tx.send(ThreadSignal::CancelSearch).unwrap();
                                        *popup_window = PopupWindow::DisplayCancel;
                                    }
                                });
                            });
                        });
                },
                PopupWindow::DisplaySearchResult(unifi_device) => {
                    let UnifiDeviceFound { mac_found, device_label, site, state, adopted } = unifi_device.clone();
                    
                    // set the name/label of the device if a name wasn't found in the controller
                    let gui_label;
                    let device_name;
                    match device_label {
                        DeviceLabel::Name(s) => {
                            gui_label = "Device Name:";
                            device_name = s;
                        },
                        DeviceLabel::Model(s) => {
                            gui_label = "Device Type / Model:";
                            device_name = s;
                        }
                    }

                    egui::Window::new("Unifi Search Result")
                        .resizable(false)
                        .collapsible(false)
                        .default_width(width)
                        .default_pos((default_x_pos, default_y_pos))
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                    ui.label("Successfully found device!");
                                });
                            });
                            
                            // grid of results, grid allows for spacing/formatting
                            egui::Grid::new("some_unique_id #2").num_columns(2).show(ui, |ui| {
                                
                                // apply device name/label to the GUI
                                ui.label(gui_label);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    ui.label(device_name);
                                });
                                ui.end_row();
                                
                                // display the name of the Unifi site
                                ui.label("Unifi Site:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    ui.label(site);
                                });
                                ui.end_row();

                                // display the MAC address of the device found
                                ui.label("MAC Address:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    ui.label(mac_found);
                                });
                                ui.end_row();

                                // show if the device is connected, offline, or unknown
                                ui.label("Device Status:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    ui.label(state);
                                });
                                ui.end_row();

                                // show if the device is adopted to the controller
                                ui.label("Adopted:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    ui.label(format!("{}", adopted));
                                });
                                ui.end_row();
                            });

                            // close button for Unifi Search Result window
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                    if ui.button("Close").clicked() {
                                        *popup_window = PopupWindow::None;
                                    }
                                });
                            });
                        });
                },
                PopupWindow::DisplayError(error) => {
                    //let /*mut*/ include_error_code = false;
                    //let /*mut*/ error_code = "".to_string();
                    let error_title: String;
                    let error_message: String;
                    let mut include_github_link = false;

                    match error {
                        GuiError::Info(error_data) => {
                            //let check = *error_data;
                            error_title = error_data.title.to_string();
                            error_message = error_data.desc.to_string();
                        },
                        GuiError::Standard(error_data) => {
                            error_title = format!("Error: {}", &error_data.title);
                            error_message = error_data.desc.to_string();
                        },
                        GuiError::Critical(error_data) => {
                            error_title = format!("Critical Error: {}", error_data.title);
                            error_message = error_data.desc.to_string();
                            include_github_link = true;
                        }
                    }

                    egui::Window::new(error_title)
                        .resizable(false)
                        .collapsible(false)
                        .default_width(width)
                        .default_pos((default_x_pos, default_y_pos))
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {

                                // if include_error_code {
                                //     // display error code
                                //     ui.label(format!("Error Code: {}", error_code));
                                // }

                                // error message
                                ui.horizontal(|ui| {
                                    if include_github_link {
                                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                            ui.label(error_message);
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                ui.label("Please report this bug to the ");
                                                ui.hyperlink_to("Github Issues Page", "https://github.com/Crypto-Spartan/unifi-search-tool/issues");
                                                ui.label(" and include as much information as possible.")
                                            });
                                        });
                                    } else {
                                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                            ui.label(error_message);
                                        });
                                    }
                                });
                                
                                // close button
                                ui.horizontal(|ui| {
                                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::BottomUp), |ui| {
                                        if ui.button("Close").clicked() {
                                            *popup_window = PopupWindow::None;
                                        }
                                    });
                                });
                            });
                        });
                },
                PopupWindow::DisplayCancel => {
                    egui::Window::new("Cancel")
                        .resizable(false)
                        .collapsible(false)
                        .default_width(width)
                        .default_pos((default_x_pos, default_y_pos))
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                    ui.label("Cancel in progress, please wait...");
                                });
                            });
                        });
                    
                    if let Ok(UnifiSearchStatus::Cancelled) = channels_for_gui.device_rx.try_recv() {
                        *popup_window = PopupWindow::None;
                    }
                },
                PopupWindow::None => {}
            }
            
            // displays a small warning message in the bottom right corner if not built in release mode
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}