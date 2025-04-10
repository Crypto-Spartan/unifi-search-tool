use crate::{
    gui::{
        popup::{GuiError, PopupModal, ModalMeta},
        {ChannelsGuiThread, ChannelsSearchThread},
    },
    mac_address::{MacAddress, validation::text_is_valid_mac},
    unifi::search::{find_unifi_device, UnifiSearchInfo},
};
use std::thread;
use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq)]
enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Default, Debug, Clone)]
struct GuiInputFields {
    username_input: String,
    password_input: String,
    server_url_input: String,
    mac_addr_input: String,
    invalid_certs_checked: bool,
    remember_pass_checked: bool,
}

pub(crate) struct GuiApp<'a> {
    font_size_enum: FontSize,
    gui_input_fields: GuiInputFields,
    gui_channels: ChannelsGuiThread,
    popup_modal_option: Option<PopupModal<'a>>,
}

impl eframe::App for GuiApp<'_> {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            font_size_enum,
            gui_input_fields,
            gui_channels,
            popup_modal_option,
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            let ui_scale_num = {
                match font_size_enum {
                    FontSize::Small => 1.25,
                    FontSize::Medium => 1.5,
                    FontSize::Large => 1.75,
                    FontSize::ExtraLarge => 2.,
                }
            };
            if ctx.pixels_per_point() > ui_scale_num || ctx.pixels_per_point() < ui_scale_num {
                ctx.set_pixels_per_point(ui_scale_num);
            }
            ui.shrink_width_to_current();
            ui.shrink_height_to_current();

            GuiApp::create_menu_bar(ui, font_size_enum);
            GuiApp::create_main_window(
                ui,
                gui_input_fields,
                popup_modal_option,
                &mut gui_channels.search_info_tx,
            );

            let main_window_size: egui::Pos2 = {
                let window_coords = ctx.input(|i| i.viewport().inner_rect).unwrap();
                let next_widget_pos = ui.next_widget_position();
                egui::pos2(window_coords.width(), next_widget_pos.y)
            };
            GuiApp::handle_popup_modal(
                ctx,
                popup_modal_option,
                main_window_size,
                &gui_input_fields.mac_addr_input,
                gui_channels,
            );

            // displays a small warning message in the bottom right corner if not built in release mode
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

impl Default for GuiApp<'_> {
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

        // all of the channel pieces for the search thread
        let mut search_thread_channels = ChannelsSearchThread {
            search_info_rx,
            signal_rx,
            percentage_tx,
            device_tx,
        };

        // spawn background thread to do the searching to avoid blocking the GUI thread
        // multiple flume channels used for communication between the gui thread and search thread
        let _ = thread::spawn(move || loop {
            let mut search_info = search_thread_channels.search_info_rx.recv()
                .expect("receiving struct UnifiSearchInfo through channel search_info_rx should be successful");
            let unifi_search_result =
                find_unifi_device(&mut search_info, &mut search_thread_channels);
            search_thread_channels
                .device_tx
                .send(unifi_search_result)
                .expect(
                    "sending unifi_search_result through channel device_tx should be successful",
                );
        });

        Self {
            font_size_enum,
            gui_input_fields: GuiInputFields::default(),
            gui_channels,
            popup_modal_option: None,
        }
    }
}

