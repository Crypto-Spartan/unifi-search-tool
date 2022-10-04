/*fn main() {
    println!("Hello, world!");
}*/

//https://ubntwiki.com/products/software/unifi-controller/api

use std::time::Duration;
use std::collections::HashMap;
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use serde_json::Value;

fn main() {
    let password = rpassword::prompt_password("Unifi Controller password: ").unwrap();
    
    let mut login_data = HashMap::new();
    login_data.insert("username", "admin");
    login_data.insert("password", &password);    

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .build().expect("failed building http client");

    /*let res = client.get("https://unifipro.infopathways.com:8443/proxy/network/status")
        //.json(&login_data)
        //.basic_auth("admin", Some("InfoPathway$"))
        .send().expect("failed client post");
    
    dbg!(res);*/

    /*let test = client.post("https://unifipro.infopathways.com:8443/api/login")
                    .header(REFERER, "/login")
                    .json(&login_data)
                    .basic_auth("admin", Some("InfoPathway$"))
                    .build();

    dbg!(test);*/
    
    let login = client.post("https://unifipro.infopathways.com:8443/api/login")
        .header(REFERER, "/login")
        .json(&login_data)
        .send().expect("failed login");

    if login.status().is_success() {
        println!("login successful");
    } else {
        println!("login unsuccessful");
        return ();
    }

    //dbg!(login);

    let sites_get = client.get("https://unifipro.infopathways.com:8443/api/self/sites")
        .send().expect("failed sites get request");
    let sites_raw = sites_get.text().expect("failed to read result of sites get request");
    
    let sites_serde: Value = serde_json::from_str(&sites_raw).unwrap();
    //let json_object = &serde_object["data"][0];
    //dbg!(&serde_object);

    let unifi_sites = sites_serde["data"].as_array().unwrap();
    //dbg!(unifi_sites);

    for site in unifi_sites {
        //dbg!(site);
        let site_code = site["name"].as_str().unwrap();
        let site_desc = site["desc"].as_str().unwrap();
        let devices_get = client.get(format!("https://unifipro.infopathways.com:8443/api/s/{}/stat/device-basic", site_code))
        .send().expect("failed devices get request");
        let devices_raw = devices_get.text().expect("failed to read result of devices get request");
        let devices_serde: Value = serde_json::from_str(&devices_raw).unwrap();
        //dbg!(&devices_serde);
        let site_devices = &devices_serde["data"].as_array().unwrap();
    }

    
}