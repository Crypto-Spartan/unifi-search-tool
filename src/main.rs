//https://ubntwiki.com/products/software/unifi-controller/api

use std::time::Duration;
use std::collections::HashMap;
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use serde_json::Value;

#[derive(Debug)]
struct UnifiDevice {
    mac: String,
    name: String,
    site: String,
    state: String
}

fn main() {
    let username = "admin";
    let password = rpassword::prompt_password("Unifi Controller password: ").unwrap();
    let base_url = "https://unifipro.infopathways.com:8443";
    //let mac_to_search = "18:e8:29:60:ca:dc";
    let mac_to_search = "24:5a:4c:52:7e:dc";

    if let Some(client) = login_with_client(username, &password, base_url) {
        
        if let Some(d) = find_unifi_device(client, base_url, mac_to_search) {
            dbg!(d);
        } else {
            println!("Device not found");
        }
    
    }
}


fn login_with_client(username: &str, password: &str, base_url: &str) -> Option<Client> {
    let mut login_data = HashMap::new();
    login_data.insert("username", "admin");
    login_data.insert("password", &password);    

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .build().expect("failed building http client");

    let login = client.post(format!("{}/api/login", base_url))
        .header(REFERER, "/login")
        .json(&login_data)
        .send().expect("failed login");

    if login.status().is_success() {
        println!("login successful");
        Some(client)
    } else {
        println!("login unsuccessful");
        None
    }
}


fn find_unifi_device(client: Client, base_url: &str, mac_to_search: &str) -> Option<UnifiDevice> {
    
    let sites_get = client.get(format!("{}/api/self/sites", base_url))
        .send().expect("failed sites get request");
    let sites_raw = sites_get.text().expect("failed to read result of sites get request");
    let sites_serde: Value = serde_json::from_str(&sites_raw).unwrap();
    let unifi_sites = sites_serde["data"].as_array().unwrap();

    /*for site in unifi_sites {
        dbg!(site);
    }*/
    //panic!("panic!");
    

    for site in unifi_sites {
        let site_code = site["name"].as_str().unwrap();
        let site_desc = site["desc"].as_str().unwrap();
        
        let devices_get = client.get(format!("{}/api/s/{}/stat/device-basic", base_url, site_code))
        .send().expect("failed devices get request");
        let devices_raw = devices_get.text().expect("failed to read result of devices get request");
        let devices_serde: Value = serde_json::from_str(&devices_raw).unwrap();
        let site_devices = &devices_serde["data"].as_array().unwrap();

        if "k0aikan6" == site_code {
            dbg!(&devices_raw);
            dbg!(&devices_serde);
            dbg!(&site_devices);
        }
        
        let mut state: String;
        
        for device in site_devices.into_iter() {
            if "k0aikan6" == site_code {
                dbg!(&device);
            }
            
            if let Value::String(mac) = &device["mac"] {
                if mac_to_search == mac.to_lowercase() {                    
                    
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
                                mac: mac.to_string(),
                                name: name.to_string(),
                                site: site_desc.to_string(),
                                state
                            }
                        )
                    } else {
                        dbg!("FLAG");
                    }
                }
            }
        }
    }
    return None
}