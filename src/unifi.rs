//https://ubntwiki.com/products/software/unifi-controller/api

use std::time::Duration;
use std::collections::HashMap;
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use serde::Deserialize;
use serde_json::{Value, Result};
use zeroize::Zeroize;
use crate::gui::{ChannelsForUnifiThread, ThreadSignal};
use std::fs;



#[derive(Debug, Clone)]
pub struct UnifiSearchInfo {
    pub username: String,
    pub password: String,
    pub server_url: String,
    pub mac_address: String,
    pub accept_invalid_certs: bool
}

impl Default for UnifiSearchInfo {
    fn default() -> Self {
        Self {
            username: "".to_owned(),
            password: "".to_owned(),
            server_url: "".to_owned(),
            mac_address: "".to_owned(),
            accept_invalid_certs: false
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnifiSearchStatus {
    DeviceFound(UnifiDeviceFound),
    DeviceNotFound,
    Cancelled,
    Error(UnifiSearchError)
}

pub type ErrorCode = usize;
#[derive(Debug, Clone)]
pub enum UnifiSearchError {
    Login(ErrorCode),
    APINetwork(ErrorCode),
    APIParsing(ErrorCode)
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnifiDeviceFound {
    pub mac_found: String,
    pub device_label: DeviceLabel,
    pub site: String,
    pub state: String,
    pub adopted: bool
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceLabel {
    Name(String),
    Model(String)
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiAllSitesJson {
    data: Vec<UnifiSiteJson>
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiSiteJson {
    #[serde(rename = "name")]
    code: String,
    desc: String
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiAllSiteDevicesJson {
    data: Vec<UnifiSiteDeviceJson>
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiSiteDeviceJson {
    mac: String,
    state: usize,
    adopted: bool,
    //disabled: bool,
    #[serde(rename = "type")]
    device_type: String,
    model: String,
    name: Option<String>
}

pub fn run_unifi_search(search_info: &mut UnifiSearchInfo, channels_for_unifi: &mut ChannelsForUnifiThread) -> UnifiSearchStatus {
    let UnifiSearchInfo { username, password, server_url, mac_address, accept_invalid_certs } = search_info;

    if let Some(client) = login_with_client(username, password, server_url, accept_invalid_certs) {

        find_unifi_device(client, server_url, mac_address, channels_for_unifi)
    
    } else {
        UnifiSearchStatus::Error(UnifiSearchError::Login(101))
    }
}

fn login_with_client(username: &mut String, password: &mut String, base_url: &String, accept_invalid_certs: &bool) -> Option<Client> {
    let mut login_data = HashMap::new();
    login_data.insert("username", &username);
    login_data.insert("password", &password);    

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(*accept_invalid_certs)
        .cookie_store(true)
        .build().expect("failed building http client");

    let login = client.post(format!("{}/api/login", base_url))
        .header(REFERER, "/login")
        .json(&login_data)
        .send()
        .ok();

    // zeroize the user entered data for security
    password.zeroize();
    username.zeroize();

    if let Some(login_status) = login {
        if login_status.status().is_success() {
            Some(client)
        } else {
            None
        }
    } else {
        None
    }
}

fn find_unifi_device(client: Client, base_url: &str, mac_to_search: &str, channels_for_unifi: &mut ChannelsForUnifiThread) -> UnifiSearchStatus /*Option<UnifiDevice>*/ {
    
    // check for cancel signal
    if let Ok(s) = channels_for_unifi.signal_rx.try_recv() {
        if s == ThreadSignal::CancelSearch {
            return UnifiSearchStatus::Cancelled
        }
    }

    let Ok(sites_get) = client.get(format!("{}/api/self/sites", base_url)).send() else {
        return UnifiSearchStatus::Error(UnifiSearchError::APINetwork(201))
    };
    let Ok(sites_raw) = sites_get.text() else {
        return UnifiSearchStatus::Error(UnifiSearchError::APIParsing(301))
    };
    let Ok::<UnifiAllSitesJson, _>(sites_parsed) = serde_json::from_str(&sites_raw) else {
        return UnifiSearchStatus::Error(UnifiSearchError::APIParsing(302))
    };
    let unifi_sites = sites_parsed.data;
    let unifi_sites_len = unifi_sites.len() as f32;
    
    for (iter_num, site) in unifi_sites.iter().enumerate() {
        // check for cancel signal
        if let Ok(v) = channels_for_unifi.signal_rx.try_recv() {
            if v == ThreadSignal::CancelSearch {
                return UnifiSearchStatus::Cancelled
            }
        }
        {   // send percentage of search completion to GUI thread
            let _ = channels_for_unifi.percentage_tx.try_send(iter_num as f32 / unifi_sites_len);
        }

        // hit the controller's API to get device info for a specific site
        let Ok(devices_get) = client.get(format!("{}/api/s/{}/stat/device-basic", base_url, site.code)).send() else {
            return UnifiSearchStatus::Error(UnifiSearchError::APINetwork(202))
        };
        let Ok(devices_raw) = devices_get.text() else {
            return UnifiSearchStatus::Error(UnifiSearchError::APIParsing(303))
        };
        // let s: Result<UnifiAllSiteDevicesJson> = serde_json::from_str(&devices_raw);
        // dbg!(s);
        //dbg!(serde_json::from_str(&devices_raw));
        let Ok::<UnifiAllSiteDevicesJson, _>(devices_parsed) = serde_json::from_str(&devices_raw) else {
            return UnifiSearchStatus::Error(UnifiSearchError::APIParsing(304))
        };

        //let devices_serde: Value = serde_json::from_str(&devices_raw).unwrap();
        let site_devices = devices_parsed.data;
        //let mut state: String;
        // loop through the devices found in the site to see if the MAC address matches what we're searching for
        for device in site_devices.iter() {
            if mac_to_search == device.mac.to_lowercase() {
                // set percentage to 100%
                let _ = channels_for_unifi.percentage_tx.try_send(1f32);

                let state = match device.state {
                    0 => { String::from("Offline") },
                    1 => { String::from("Connected") },
                    _ => { String::from("Unknown") }
                };

                let device_label = {
                    match &device.name {
                        Some(device_name) => {
                            DeviceLabel::Name(device_name.to_string())
                        },
                        None => {
                            DeviceLabel::Model(format!("{} / {}", device.device_type.to_uppercase(), device.model.to_uppercase()))
                        }
                    }
                };

                return UnifiSearchStatus::DeviceFound(
                    UnifiDeviceFound {
                        mac_found: device.mac.to_lowercase(),
                        device_label,
                        site: site.desc.to_string(),
                        state,
                        adopted: device.adopted
                    }
                )
            }
        }
    }
    return UnifiSearchStatus::DeviceNotFound
}