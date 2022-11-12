use std::{thread, time};
use crate::unifi::{UnifiSearchInfo, UnifiDevice, UnifiSearchStatus, DeviceLabel, run_unifi_search};
use flume::{Sender, Receiver};
use fancy_regex::Regex;

#[derive(Debug, Clone)]
struct GuiErrorInfo {
    title: String,
    desc: String,
    err_type: GuiErrorType
}

#[derive(Debug, Clone)]
enum GuiErrorType {
    Critical(String),
    Standard,
    Info
}

impl GuiErrorInfo {
    fn new_critical<A: AsRef<str>, S: AsRef<str>, T: AsRef<str>>(err_code: A, title: S, desc: T) -> Self {
        Self {
            title: title.as_ref().to_string(),
            desc: desc.as_ref().to_string(),
            err_type: GuiErrorType::Critical(err_code.as_ref().to_string())
        }
    }

    fn new_standard<S: AsRef<str>, T: AsRef<str>>(title: S, desc: T) -> Self {
        Self {
            title: title.as_ref().to_string(),
            desc: desc.as_ref().to_string(),
            err_type: GuiErrorType::Standard
        }
    }

    fn new_info<S: AsRef<str>, T: AsRef<str>>(title: S, desc: T) -> Self {
        Self {
            title: title.as_ref().to_string(),
            desc: desc.as_ref().to_string(),
            err_type: GuiErrorType::Info
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum ThreadSignal {
    Proceed,
    Stop
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
    mac_addr_regex: Regex,
    unifi_search_info: UnifiSearchInfo,
    run_search: bool,
    percentage: f32,
    channels_for_gui: ChannelsForGuiThread,
    display_result: bool,
    search_status_option: Option<UnifiSearchStatus>,
    display_error: bool,
    error_info_option: Option<GuiErrorInfo>
}

impl Default for GuiApp {
    fn default() -> Self {
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
            mac_addr_regex,
            unifi_search_info: UnifiSearchInfo::default(),
            run_search: false,
            percentage: 0.0,
            channels_for_gui,
            display_result: false,
            search_status_option: None,
            display_error: false,
            error_info_option: None
        }
    }
}

impl GuiApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        Default::default()
    }
}

