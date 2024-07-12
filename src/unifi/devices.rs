use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Default, Debug, Clone, Deserialize)]
pub(crate) struct UnifiSite {
    #[serde(rename(deserialize = "name"))]
    pub(crate) code: Box<str>,
    pub(crate) desc: Box<str>,
}

// from https://github.com/Art-of-WiFi/UniFi-API-client/blob/d36a088101e3422e98be1c042afdebaf5f190e8b/src/Client.php#L3379
#[derive(Debug, Clone, Eq, PartialEq, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum DeviceState {
    Offline         = 0,
    Connected       = 1,
    PendingAdoption = 2,
    Updating        = 4,
    Provisioning    = 5,
    Unreachable     = 6,
    Adopting        = 7,
    AdoptionError   = 9,
    AdoptionFailed  = 10,
    Isolated        = 11,
}

impl DeviceState {
    #[inline]
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            DeviceState::Offline         => "Offline",
            DeviceState::Connected       => "Connected",
            DeviceState::PendingAdoption => "Pending Adoption",
            DeviceState::Updating        => "Updating",
            DeviceState::Provisioning    => "Provisioning",
            DeviceState::Unreachable     => "Unreachable",
            DeviceState::Adopting        => "Adopting",
            DeviceState::AdoptionError   => "Adoption Error",
            DeviceState::AdoptionFailed  => "Adoption Failed",
            DeviceState::Isolated        => "Isolated",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub(crate) struct UnifiDeviceBasic {
    pub(crate) mac: Box<str>,
    pub(crate) state: DeviceState,
    pub(crate) adopted: bool,
    #[serde(rename(deserialize = "type"))]
    pub(crate) device_type: Box<str>,
    #[serde(rename(deserialize = "model"))]
    pub(crate) device_model: Box<str>,
    #[serde(rename(deserialize = "in_gateway_mode"))]
    pub(crate) gateway_mode: Option<bool>,
    #[serde(rename(deserialize = "name"))]
    pub(crate) name_option: Option<Box<str>>,
    #[serde(skip_deserializing)]
    pub(crate) device_label_option: Option<&'static str>,
    #[serde(skip_deserializing)]
    pub(crate) site: Box<str>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub(crate) struct UnifiDeviceFull {
    #[serde(flatten)]
    device: UnifiDeviceBasic,
    port_table: Option<Vec<Port>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub(crate) struct Port {
    name: Box<str>,
    ifname: Box<str>,
    mac: Box<str>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub(crate) struct ClientDevice {
    last_ip: Box<str>,
    oui: Box<str>,
    #[serde(with = "ts_seconds")]
    first_seen: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    last_seen: DateTime<Utc>,
    is_wired: bool,
    #[serde(rename(deserialize = "last_connection_network_name"))]
    network_name: Box<str>,
    mac: Box<str>,
    hostname: Box<str>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub(crate) struct ClientDeviceActive {
    #[serde(rename(deserialize = "assoc_time"), with = "ts_seconds")]
    session_start: DateTime<Utc>,
    #[serde(rename(deserialize = "latest_assoc_time"), with = "ts_seconds")]
    session_latest: DateTime<Utc>,
    oui: Box<str>,
    last_ip: Box<str>,
    #[serde(with = "ts_seconds")]
    first_seen: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    last_seen: DateTime<Utc>,
    is_wired: bool,
    #[serde(rename(deserialize = "last_connection_network_name"))]
    network_name: Box<str>,
    mac: Box<str>,
    hostname: Box<str>,
    uptime: usize,
}


impl UnifiDeviceBasic {
    
    #[inline]
    pub(crate) fn create_device_label(&mut self) {
        self.device_label_option = match &*self.device_type {
            "uap" => {
                match self.device_model.as_ref() {
                    "BZ2"     => Some("UAP / Access Point"),
                    "BZ2LR"   => Some("UAP-LR / Access Point Long-Range"),
                    "U2HSR"   => Some("UAP-Outdoor+ / Access Point Outdoor+"),
                    "U2IW"    => Some("UAP-IW / Access Point In-Wall"),
                    "U2L48"   => Some("UAP-LR / Access Point Long-Range"),
                    "U2Lv2"   => Some("UAP-LRv2 / Access Point Long-Range"),
                    "U2M"     => Some("UAP-Mini / Access Point Mini"),
                    "U2O"     => Some("UAP-Outdoor / Access Point Outdoor"),
                    "U2S48"   => Some("UAP / Access Point"),
                    "U2Sv2"   => Some("UAPv2 / Access Point"),
                    "U5O"     => Some("UAP-Outdoor5 / Access Point Outdoor 5"),
                    "U6ENT"   => Some("U6-Enterprise / Access Point WiFi 6 Enterprise"),
                    "U6EXT"   => Some("U6-Extender / Access Point WiFi 6 Extender"),
                    "U6IW"    => Some("U6-IW / Access Point WiFi 6 In-Wall"),
                    "U6M"     => Some("U6-Mesh / Access Point WiFi 6 Mesh"),
                    "U7E"     => Some("UAP-AC / Access Point AC"),
                    "U7EDU"   => Some("UAP-AC-EDU / Access Point AC EDU"),
                    "U7Ev2"   => Some("UAP-AC / Access Point AC"),
                    "U7HD"    => Some("UAP-AC-HD / Access Point AC HD"),
                    "U7IW"    => Some("UAP-AC-IW / Access Point AC In-Wall"),
                    "U7IWP"   => Some("UAP-AC-IW-Pro / Access Point AC In-Wall Pro"),
                    "U7LR"    => Some("UAP-AC-LR / Access Point AC Long-Range"),
                    "U7LT"    => Some("UAP-AC-Lite / Access Point AC Lite"),
                    "U7MP"    => Some("UAP-AC-M-Pro / Access Point AC Mesh Pro"),
                    "U7MSH"   => Some("UAP-AC-M / Access Point AC Mesh"),
                    "U7NHD"   => Some("UAP-nanoHD / Access Point nanoHD"),
                    "U7O"     => Some("UAP-AC-Outdoor / Access Point AC Outdoor"),
                    "U7P"     => Some("UAP-AC-Pro / Access Point AC Pro"),
                    "U7PG2"   => Some("UAP-AC-Pro / Access Point AC Pro"),
                    "U7SHD"   => Some("UAP-AC-SHD / Access Point AC SHD"),
                    "UAE6"    => Some("U6-Extender-EA / Access Point WiFi 6 Extender"),
                    "UAIW6"   => Some("U6-IW-EA / Access Point WiFi 6 In-Wall"),
                    "UAL6"    => Some("U6-Lite / Access Point WiFi 6 Lite"),
                    "UALR6"   => Some("U6-LR-EA / Access Point WiFi 6 Long-Range"),
                    "UALR6v2" => Some("U6-LR / Access Point WiFi 6 Long-Range"),
                    "UALR6v3" => Some("U6-LR / Access Point WiFi 6 Long-Range"),
                    "UAM6"    => Some("U6-Mesh-EA / Access Point WiFi 6 Mesh"),
                    "UAP6"    => Some("U6-LR / Access Point WiFi 6 Long-Range"),
                    "UAP6MP"  => Some("U6-Pro / Access Point WiFi 6 Pro"),
                    "UCMSH"   => Some("UAP-XG-Mesh / Access Point Mesh XG"),
                    "UCXG"    => Some("UAP-XG / Access Point XG"),
                    "UDMB"    => Some("UAP-BeaconHD / Access Point BeaconHD"),
                    "UFLHD"   => Some("UAP-FlexHD / Access Point FlexHD"),
                    "UHDIW"   => Some("UAP-IW-HD / Access Point In-Wall HD"),
                    "ULTE"    => Some("U-LTE / UniFi LTE"),
                    "ULTEPEU" => Some("U-LTE-Pro / UniFi LTE Pro"),
                    "ULTEPUS" => Some("U-LTE-Pro / UniFi LTE Pro"),
                    "UP1"     => Some("USP-Plug / SmartPower Plug"),
                    "UP6"     => Some("USP-Strip / SmartPower Strip (6 ports)"),
                    "UXBSDM"  => Some("UWB-XG-BK / WiFi BaseStation XG"),
                    "UXSDM"   => Some("UWB-XG / WiFi BaseStation XG"),
                    "p2N"     => Some("PICOM2HP / PicoStation M2 HP"),
                    _         => None
                }
            },
            "usw" => {
                match self.device_model.as_ref() {
                    "S216150"  => Some("US-16-150W / Switch 16 PoE (150 W)"),
                    "S224250"  => Some("US-24-250W / Switch 24 PoE (250 W)"),
                    "S224500"  => Some("US-24-500W / Switch 24 PoE (500 W)"),
                    "S248500"  => Some("US-48-500W / Switch 48 PoE (500 W)"),
                    "S248750"  => Some("US-48-750W / Switch 48 PoE (750 W)"),
                    "S28150"   => Some("US-8-150W / Switch 8 PoE (150 W)"),
                    "UDC48X6"  => Some("USW-Leaf / Switch Leaf"),
                    "US16P150" => Some("US-16-150W / Switch 16 PoE (150 W)"),
                    "US24"     => Some("USW-24-G1 / Switch 24"),
                    "US24P250" => Some("US-24-250W / Switch 24 PoE (250 W)"),
                    "US24P500" => Some("US-24-500W / Switch 24 PoE (500 W)"),
                    "US24PL2"  => Some("US-L2-24-PoE / Switch 24 PoE"),
                    "US24PRO"  => Some("USW-Pro-24-PoE / Switch Pro 24 PoE"),
                    "US24PRO2" => Some("USW-Pro-24 / Switch Pro 24"),
                    "US48"     => Some("US-48-G1 / Switch 48"),
                    "US48P500" => Some("US-48-500W / Switch 48 PoE (500 W)"),
                    "US48P750" => Some("US-48-750W / Switch 48 PoE (750 W)"),
                    "US48PL2"  => Some("US-L2-48-PoE / Switch 48 PoE"),
                    "US48PRO"  => Some("USW-Pro-48-PoE / Switch Pro 48 PoE"),
                    "US48PRO2" => Some("USW-Pro-48 / Switch Pro 48"),
                    "US624P"   => Some("USW-Enterprise-24-PoE / Switch Enterprise 24 PoE"),
                    "US648P"   => Some("USW-Enterprise-48-PoE / Switch Enterprise 48 PoE"),
                    "US68P"    => Some("USW-Enterprise-8-PoE / Switch Enterprise 8 PoE"),
                    "US6XG150" => Some("US-XG-6PoE / Switch 6 XG PoE"),
                    "US8"      => Some("US-8 / Switch 8"),
                    "US8P150"  => Some("US-8-150W / Switch 8 PoE (150 W)"),
                    "US8P60"   => Some("US-8-60W / Switch 8 (60 W)"),
                    "USAGGPRO" => Some("USW-Pro-Aggregation / Switch Aggregation Pro"),
                    "USC8"     => Some("US-8 / Switch 8"),
                    "USC8P150" => Some("US-8-150W / Switch 8 PoE (150 W)"),
                    "USC8P450" => Some("USW-Industrial / Switch Industrial"),
                    "USC8P60"  => Some("US-8-60W / Switch 8 (60 W)"),
                    "USF5P"    => Some("USW-Flex / Switch Flex"),
                    "USFXG"    => Some("USW-Flex-XG / Switch Flex XG"),
                    "USL16LP"  => Some("USW-Lite-16-PoE / Switch Lite 16 PoE"),
                    "USL16P"   => Some("USW-16-PoE / Switch 16 PoE"),
                    "USL24"    => Some("USW-24-G2 / Switch 24"),
                    "USL24P"   => Some("USW-24-PoE / Switch 24 PoE"),
                    "USL48"    => Some("USW-48-G2 / Switch 48"),
                    "USL48P"   => Some("USW-48-PoE / Switch 48 PoE"),
                    "USL8A"    => Some("USW-Aggregation / Switch Aggregation"),
                    "USL8LP"   => Some("USW-Lite-8-PoE / Switch Lite 8 PoE"),
                    "USL8MP"   => Some("USW-Mission-Critical / Switch Mission Critical"),
                    "USMINI"   => Some("USW-Flex-Mini / Switch Flex Mini"),
                    "USPPDUP"  => Some("USP-PDU-Pro / SmartPower PDU Pro"),
                    "USPRPS"   => Some("USP-RPS / SmartPower Redundant Power System"),
                    "USXG"     => Some("US-16-XG / Switch XG 16"),
                    "USXG24"   => Some("USW-EnterpriseXG-24 / Switch Enterprise XG 24"),
                    _          => None
                }
            },
            "ugw" => {
                match self.device_model.as_ref() {
                    "UGW3"   => Some("USG-3P / Security Gateway"),
                    "UGW4"   => Some("USG-Pro-4 / Security Gateway Pro"),
                    "UGWHD4" => Some("USG / Security Gateway"),
                    "UGWXG"  => Some("USG-XG-8 / Security Gateway XG"),
                    _        => None
                }
            },
            "uxg" => {
                match self.device_model.as_ref() {
                    "UXGPRO" => Some("UXG-Pro / Next-Generation Gateway Pro"),
                    _        => None
                }
            },
            "ubb" => {
                match self.device_model.as_ref() {
                    "UBB"   => Some("UBB / Building-to-Building Bridge"),
                    "UBBXG" => Some("UBB-XG / Building-to-Building Bridge XG"),
                    _       => None
                }
            },
            "uas" => {
                match self.device_model.as_ref() {
                    "UASXG" => Some("UAS-XG / Application Server XG"),
                    _       => None
                }
            },
            "udm" => {
                match self.device_model.as_ref() {
                    "UDM"      => Some("UDM / Dream Machine"),
                    "UDMPRO"   => Some("UDM-Pro / Dream Machine Pro"),
                    "UDMPROSE" => Some("UDM-SE / Dream Machine Special Edition"),
                    "UDR"      => Some("UDR / Dream Router"),
                    "UDW"      => Some("UDW / Dream Wall"),
                    "UDWPRO"   => Some("UDWPRO / Dream Wall Pro"),
                    _          => None
                }
            },
            "uck" => {
                match self.device_model.as_ref() {
                    "UCK"    => Some("UCK / Cloud Key"),
                    "UCK-v2" => Some("UCK / Cloud Key"),
                    "UCK-v3" => Some("UCK / Cloud Key"),
                    "UCKG2"  => Some("UCK-G2 / Cloud Key Gen2"),
                    "UCKP"   => Some("UCK-G2-Plus / Cloud Key Gen2 Plus"),
                    _        => None
                }
            },
            "uph" => {
                match self.device_model.as_ref() {
                    "UP4"   => Some("UVP-X / Phone"),
                    "UP5"   => Some("UVP / Phone"),
                    "UP5c"  => Some("UVP / Phone"),
                    "UP5t"  => Some("UVP-Pro / Phone Professional"),
                    "UP5tc" => Some("UVP-Pro / Phone Professional"),
                    "UP7"   => Some("UVP-Executive / Phone Executive"),
                    "UP7c"  => Some("UVP-Executive / Phone Executive"),
                    _       => None
                }
            },
            _ => None
        }
    }
}
