pub(crate) mod app;
mod popup;

use crate::unifi::search::{UnifiSearchInfo, UnifiSearchResult};
use flume::{Receiver, Sender};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CancelSignal;

struct ChannelsGuiThread {
    search_info_tx: Sender<UnifiSearchInfo>,
    signal_tx: Sender<CancelSignal>,
    percentage_rx: Receiver<f32>,
    device_rx: Receiver<UnifiSearchResult>,
}

pub(crate) struct ChannelsSearchThread {
    pub(crate) search_info_rx: Receiver<UnifiSearchInfo>,
    pub(crate) signal_rx: Receiver<CancelSignal>,
    pub(crate) percentage_tx: Sender<f32>,
    pub(crate) device_tx: Sender<UnifiSearchResult>,
}
