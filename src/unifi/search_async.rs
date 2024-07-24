use crate::{
    gui::{CancelSignal, ChannelsSearchThread},
    mac_address::validation::text_is_valid_mac,
    unifi::{
        api_async::{UnifiAPIError, UnifiClient}, devices::UnifiDeviceBasic
    },
};
use std::{
    ops::Deref,
    iter,
};
use tokio::{
    runtime::Builder,
    task::{JoinSet, LocalSet},
};
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

async fn get_client_and_login(
    username: &mut str,
    password: &mut str,
    server_url: &str,
    accept_invalid_certs: bool,
) -> Result<UnifiClient, UnifiAPIError> {
    let mut client = UnifiClient::new(server_url, accept_invalid_certs)?;
    let login_result = client.login(username, password).await;

    // zeroize the user entered data for security
    password.zeroize();
    username.zeroize();

    // return any errors with the login
    login_result?;
    // if we make it here, we should be logged in
    //debug_assert!(client.is_logged_in());
    Ok(client)
}

pub fn find_unifi_device(
    search_info: &mut UnifiSearchInfo,
    search_thread_channels: &mut ChannelsSearchThread,
) -> UnifiSearchResult {
    // let UnifiSearchInfo {
    //     username,
    //     password,
    //     ref server_url,
    //     ref mac_to_search,
    //     ref accept_invalid_certs,
    // } = search_info;

    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|source| UnifiAPIError::TokioInitError { source })?;
    let local_set = LocalSet::new();

    local_set.block_on(&runtime, search_unifi_api(search_info, search_thread_channels))


    // runtime.block_on(
    //     async move {
            
    //         local_set.run_until(
    //             .await
    //         )
    //     }.await
    // )
    
}


async fn search_unifi_api(
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

    // dbg!(accept_invalid_certs);
    //let mut client = get_client_and_login(username, password, server_url, *accept_invalid_certs).await?;
    let mut client = get_client_and_login(username, password, server_url, *accept_invalid_certs).await?;

    // check for cancel signal; if channel empty, move on
    if let Ok(v) = search_thread_channels.signal_rx.try_recv() {
        if v == CancelSignal {
            return Ok(None);
        }
    }

    let mac_str = mac_to_search.as_str();
    let unifi_sites = client.get_sites().await?;
    let unifi_sites_len = unifi_sites.len() as f32;
    // dbg!(&unifi_sites);

    let mut join_set = JoinSet::new();
    //let mut handles = Vec::with_capacity(unifi_sites.len());
    let zip_iter = iter::zip(unifi_sites.iter().cloned(), iter::repeat(client.clone()));

    zip_iter.for_each(|(site, mut client)| {
        let _ = join_set.spawn_local(async move {
            let site_devices_res = client.get_site_devices_basic(site.code).await;
            dbg!("#1");
            (site_devices_res, site.desc)
        });
        //handles.push(handle);
    });

    let ret: UnifiSearchResult;
    let mut iter_num: f32 = 0.;

    'loopy: loop {
        let Some(Ok((api_result, site_desc))) = join_set.join_next().await else {
            ret = Ok(None);
            break 'loopy;
        };
        let Ok(site_devices) = api_result else {
            ret = Err(api_result.unwrap_err());
            break 'loopy;
        };
        dbg!("@2");
        // let ;
        // match api_result {
        //     Ok(v) => site_devices = v,
        //     Err(e) => {
        //         ret = Err(e);
        //         break 'loopy;
        //     }
        // }
        // let Ok(site_devices) = api_result else {

        // }
        // match join_set.join_next().await {
        //     Some(Ok(v)) => {},
        //     _ => {
        //         ret = Ok(None);
        //         break 'loopy;
        //     }
        // }
        // let Some(Ok((Ok(site_devices), site_desc))) = join_set.join_next().await else {
        //     ret = Ok(None);
        //     break 'loopy;
        // };

        if let Ok(v) = search_thread_channels.signal_rx.try_recv() {
            if v == CancelSignal {
                ret = Ok(None);
                break 'loopy;
            }
        }
    
        // send percentage of search completion to GUI thread
        let _ = search_thread_channels
            .percentage_tx
            .try_send(iter_num / unifi_sites_len);

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
            unifi_device.site = Box::from(site_desc.deref());
            ret = Ok(Some(unifi_device));
            break 'loopy;
        }

        iter_num += 1.;
    };

    join_set.abort_all();

    ret
}