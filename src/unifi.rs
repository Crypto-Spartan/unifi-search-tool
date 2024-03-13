//https://ubntwiki.com/products/software/unifi-controller/api

use crate::gui::{CancelSignal, ChannelsForUnifiThread};
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use zeroize::Zeroize;

#[derive(Default, Debug, Clone)]
pub struct UnifiSearchInfo {
    pub username: String,
    pub password: String,
    pub server_url: String,
    pub mac_address: String,
    pub accept_invalid_certs: bool,
}

pub type UnifiSearchResult = Result<UnifiSearchStatus, UnifiSearchError>;
type UnifiLoginResult = Result<Client, ErrorCode>;
type ErrorCode = usize;

#[derive(Debug, Clone)]
pub enum UnifiSearchStatus {
    DeviceFound(UnifiDevice),
    DeviceNotFound,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnifiDevice {
    pub mac_found: Box<str>,
    pub device_label: DeviceLabel,
    pub site: Box<str>,
    pub state: &'static str,
    pub adopted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceLabel {
    Name(Box<str>),
    Model(Box<str>),
}

#[derive(Debug, Clone)]
pub struct UnifiSearchError {
    pub code: usize,
    pub kind: UnifiErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnifiErrorKind {
    Login,
    Network,
    APIParsing,
}

impl UnifiSearchError {
    fn new_login(code: usize) -> Self {
        Self {
            code,
            kind: UnifiErrorKind::Login,
        }
    }

    fn new_network(code: usize) -> Self {
        Self {
            code,
            kind: UnifiErrorKind::Network,
        }
    }

    fn new_api_parsing(code: usize) -> Self {
        Self {
            code,
            kind: UnifiErrorKind::APIParsing,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiAllSitesJson {
    data: Vec<UnifiSiteJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiSiteJson {
    #[serde(rename = "name")]
    code: Box<str>,
    desc: Box<str>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiAllSiteDevicesJson {
    data: Vec<UnifiSiteDeviceJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiSiteDeviceJson {
    mac: Box<str>,
    state: usize,
    adopted: bool,
    //disabled: bool,
    #[serde(rename = "type")]
    device_type: Box<str>,
    model: Box<str>,
    name: Option<Box<str>>,
}

pub fn run_unifi_search(
    search_info: &mut UnifiSearchInfo,
    channels_for_unifi: &mut ChannelsForUnifiThread,
) -> UnifiSearchResult {
    let UnifiSearchInfo {
        username,
        password,
        ref server_url,
        ref mac_address,
        ref accept_invalid_certs,
    } = search_info;

    match login_with_client(username, password, server_url, *accept_invalid_certs) {
        Ok(client) => find_unifi_device(client, server_url, mac_address, channels_for_unifi),
        Err(code) => Err(UnifiSearchError::new_login(code)),
    }
}

fn login_with_client(
    username: &mut String,
    password: &mut String,
    base_url: &str,
    accept_invalid_certs: bool,
) -> UnifiLoginResult {
    let mut login_data: HashMap<&str, &str> = HashMap::new();
    login_data.insert("username", username);
    login_data.insert("password", password);

    let Ok(client) = Client::builder()
        .timeout(Duration::from_secs(15))
        .danger_accept_invalid_certs(accept_invalid_certs)
        .cookie_store(true)
        .build()
    else {
        return Err(101);
    };

    let login = client
        .post(format!("{}/api/login", base_url))
        .header(REFERER, "/login")
        .json(&login_data)
        .send()
        .ok();

    // zeroize the user entered data for security
    password.zeroize();
    username.zeroize();

    if let Some(login_status) = login {
        if login_status.status().is_success() {
            Ok(client)
        } else {
            Err(103)
        }
    } else {
        Err(102)
    }
}

fn find_unifi_device(
    client: Client,
    base_url: &str,
    mac_to_search: &str,
    channels_for_unifi: &mut ChannelsForUnifiThread,
) -> UnifiSearchResult {
    // check for cancel signal
    if let Ok(v) = channels_for_unifi.signal_rx.try_recv() {
        if v == CancelSignal {
            return Ok(UnifiSearchStatus::Cancelled);
        }
    }

    let Ok(sites_get) = client.get(format!("{}/api/self/sites", base_url)).send() else {
        return Err(UnifiSearchError::new_network(201));
    };
    let Ok(sites_raw) = sites_get.text() else {
        return Err(UnifiSearchError::new_api_parsing(301));
    };
    let Ok::<UnifiAllSitesJson, _>(sites_parsed) = serde_json::from_str(&sites_raw) else {
        return Err(UnifiSearchError::new_api_parsing(302));
    };
    let unifi_sites = sites_parsed.data;
    let unifi_sites_len = unifi_sites.len() as f32;

    for (iter_num, site) in unifi_sites.iter().enumerate() {
        // check for cancel signal
        if let Ok(v) = channels_for_unifi.signal_rx.try_recv() {
            if v == CancelSignal {
                return Ok(UnifiSearchStatus::Cancelled);
            }
        }
        {
            // send percentage of search completion to GUI thread
            let _ = channels_for_unifi
                .percentage_tx
                .try_send(iter_num as f32 / unifi_sites_len);
        }

        // hit the controller's API to get device info for a specific site
        let Ok(devices_get) = client
            .get(format!(
                "{}/api/s/{}/stat/device-basic",
                base_url, site.code
            ))
            .send()
        else {
            return Err(UnifiSearchError::new_network(202));
        };
        // get the string of the API response
        let Ok(devices_raw) = devices_get.text() else {
            return Err(UnifiSearchError::new_api_parsing(303));
        };
        // parse the API response with serde
        let Ok::<UnifiAllSiteDevicesJson, _>(devices_parsed) = serde_json::from_str(&devices_raw)
        else {
            return Err(UnifiSearchError::new_api_parsing(304));
        };

        let site_devices = devices_parsed.data;
        // loop through the devices found in the site to see if the MAC address matches what we're searching for
        for device in site_devices.into_iter() {
            if mac_to_search == device.mac.to_lowercase() {
                // set percentage to 100%
                {
                    let _ = channels_for_unifi.percentage_tx.try_send(1f32);
                }

                let state = match device.state {
                    0 => "Offline",
                    1 => "Connected",
                    _ => "Unknown",
                };
                let device_label = {
                    match device.name {
                        Some(device_name) => DeviceLabel::Name(device_name),
                        None => DeviceLabel::Model(
                            format!(
                                "{} / {}",
                                device.device_type.to_uppercase(),
                                device.model.to_uppercase()
                            )
                            .into_boxed_str(),
                        ),
                    }
                };

                return Ok(UnifiSearchStatus::DeviceFound(UnifiDevice {
                    mac_found: device.mac.to_lowercase().into_boxed_str(),
                    device_label,
                    site: site.desc.clone(),
                    state,
                    adopted: device.adopted,
                }));
            }
        }
    }
    return Ok(UnifiSearchStatus::DeviceNotFound);
}
