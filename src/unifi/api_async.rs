#![feature(portable_simd)]
use super::devices::{
    ClientDevice, ClientDeviceActive, UnifiDeviceBasic, /*UnifiDeviceFull,*/ UnifiSite,
};
use reqwest::{
    Client,
    header::REFERER,
    StatusCode,
};
use serde::{de, Deserialize};
use std::{borrow::BorrowMut, rc::Rc};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use zeroize::Zeroize;
use bytes::{Bytes, Buf};
use std::io::Read;


trait UnifiApiResp {
    type Output;
    fn into_data(self) -> Self::Output;
}

macro_rules! impl_UnifiApiResp {
    ($t:ty, $out:ty) => {
        impl UnifiApiResp for $t {
            type Output = $out;
            fn into_data(self) -> Self::Output {
                self.data
            }
        }
    }
}


#[derive(Debug, Clone, Deserialize)]
struct RespMeta {
    #[serde(rename(deserialize = "rc"))]
    result: RespResult,
    msg: Option<Box<str>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RespResult {
    Ok,
    Error,
}

#[derive(Debug, Clone, Deserialize)]
struct UnifiSitesResp {
    meta: RespMeta,
    data: Vec<UnifiSite>,
}
impl_UnifiApiResp!(UnifiSitesResp, Vec<UnifiSite>);
// impl UnifiData for UnifiSitesResp {
//     type Output = Vec<UnifiSite>;

//     fn get_data(self) -> Self::Output {
//         self.data
//     }
// }

#[derive(Debug, Clone, Deserialize)]
struct UnifiDevicesBasicResp {
    meta: RespMeta,
    data: Vec<UnifiDeviceBasic>,
}
impl_UnifiApiResp!(UnifiDevicesBasicResp, Vec<UnifiDeviceBasic>);

// #[derive(Debug, Clone, Deserialize)]
// struct UnifiDevicesFullResp {
//     meta: RespMeta,
//     data: Vec<UnifiDeviceFull>,
// }

#[derive(Debug, Clone, Deserialize)]
struct UnifiClientsAllResp {
    meta: RespMeta,
    data: Vec<ClientDevice>,
}
impl_UnifiApiResp!(UnifiClientsAllResp, Vec<ClientDevice>);

#[derive(Debug, Clone, Deserialize)]
struct UnifiClientsActiveResp {
    meta: RespMeta,
    data: Vec<ClientDeviceActive>,
}
impl_UnifiApiResp!(UnifiClientsActiveResp, Vec<ClientDeviceActive>);

#[derive(Error, Debug)]
pub(crate) enum UnifiAPIError {
    #[error("Error building Tokio runtime")]
    TokioInitError{ source: std::io::Error },
    #[error("Error building reqwest client")]
    ClientError{ source: reqwest::Error },
    #[error("Invalid credentials")]
    LoginAuthenticationError { url: Box<str> },
    #[error("Error communicating with Unifi API, check your URL & try again")]
    ReqwestError{ source: reqwest::Error },
    #[error("Error getting bytes from Response")]
    BytesError{ source: reqwest::Error },
    #[error("Error parsing json from\n{url}")]
    JsonError {
        url: Box<str>,
        source: simd_json::Error,
    },
}

#[derive(Clone)]
pub(crate) struct UnifiClient {
    client: Client,
    server_url: Rc<str>,
    is_logged_in: bool,
}

impl UnifiClient {
    pub(crate) fn new(
        server_url: &str,
        accept_invalid_certs: bool,
    ) -> Result<Self, UnifiAPIError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .danger_accept_invalid_certs(accept_invalid_certs)
            .cookie_store(true)
            .build()
            .map_err(|source| UnifiAPIError::ClientError{ source })?;

        Ok(Self {
            client,
            server_url: Rc::from(server_url),
            is_logged_in: false,
        })
    }