impl eframe::App for GuiApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            mac_addr_regex,
            unifi_search_info,
            run_search,
            percentage,
            channels_for_gui,
            display_result,
            search_status_option,
            display_error,
            error_info_option
        } = self;
        let UnifiSearchInfo { username, password, server_url, mac_address } = unifi_search_info;

        egui::CentralPanel::default().show(ctx, |ui| {
            let main_window_size = ui.available_size();

            // create top menu bar with light/dark buttons & hyperlinks
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
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

            // add "Search Unifi" button
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    if ui.button("Search Unifi").clicked() {

                        if username.len() == 0 
                        || password.len() == 0
                        || server_url.len() == 0
                        || mac_address.len() == 0 {
                            *display_error = true;
                            *error_info_option = Some(
                                GuiErrorInfo::new_standard("Required Fields", "Username, Password, Server URL, & MAC Address are required fields.")
                                //GuiErrorInfo::new_critical("0002", "Search Status", "Error loading search status result")
                            );
                        } else if !mac_addr_regex.is_match(&mac_address).unwrap_or(false) {
                            *display_error = true;
                            *error_info_option = Some(
                                GuiErrorInfo::new_standard("Invalid MAC Address", "MAC Address must be formatted like XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX with hexadecimal characters only.")
                            );
                        } else {
                            *percentage = 0.;
                            *run_search = true;
                            channels_for_gui.search_info_tx.send(
                                UnifiSearchInfo {
                                    username: username.to_string(),
                                    password: password.to_string(),
                                    server_url: server_url.to_string(),
                                    mac_address: mac_address.to_lowercase()
                                }
                            ).unwrap();
                        }
                    }
                });
            });
            
            // spawn new internal window to show search progress
            if *run_search {
                let width = main_window_size.x*0.7;
                let default_x_pos = (main_window_size.x/2.) - (width/2.);
                let default_y_pos = main_window_size.y*0.25;

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
                        if let Ok(v) = channels_for_gui.percentage_rx.try_recv() {
                            *percentage = v;
                        }
                        if let Ok(v) = channels_for_gui.device_rx.try_recv() {
                            *search_status_option = Some(v);
                            *run_search = false;
                            *display_result = true;
                        }

                        // create progress bar
                        let progress_bar = egui::widgets::ProgressBar::new(*percentage)
                            .show_percentage()
                            .animate(true);
                        ui.add(progress_bar);

                        // cancel button
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                if ui.button("Cancel").clicked() {
                                    *run_search = false;
                                    channels_for_gui.signal_tx.send(ThreadSignal::Stop).unwrap();
                                } else {
                                    let _ = channels_for_gui.signal_tx.try_send(ThreadSignal::Proceed);
                                }
                            });
                        });
                    });
            }

            // spawn new internal window to show search results
            if *display_result {
                let width = main_window_size.x*0.7;
                let default_x_pos = (main_window_size.x/2.) - (width/2.);
                let default_y_pos = main_window_size.y*0.25;


                egui::Window::new("Unifi Search Result")
                    .resizable(false)
                    .collapsible(false)
                    .default_width(width)
                    .default_pos((default_x_pos, default_y_pos))
                    .show(ctx, |ui| {

                        if let Some(search_status) = search_status_option.clone() {
                            match search_status {
                                UnifiSearchStatus::device_found(d) => {
                                    let UnifiDevice { mac_found, device_label, site, state } = d;
                                    ui.horizontal(|ui| {
                                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                            ui.label("Successfully found device!");
                                        });
                                    });
                                    
                                    // grid of results, grid allows for spacing/formatting
                                    egui::Grid::new("some_unique_id #2").num_columns(2).show(ui, |ui| {
                                        
                                        // set the name/label of the device if a name wasn't found in the controller
                                        let gui_label;
                                        let device_name;
                                        match device_label {
                                            DeviceLabel::name(s) => {
                                                gui_label = "Device Name:";
                                                device_name = s;
                                            },
                                            DeviceLabel::model(s) => {
                                                gui_label = "Device Type / Model:";
                                                device_name = s;
                                            }
                                        }
                                        
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
                                    });
                                },
                                UnifiSearchStatus::device_not_found => {
                                    // set error title & desc for device not found
                                    *display_error = true;
                                    *display_result = false;
                                    *error_info_option = Some(
                                        GuiErrorInfo::new_info("Device Not Found", format!("Unable to find device with MAC Address {}", mac_address))
                                    );
                                },
                                UnifiSearchStatus::login_error => {
                                    // set error title & desc for failed login
                                    *display_error = true;
                                    *display_result = false;
                                    *error_info_option = Some(
                                        GuiErrorInfo::new_standard("Login Failed", format!("Unable to login to {}", server_url))
                                    );
                                }
                            }                            
                        } else {
                            // by the time the if statement above is run, `search_status_option` should never be set to `None`, but this is here just in case
                            *display_error = true;
                            *display_result = false;
                            *error_info_option = Some(
                                GuiErrorInfo::new_critical("0002", "Search Status", "Error loading search status result")
                            );
                        }

                        // close button for Unifi Search Result window
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                                if ui.button("Close").clicked() {
                                    *search_status_option = None;
                                    *display_result = false;
                                }
                            });
                        });
                    });
            }

            // spawn new internal window to show errors
            if *display_error {
                let width = main_window_size.x*0.7;
                let default_x_pos = (main_window_size.x/2.) - (width/2.);
                let default_y_pos = main_window_size.y*0.25;

                let mut include_error_code = false;
                let mut include_github_link = false;
                let mut error_code = "".to_string();
                let full_error_title;
                let error_message;

                if let Some(error_info) = error_info_option {

                    match &error_info.err_type {
                        GuiErrorType::Critical(err_code) => {
                            full_error_title = format!("Critical Error: {}", &error_info.title);
                            include_error_code = true;
                            error_code = err_code.to_string();
                            error_message = error_info.desc.to_string();
                            include_github_link = true;
                        },
                        GuiErrorType::Standard => {
                            full_error_title = format!("Error: {}", error_info.title);
                            error_message = error_info.desc.to_string();
                        },
                        GuiErrorType::Info => {
                            full_error_title = error_info.title.to_string();
                            error_message = error_info.desc.to_string();
                        },
                    }

                } else {
                    full_error_title = "Critical Error: Unable To Load Error".to_string();
                    include_error_code = true;
                    error_code = "0001".to_string();
                    error_message = "Error displaying previous error".to_string();
                    include_github_link = true;
                }

                egui::Window::new(full_error_title)
                    .resizable(false)
                    .collapsible(false)
                    .default_width(width)
                    .default_pos((default_x_pos, default_y_pos))
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {

                            if include_error_code {
                                // display error code
                                ui.label(format!("Error Code: {}", error_code));
                            }

                            // error message
                            ui.horizontal(|ui| {
                                if include_github_link {
                                    ui.label(format!("{}, please report this bug to the", error_message));
                                    ui.hyperlink_to("Github Issues Page", "https://github.com/Crypto-Spartan/unifi-search-tool/issues");
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
                                        *display_error = false;
                                    }
                                });
                            });
                        });
                    });

            }
            
            // displays a small warning message in the bottom right corner if not built in release mode
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                egui::warn_if_debug_build(ui);
            });            
        });
    }
}