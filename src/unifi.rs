//https://ubntwiki.com/products/software/unifi-controller/api

use std::time::Duration;
use std::collections::HashMap;
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use serde_json::Value;
use flume::{Sender, Receiver};
use zeroize::Zeroize;
use crate::gui::{ThreadSignal, ChannelsForUnifiThread};


#[derive(Debug, Clone)]
pub struct UnifiSearchInfo {
    pub username: String,
    pub password: String,
    pub server_url: String,
    pub mac_address: String
}

impl Default for UnifiSearchInfo {
    fn default() -> Self {
        Self {
            username: "".to_owned(),
            password: "".to_owned(),
            server_url: "".to_owned(),
            mac_address: "".to_owned()
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnifiSearchStatus {
    login_error,
    device_not_found,
    device_found(UnifiDevice)
}


#[derive(Debug, Clone)]
pub struct UnifiDevice {
    pub mac_found: String,
    pub device_label: DeviceLabel,
    pub site: String,
    pub state: String
}

#[derive(Debug, Clone)]
pub enum DeviceLabel {
    name(String),
    model(String)
}

pub fn run_unifi_search(search_info: &mut UnifiSearchInfo, channels_for_unifi: &mut ChannelsForUnifiThread) -> UnifiSearchStatus {
    let UnifiSearchInfo { username, password, server_url, mac_address } = search_info;

    if let Some(client) = login_with_client(username, password, server_url) {

        if let Some(d) = find_unifi_device(client, server_url, mac_address, channels_for_unifi) {
            UnifiSearchStatus::device_found(d)
        } else {
            UnifiSearchStatus::device_not_found
        }
    
    } else {
        UnifiSearchStatus::login_error
    }
}


fn login_with_client(username: &mut String, password: &mut String, base_url: &String) -> Option<Client> {
    let mut login_data = HashMap::new();
    login_data.insert("username", &username);
    login_data.insert("password", &password);    

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .build().expect("failed building http client");

    let login = client.post(format!("{}/api/login", base_url))
        .header(REFERER, "/login")
        .json(&login_data)
        .send()
        .ok();

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


fn find_unifi_device(client: Client, base_url: &str, mac_to_search: &str, channels_for_unifi: &mut ChannelsForUnifiThread) -> Option<UnifiDevice> {
    
    let sites_get = client.get(format!("{}/api/self/sites", base_url))
        .send().expect("failed sites get request");
    let sites_raw = sites_get.text().expect("failed to read result of sites get request");
    let sites_serde: Value = serde_json::from_str(&sites_raw).unwrap();
    let unifi_sites = sites_serde["data"].as_array().unwrap();
    let unifi_sites_len = unifi_sites.len() as f32;

    for (iter_num, site) in unifi_sites.iter().enumerate() {

        if let Ok(v) = channels_for_unifi.signal_rx.try_recv() {
            if v == ThreadSignal::Stop {
                return None
            }
        }
        let _ = channels_for_unifi.percentage_tx.try_send(iter_num as f32 / unifi_sites_len);

        let site_code = site["name"].as_str().unwrap();
        let site_desc = site["desc"].as_str().unwrap();
        
        let devices_get = client.get(format!("{}/api/s/{}/stat/device-basic", base_url, site_code))
            .send().expect("failed devices get request");
        let devices_raw = devices_get.text().expect("failed to read result of devices get request");
        let devices_serde: Value = serde_json::from_str(&devices_raw).unwrap();
        let site_devices = &devices_serde["data"].as_array().unwrap();
        
        let mut state: String;
        
        for device in site_devices.into_iter() {
            if let Value::String(mac_found) = &device["mac"] {
                if mac_to_search == mac_found.to_lowercase() {          
                    let _ = channels_for_unifi.percentage_tx.try_send(1f32);          
                    
                    if let Some(i) = device["state"].as_i64() {
                        if i == 1 {
                            state = String::from("Connected");
                        } else if i == 0 {
                            state = String::from("Offline");
                        } else {
                            state = String::from("Unknown");
                        }
                    } else {
                        state = String::from("Unknown");
                    }

                    if let Value::String(name) = &device["name"] {
                        return Some(
                            UnifiDevice{
                                mac_found: mac_found.to_lowercase(),
                                device_label: DeviceLabel::name(name.to_string()),
                                site: site_desc.to_string(),
                                state
                            }
                        )
                    } else {

                        if let (Value::String(device_type), Value::String(model)) = (&device["type"], &device["model"]) {
                            return Some(
                                UnifiDevice{
                                    mac_found: mac_found.to_lowercase(),
                                    device_label: DeviceLabel::model(format!("{} / {}", device_type.to_uppercase(), model.to_uppercase())),
                                    site: site_desc.to_string(),
                                    state
                                }
                            )
                        }
                    }
                }
            }
        }
    }
    return None
}