impl GuiApp<'_> {
    /// Called once before the first frame.
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        cc.egui_ctx.set_pixels_per_point(1.5);

        Default::default()
    }

    fn create_menu_bar(ui: &mut egui::Ui, font_size_enum: &mut FontSize) {
        // create top menu bar with light/dark buttons & hyperlinks
        egui::menu::bar(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                ui.label(" | ");
                egui::ComboBox::from_id_salt("Menu Bar ComboBox - Gui Scaling")
                    .selected_text("Gui Scaling")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(font_size_enum, FontSize::Small, "Small");
                        ui.selectable_value(font_size_enum, FontSize::Medium, "Medium");
                        ui.selectable_value(font_size_enum, FontSize::Large, "Large");
                        ui.selectable_value(font_size_enum, FontSize::ExtraLarge, "Extra Large");
                    });
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let source_code_url: &'static str = "https://github.com/Crypto-Spartan/unifi-search-tool";
                ui.hyperlink_to(
                    "Source Code",
                    source_code_url,
                ).on_hover_text(source_code_url);

                ui.label(" | ");

                let license_url: &'static str = "https://github.com/Crypto-Spartan/unifi-search-tool/blob/master/LICENSE";
                ui.hyperlink_to(
                    "License",
                    license_url,
                ).on_hover_text(license_url);
            });
        });
    }

    fn create_main_window(
        ui: &mut egui::Ui,
        gui_input_fields: &mut GuiInputFields,
        popup_modal_option: &mut Option<PopupModal>,
        search_info_tx: &mut flume::Sender<UnifiSearchInfo>,
    ) {
        let GuiInputFields {
            username_input,
            password_input,
            server_url_input,
            mac_addr_input,
            invalid_certs_checked,
            remember_pass_checked,
        } = gui_input_fields;

        // title in main window
        ui.vertical_centered(|ui| {
            ui.strong("Enter Unifi Controller Credentials");
        });

        // use of grid for the input fields for formatting/spacing
        egui::Grid::new("Main Window Grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Username");
                ui.add(egui::TextEdit::singleline(username_input).desired_width(f32::INFINITY));
                ui.end_row();

                ui.label("Password");
                ui.add(
                    egui::TextEdit::singleline(password_input)
                        .password(true)
                        .desired_width(f32::INFINITY)
                );
                ui.end_row();

                ui.label("Server URL");
                ui.add(
                    egui::TextEdit::singleline(server_url_input)
                        .char_limit(2048)
                        .hint_text("https://<unifi-domain>:8443")
                        .desired_width(f32::INFINITY)
                );
                ui.end_row();

                ui.label("MAC Address");
                ui.add(
                    egui::TextEdit::singleline(mac_addr_input)
                        .char_limit(17)
                        .hint_text("ff:ff:ff:ff:ff:ff")
                        .desired_width(f32::INFINITY)
                );
                ui.end_row();
            });

        let remember_pass_tooltip: &'static str = "Checking this box prevents the password from being cleared between searches. The password will not be saved when the app is closed.";
        ui.checkbox(remember_pass_checked, "Remember Password").on_hover_text(remember_pass_tooltip);
        ui.checkbox(invalid_certs_checked, "Accept Invalid HTTPS Certificate");

        // add "Search Unifi" button
        ui.vertical_centered(|ui| {
            if ui.button("Search Unifi").clicked() {
                GuiApp::handle_button_click(gui_input_fields, popup_modal_option, search_info_tx);
            }
        });
    }

    fn handle_button_click(
        gui_input_fields: &mut GuiInputFields,
        popup_modal_option: &mut Option<PopupModal>,
        search_info_tx: &mut flume::Sender<UnifiSearchInfo>,
    ) {
        // all fields with `ref` are immutable when destructured
        let GuiInputFields {
            ref username_input,
            password_input,
            ref server_url_input,
            ref mac_addr_input,
            ref invalid_certs_checked,
            ref remember_pass_checked,
        } = gui_input_fields;

        // if any fields are empty, display error
        if username_input.is_empty()
        || password_input.is_empty()
        || server_url_input.is_empty()
        || mac_addr_input.is_empty() {
            *popup_modal_option = Some(PopupModal::Error(
                GuiError::new_standard(
                    "Required Fields",
                    Box::from("Username, Password, Server URL, & MAC Address are all required fields.")
                )
            ));
        // if the mac address isn't in a valid format, display error
        } else if !text_is_valid_mac(mac_addr_input.as_bytes()) {
            *popup_modal_option = Some(PopupModal::Error(
                GuiError::new_standard(
                    "Invalid MAC Address",
                    Box::from("MAC Address must be formatted like XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX with hexadecimal characters only.")
                )
            ));
        // other checks passed, run the search
        } else {
            *popup_modal_option = Some(PopupModal::SearchProgress(0.));

            let username = username_input.to_string();
            // don't zeroize the password if remember password checkbox is checked
            // password is always zeroized on the search thread immediately after authentication
            let password = {
                if *remember_pass_checked {
                    password_input.to_string()
                } else {
                    let p = std::mem::take(password_input);
                    password_input.zeroize();
                    p
                }
            };
            let server_url = server_url_input.strip_suffix('/').unwrap_or(server_url_input).to_string();
            let mac_to_search = MacAddress::try_from(mac_addr_input.as_ref())
                .expect("Mac Address validation failed"); // SAFETY: this should never error due to the check above
            let accept_invalid_certs = *invalid_certs_checked;

            search_info_tx.send(
                UnifiSearchInfo {
                    username,
                    password,
                    server_url,
                    mac_to_search,
                    accept_invalid_certs
                }
            ).expect("sending struct UnifiSearchInfo through channel search_info_tx should be successful");
        }
    }

    fn handle_popup_modal(
        ctx: &egui::Context,
        popup_modal_option: &mut Option<PopupModal>,
        main_window_size: egui::Pos2,
        mac_addr_input: &str,
        gui_channels: &mut ChannelsGuiThread,
    ) {
        if popup_modal_option.is_none() {
            return
        }
        let popup_modal = popup_modal_option.clone().unwrap();
        let popup_metadata = {
            let width = main_window_size.x * 0.7;
            let default_pos = egui::pos2(main_window_size.x / 2., main_window_size.y / 2.);
            ModalMeta {
                ctx,
                width,
                default_pos,
            }
        };

        match popup_modal {
            PopupModal::SearchProgress(percentage) => {
                PopupModal::create_search_progress(
                    popup_metadata,
                    popup_modal_option,
                    percentage,
                    mac_addr_input,
                    gui_channels,
                );
            }
            PopupModal::SearchResult(unifi_device) => {
                PopupModal::create_search_result(
                    popup_metadata,
                    popup_modal_option,
                    unifi_device,
                );
            }
            PopupModal::Error(error) => {
                PopupModal::create_error(popup_metadata, popup_modal_option, error);
            }
            PopupModal::DisplayCancel => {
                PopupModal::create_cancel(
                    popup_metadata,
                    popup_modal_option,
                    &mut gui_channels.device_rx,
                );
            }
        }
    }
}
