use crate::{
    gui::{CancelSignal, ChannelsSearchThread},
    mac_address::validation::text_is_valid_mac,
    unifi::{
        api::{UnifiAPIError, UnifiClient},
        devices::UnifiDeviceBasic,
    },
};
//use std::time::Duration;
use zeroize::Zeroize;

#[derive(Default, Debug, Clone)]
pub struct UnifiSearchInfo {
    pub username: String,
    pub password: String,
    pub server_url: String,
    pub mac_to_search: String,
    pub accept_invalid_certs: bool,
}

pub type UnifiSearchResult = Result<Option<UnifiDeviceBasic>, UnifiAPIError>;

// #[derive(Debug, Clone)]
// pub(crate) enum UnifiSearchType {
//     NetworkDevice,
//     ClientDevice
// }

fn get_client_and_login<'a>(
    username: &mut str,
    password: &mut str,
    server_url: &'a str,
    accept_invalid_certs: bool,
) -> Result<UnifiClient<'a>, UnifiAPIError> {
    let mut client = UnifiClient::new(server_url, accept_invalid_certs)?;
    let login_result = client.login(username, password);

    // zeroize the user entered data for security
    password.zeroize();
    username.zeroize();

    // return any errors with the login
    login_result?;
    // if we make it here, we should be logged in
    debug_assert!(client.is_logged_in());
    Ok(client)
}

pub fn find_unifi_device(
    search_info: &mut UnifiSearchInfo,
    search_thread_channels: &mut ChannelsSearchThread,
) -> UnifiSearchResult {
    let UnifiSearchInfo {
        username,
        password,
        ref server_url,
        ref mac_to_search,
        ref accept_invalid_certs,
    } = search_info;

    let mut client = get_client_and_login(username, password, server_url, *accept_invalid_certs)?;

    // check for cancel signal; if channel empty, move on
    if let Ok(v) = search_thread_channels.signal_rx.try_recv() {
        if v == CancelSignal {
            return Ok(None);
        }
    }

    let mac_str = mac_to_search.as_str();
    let mut unifi_sites = client.get_sites()?;
    let unifi_sites_len = unifi_sites.len() as f32;
    //dbg!(&unifi_sites);

    for (iter_num, site) in unifi_sites.iter_mut().enumerate() {
        // check for cancel signal each iteration
        if let Ok(v) = search_thread_channels.signal_rx.try_recv() {
            if v == CancelSignal {
                return Ok(None);
            }
        }

        {
            // send percentage of search completion to GUI thread
            let _ = search_thread_channels
                .percentage_tx
                .try_send(iter_num as f32 / unifi_sites_len);
        }

        // get devices from a specific site
        let site_devices = client.get_site_devices_basic(&site.code)?;
        /*if let Some(device) = site_devices.iter().find(|device| !device.adopted) {
            dbg!(&site);
            dbg!(device);
        }*/
        let unifi_device_option = site_devices
            .into_iter()
            .filter(|device| text_is_valid_mac(device.mac.as_bytes()))
            .find(|device| mac_str == device.mac.to_lowercase().as_str());

        if let Some(mut unifi_device) = unifi_device_option {
            {
                // set percentage to 100% since we got a match
                let _ = search_thread_channels.percentage_tx.try_send(1f32);
            }

            unifi_device.create_device_label();
            unifi_device.site = std::mem::take(&mut site.desc);
            return Ok(Some(unifi_device));
        }
    }
    Ok(None)
}
