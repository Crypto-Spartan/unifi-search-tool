use super::devices::{UnifiSite, UnifiDeviceBasic, UnifiDeviceFull, ClientDevice, ClientDeviceActive};
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use zeroize::Zeroize;


#[derive(Debug, Clone, Deserialize)]
struct RespMeta {
    #[serde(rename(deserialize = "rc"))]
    result: RespResult,
    msg: Option<Box<str>>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RespResult {
    Ok,
    Error
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiSitesResp {
    meta: RespMeta,
    data: Vec<UnifiSite>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiDevicesBasicResp {
    meta: RespMeta,
    data: Vec<UnifiDeviceBasic>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiDevicesFullResp {
    meta: RespMeta,
    data: Vec<UnifiDeviceFull>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiClientsAllResp {
    meta: RespMeta,
    data: Vec<ClientDevice>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiClientsActiveResp {
    meta: RespMeta,
    data: Vec<ClientDeviceActive>,
}


#[derive(Error, Debug)]
pub(crate) enum UnifiAPIError {
    #[error("error building reqwest client")]
    ClientError(#[from] reqwest::Error),
    #[error("invalid credentials")]
    LoginAuthenticationError {
        url: Box<str>
    },
    #[error("error communicating with\n{url}")]
    ReqwestError {
        url: Box<str>,
        source: reqwest::Error
    },
    #[error("error parsing json from\n{url}")]
    JsonError {
        url: Box<str>,
        source: simd_json::Error
    }
}

pub(crate) struct UnifiClient<'a> {
    client: Client,
    server_url: &'a str,
    is_logged_in: bool,
}

impl<'a> UnifiClient<'a> {
    pub(crate) fn new(server_url: &'a str, accept_invalid_certs: bool) -> Result<Self, UnifiAPIError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .danger_accept_invalid_certs(accept_invalid_certs)
            .cookie_store(true)
            .build()
            .map_err(UnifiAPIError::ClientError)?;

        Ok(Self {
            client,
            server_url,
            is_logged_in: false,
        })
    }

    pub(crate) fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }

    pub(crate) fn login(&mut self, username: &mut str, password: &mut str) -> Result<(), UnifiAPIError> {
        let mut login_data: HashMap<&str, &mut str> = HashMap::new();
        login_data.insert("username", username);
        login_data.insert("password", password);

        let mut url = format!("{}/api/login", self.server_url).into_boxed_str();
        let login_result = self
            .client
            .post(url.as_ref())
            .header(REFERER, "/login")
            .json(&login_data)
            .send();

        {   // zeroize the user entered data for security
            login_data.iter_mut().for_each(|(_, v)| {
                v.zeroize();
            });
            std::mem::drop(login_data);
            password.zeroize();
            username.zeroize();
        };        

        let login = login_result
            .map_err(|source| UnifiAPIError::ReqwestError{ 
                url: std::mem::take(&mut url),
                source
            })?
            .error_for_status()
            .map_err(|source| UnifiAPIError::ReqwestError{ 
                url: std::mem::take(&mut url),
                source
            })?;

        if login.status().is_success() {
            self.is_logged_in = true;
            Ok(())
        } else {
            Err(UnifiAPIError::LoginAuthenticationError{ url })
        }
    }

    fn api_call(&mut self, url: &mut Box<str>) -> Result<reqwest::blocking::Response, UnifiAPIError> {
        let resp = self.client.get(url.as_ref()).send()
            .map_err(|source| UnifiAPIError::ReqwestError{ 
                url: std::mem::take(url),
                source
            })?
            .error_for_status()
            .map_err(|source| UnifiAPIError::ReqwestError{ 
                url: std::mem::take(url),
                source
            })?;
        Ok(resp)
    }

    pub(crate) fn get_sites(&mut self) -> Result<Vec<UnifiSite>, UnifiAPIError> {
        let mut url = format!("{}/api/self/sites", self.server_url).into_boxed_str();
        let resp = self.api_call(&mut url)?;
        let sites: UnifiSitesResp = simd_json::serde::from_reader(resp)
            .map_err(|source| UnifiAPIError::JsonError{ url, source })?;
        Ok(sites.data)
    }

    pub(crate) fn get_site_devices_basic(&mut self, site_code: &str) -> Result<Vec<UnifiDeviceBasic>, UnifiAPIError> {
        let mut url = format!("{}/api/s/{}/stat/device-basic", self.server_url, site_code).into_boxed_str();
        let resp = self.api_call(&mut url)?;
        let site_unifi_devices_basic: UnifiDevicesBasicResp = simd_json::serde::from_reader(resp)
            .map_err(|source| UnifiAPIError::JsonError{ url, source })?;
        Ok(site_unifi_devices_basic.data)
    }

    pub(crate) fn get_site_devices_full(&mut self, site_code: &str) -> Result<Vec<UnifiDeviceFull>, UnifiAPIError> {
        let mut url = format!("{}/api/s/{}/stat/device", self.server_url, site_code).into_boxed_str();
        let resp = self.api_call(&mut url)?;
        let site_unifi_devices_full: UnifiDevicesFullResp = simd_json::serde::from_reader(resp)
            .map_err(|source| UnifiAPIError::JsonError{ url, source })?;
        Ok(site_unifi_devices_full.data)
    }

    pub(crate) fn get_site_device_mac(&mut self, site_code: &str, mac: &str) -> Result<Vec<UnifiDeviceFull>, UnifiAPIError> {
        let mut url = format!("{}/api/s/{}/stat/device/{}", self.server_url, site_code, mac).into_boxed_str();
        let resp = self.api_call(&mut url)?;
        let site_unifi_device_mac: UnifiDevicesFullResp = simd_json::serde::from_reader(resp)
            .map_err(|source| UnifiAPIError::JsonError{ url, source })?;
        Ok(site_unifi_device_mac.data)
    }

    pub(crate) fn get_site_clients_all(&mut self, site_code: &str) -> Result<Vec<ClientDevice>, UnifiAPIError> {
        let mut url = format!("{}/api/s/{}/rest/user", self.server_url, site_code).into_boxed_str();
        let resp = self.api_call(&mut url)?;
        let site_client_devices_all: UnifiClientsAllResp = simd_json::serde::from_reader(resp)
            .map_err(|source| UnifiAPIError::JsonError{ url, source })?;
        Ok(site_client_devices_all.data)
    }

    pub(crate) fn get_site_clients_active(&mut self, site_code: &str) -> Result<Vec<ClientDeviceActive>, UnifiAPIError> {
        let mut url = format!("{}/api/s/{}/stat/sta", self.server_url, site_code).into_boxed_str();
        let resp = self.api_call(&mut url)?;
        let site_client_devices_active: UnifiClientsActiveResp = simd_json::serde::from_reader(resp)
            .map_err(|source| UnifiAPIError::JsonError{ url, source })?;
        Ok(site_client_devices_active.data)
    }
}
