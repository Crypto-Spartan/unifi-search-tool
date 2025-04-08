use crate::{
    gui::{CancelSignal, ChannelsGuiThread},
    unifi::{api::UnifiAPIError, devices::UnifiDeviceBasic, search::UnifiSearchResult},
};
use egui::{Id, TextBuffer};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum GuiErrorLevel {
    Info,
    Standard,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct GuiError<'a> {
    title: Cow<'a, str>,
    desc: Box<str>,
    err_lvl: GuiErrorLevel,
}

impl GuiError<'_> {
    pub(super) fn new_info(title: &'static str, desc: Box<str>) -> Self {
        Self {
            title: Cow::Borrowed(title),
            desc,
            err_lvl: GuiErrorLevel::Info,
        }
    }
    pub(super) fn new_standard(title: &'static str, desc: Box<str>) -> Self {
        Self {
            title: Cow::Owned(format!("Error: {}", title)),
            desc,
            err_lvl: GuiErrorLevel::Standard,
        }
    }
    pub(super) fn new_critical(title: &'static str, desc: Box<str>) -> Self {
        Self {
            title: Cow::Owned(format!("Critical Error: {}", title)),
            desc,
            err_lvl: GuiErrorLevel::Critical,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PopupModal<'a> {
    SearchProgress(f32),
    SearchResult(UnifiDeviceBasic),
    Error(GuiError<'a>),
    DisplayCancel,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(super) struct ModalMeta<'a> {
    pub(super) ctx: &'a egui::Context,
    pub(super) width: f32,
    pub(super) default_pos: egui::Pos2,
}

impl<'a> PopupModal<'a> {
    pub(super) fn create_search_progress(
        popup_metadata: ModalMeta,
        popup_modal_option: &mut Option<PopupModal>,
        mut percentage: f32,
        mac_address: &str,
        gui_channels: &mut ChannelsGuiThread,
    ) {
        // get percentage value from channel to update the progress bar
        if let Ok(new_percentage) = gui_channels.percentage_rx.try_recv() {
            *popup_modal_option = Some(PopupModal::SearchProgress(new_percentage));
            percentage = new_percentage;
        }

        egui::Modal::new(Id::new("Search Progress Modal")).show(popup_metadata.ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Running Unifi Search");

                ui.label(format!(
                    "Searching for Unifi device with MAC Address: {}",
                    mac_address
                ));

                // create progress bar
                let progress_bar = {
                    egui::widgets::ProgressBar::new(percentage)
                        .show_percentage()
                        .animate(true)
                };
                ui.add(progress_bar);

                // cancel button
                if ui.button("Cancel").clicked() {
                    gui_channels.signal_tx.send(CancelSignal).unwrap();
                    *popup_modal_option = Some(PopupModal::DisplayCancel);
                }
            });
        });

        // return if canceled
        if *popup_modal_option == Some(PopupModal::DisplayCancel) {
            return;
        }

        // check channel to see if we have a search result
        if let Ok(unifi_search_result) = gui_channels.device_rx.try_recv() {
            match unifi_search_result {
                Ok(unifi_search_option) => match unifi_search_option {
                    Some(unifi_device) => {
                        *popup_modal_option =
                            Some(PopupModal::SearchResult(unifi_device));
                    }
                    None => {
                        *popup_modal_option =
                            Some(PopupModal::Error(GuiError::new_info(
                                "Device Not Found",
                                format!(
                                    "Unable to find device with MAC Address {}",
                                    mac_address
                                )
                                .into_boxed_str(),
                            )));
                    }
                },
                Err(ref unifi_api_error) => {
                    *popup_modal_option = match unifi_api_error {
                        UnifiAPIError::ClientError { source } => {
                            debug_assert!(source.is_builder());
                            Some(PopupModal::Error(GuiError::new_critical(
                                "Reqwest Client Error",
                                format!(
                                    "Unable to Build Unifi Client\n{}\n{}",
                                    unifi_api_error, source
                                )
                                .into_boxed_str(),
                            )))
                        }
                        UnifiAPIError::LoginAuthenticationError { url } => {
                            Some(PopupModal::Error(GuiError::new_standard(
                                "Login Failed",
                                format!("Unable to login to {}\n{}", url, unifi_api_error)
                                    .into_boxed_str(),
                            )))
                        }
                        UnifiAPIError::ReqwestError { source } => {
                            Some(PopupModal::Error(GuiError::new_standard(
                                "Unifi API Error",
                                format!("{}\n{}", unifi_api_error, source).into_boxed_str(),
                            )))
                        }
                        UnifiAPIError::JsonError { source, .. } => {
                            Some(PopupModal::Error(GuiError::new_critical(
                                "Json Parsing Error",
                                format!("{}\n{}", unifi_api_error, source).into_boxed_str(),
                            )))
                        }
                    }
                }
            }
        }
    }

    pub(super) fn create_search_result(
        popup_metadata: ModalMeta,
        popup_modal_option: &mut Option<PopupModal>,
        unifi_device: UnifiDeviceBasic,
    ) {
        let UnifiDeviceBasic {
            mac,
            state,
            adopted,
            device_type,
            device_model,
            gateway_mode,
            name_option,
            device_label_option,
            site,
        } = unifi_device;

        egui::Modal::new(Id::new("Search Result Modal")).show(popup_metadata.ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Unifi Search Result");
                ui.label("Successfully found device!");
            });

            // grid of results, grid allows for spacing/formatting
            egui::Grid::new("Search Result Modal - Grid")
                .num_columns(2)
                .show(ui, |ui| {
                    // add device name to the popup, if it's available
                    if let Some(device_name) = name_option {
                        PopupModal::create_search_result_row(
                            ui, "Device Name:", device_name.as_ref(),
                        );
                    }

                    // add device label to the popup, if it's available
                    // else add the device type & model
                    if let Some(device_label) = device_label_option {
                        PopupModal::create_search_result_row(
                            ui, "Model / SKU / Product:",
                            format!("{} / {}",
                                device_model, device_label
                            ).as_str(),
                        );
                    } else {
                        PopupModal::create_search_result_row(
                            ui, "Device Type / Model:",
                            format!("{} / {}",
                                device_type.to_uppercase(),
                                device_model
                            ),
                        );
                    }

                    // add the name of the Unifi site
                    PopupModal::create_search_result_row(
                        ui, "Unifi Site:", site.as_ref(),
                    );

                    // add the MAC address of the device found
                    PopupModal::create_search_result_row(
                        ui, "MAC Address:", format!("{mac}"),
                    );

                    // add device status; ie if the device is connected, offline, or unknown
                    PopupModal::create_search_result_row(
                        // custom state.as_str implementation
                        ui, "Device Status:", state.as_str(),
                    );

                    // add adoption status if false
                    // it's weird that the controller has info on a device that's not adopted
                    // device status will most likely be `unknown`
                    if !adopted {
                        PopupModal::create_search_result_row(
                            // custom state.as_str implementation
                            ui, "Adopted", "False",
                        );
                    }

                    // add gateway mode if true
                    if gateway_mode.is_some_and(|x| x) {
                        PopupModal::create_search_result_row(
                            // custom state.as_str implementation
                            ui, "Gateway Mode:", "True",
                        );
                    }
                });

            // close button
            ui.vertical_centered(|ui| {
                PopupModal::create_close_button(ui, popup_modal_option);
            });
        });
    }

    #[inline]
    fn create_search_result_row(
        ui: &mut egui::Ui,
        field: &'static str,
        value: impl Into<egui::WidgetText>,
    ) {
        ui.label(field);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(value);
        });
        ui.end_row();
    }

    pub(super) fn create_error(
        popup_metadata: ModalMeta,
        popup_modal_option: &mut Option<PopupModal>,
        error: GuiError,
    ) {
        egui::Modal::new(Id::new("Error Modal")).show(popup_metadata.ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(error.title.as_str());

                // error message
                if error.err_lvl == GuiErrorLevel::Critical {
                    ui.label(&*error.desc);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("Please report this bug to the ");
                        let github_issues_url: &'static str = "https://github.com/Crypto-Spartan/unifi-search-tool/issues";
                        ui.hyperlink_to("Github Issues Page", github_issues_url).on_hover_text(github_issues_url);
                        ui.label(" and include as much information as possible.")
                    });
                } else {
                    ui.label(error.desc.as_ref());
                }

                // close button
                PopupModal::create_close_button(ui, popup_modal_option);
            });
        });
    }

    pub(super) fn create_cancel(
        popup_metadata: ModalMeta,
        popup_modal_option: &mut Option<PopupModal>,
        device_rx: &mut flume::Receiver<UnifiSearchResult>,
    ) {
        egui::Modal::new(Id::new("Cancel Modal")).show(popup_metadata.ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Cancel");
                ui.label("Cancel in progress, please wait...");
            });
        });

        if let Ok(Ok(None)) = device_rx.recv() {
            *popup_modal_option = None;
        }
    }

    #[inline]
    fn create_close_button(ui: &mut egui::Ui, popup_modal_option: &mut Option<PopupModal>) {
        if ui.button("Close").clicked() {
            *popup_modal_option = None;
        }
    }
}