    pub(crate) fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }

    pub(crate) async fn login(
        &mut self,
        username: &mut str,
        password: &mut str,
    ) -> Result<(), UnifiAPIError> {
        let mut login_data: HashMap<&str, &mut str> = HashMap::new();
        login_data.insert("username", username);
        login_data.insert("password", password);

        let url = format!("{}/api/login", self.server_url).into_boxed_str();
        let login_result = self
            .client
            .post(url.as_ref())
            .header(REFERER, "/login")
            .json(&login_data)
            .send()
            .await;

        {
            // zeroize the user entered data for security
            login_data.iter_mut().for_each(|(_, v)| {
                v.zeroize();
            });
            std::mem::drop(login_data);
            password.zeroize();
            username.zeroize();
        };

        let login = {
            let login_response = login_result
                .map_err(|source| UnifiAPIError::ReqwestError {
                    source,
                })?;
            // if controller returns HTTP 400, creds were bad
            if login_response.status() == StatusCode::BAD_REQUEST {
                return Err(UnifiAPIError::LoginAuthenticationError { url });
            }
            login_response
                .error_for_status()
                .map_err(|source| UnifiAPIError::ReqwestError {
                    source,
                })?
        };
        if login.status().is_success() {
            self.is_logged_in = true;
            Ok(())
        } else {
            Err(UnifiAPIError::LoginAuthenticationError { url })
        }
    }

    // async fn api_call(
    //     &mut self,
    //     url: &str,
    // ) -> Result<reqwest::Response, UnifiAPIError> {
    //     let resp = self.client
    //         .get(url)
    //         .send()
    //         .await
    //         .map_err(|source| UnifiAPIError::ReqwestError {
    //             source
    //         })?
    //         .error_for_status()
    //         .map_err(|source| UnifiAPIError::ReqwestError {
    //             source
    //         })?;
    //     Ok(resp)
    // }

    async fn api_call<T: de::DeserializeOwned + UnifiApiResp>(&mut self, url: Box<str>) -> Result<T::Output, UnifiAPIError> {
        let resp = self.client
            .get(url.as_ref())
            .send()
            .await
            .map_err(|source| UnifiAPIError::ReqwestError {
                source
            })?
            .error_for_status()
            .map_err(|source| UnifiAPIError::ReqwestError {
                source
            })?;
        let bytes = &mut resp
            .bytes()
            .await
            .map_err(|source| UnifiAPIError::BytesError {
                source
            })?;
        let struct_json: T = simd_json::serde::from_reader(bytes.reader())
            .map_err(|source| UnifiAPIError::JsonError { url, source })?;
        Ok(struct_json.into_data())
    }

    pub(crate) async fn get_sites(&mut self) -> Result<Vec<UnifiSite>, UnifiAPIError> {
        let url = format!("{}/api/self/sites", self.server_url).into_boxed_str();
        // let resp = self.api_call(&url).await?;
        // let bytes = &mut resp
        //     .bytes()
        //     .await
        //     .map_err(|source| UnifiAPIError::BytesError {
        //         source
        //     })?;
        // let sites: UnifiSitesResp = simd_json::serde::from_reader(bytes.reader())
        //     .map_err(|source| UnifiAPIError::JsonError { url, source })?;
        // Ok(sites.data)
        self.api_call::<UnifiSitesResp>(url).await
    }

    pub(crate) async fn get_site_devices_basic(
        &mut self,
        site_code: Rc<str>,
    ) -> Result<Vec<UnifiDeviceBasic>, UnifiAPIError> {
        let url = format!("{}/api/s/{}/stat/device-basic", self.server_url, site_code).into_boxed_str();
        // let resp = self.api_call(&url)?;
        // let site_unifi_devices_basic: UnifiDevicesBasicResp =
        //     simd_json::serde::from_reader(resp)
        //         .map_err(|source| UnifiAPIError::JsonError { url, source })?;
        // Ok(site_unifi_devices_basic.data)
        self.api_call::<UnifiDevicesBasicResp>(url).await
    }

    // pub(crate) fn get_site_devices_full(
    //     &mut self,
    //     site_code: &str,
    // ) -> Result<Vec<UnifiDeviceFull>, UnifiAPIError> {
    //     let url =
    //         format!("{}/api/s/{}/stat/device", self.server_url, site_code).into_boxed_str();
    //     let resp = self.api_call(&url)?;
    //     let site_unifi_devices_full: UnifiDevicesFullResp = simd_json::serde::from_reader(resp)
    //         .map_err(|source| UnifiAPIError::JsonError { url, source })?;
    //     Ok(site_unifi_devices_full.data)
    // }

    // pub(crate) fn get_site_device_mac(
    //     &mut self,
    //     site_code: &str,
    //     mac: &str,
    // ) -> Result<Vec<UnifiDeviceFull>, UnifiAPIError> {
    //     let url =
    //         format!("{}/api/s/{}/stat/device/{}", self.server_url, site_code, mac).into_boxed_str();
    //     let resp = self.api_call(&url)?;
    //     let site_unifi_device_mac: UnifiDevicesFullResp = simd_json::serde::from_reader(resp)
    //         .map_err(|source| UnifiAPIError::JsonError { url, source })?;
    //     Ok(site_unifi_device_mac.data)
    // }

    pub(crate) async fn get_site_clients_all(
        &mut self,
        site_code: &str,
    ) -> Result<Vec<ClientDevice>, UnifiAPIError> {
        let url = format!("{}/api/s/{}/rest/user", self.server_url, site_code).into_boxed_str();
        // let resp = self.api_call(&url)?;
        // let site_client_devices_all: UnifiClientsAllResp = simd_json::serde::from_reader(resp)
        //     .map_err(|source| UnifiAPIError::JsonError { url, source })?;
        // Ok(site_client_devices_all.data)
        self.api_call::<UnifiClientsAllResp>(url).await
    }

    pub(crate) async fn get_site_clients_active(
        &mut self,
        site_code: &str,
    ) -> Result<Vec<ClientDeviceActive>, UnifiAPIError> {
        let url = format!("{}/api/s/{}/stat/sta", self.server_url, site_code).into_boxed_str();
        // let resp = self.api_call(&url)?;
        // let site_client_devices_active: UnifiClientsActiveResp =
        //     simd_json::serde::from_reader(resp)
        //         .map_err(|source| UnifiAPIError::JsonError { url, source })?;
        // Ok(site_client_devices_active.data)
        self.api_call::<UnifiClientsActiveResp>(url).await
    }
}
