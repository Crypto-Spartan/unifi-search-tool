use constcat::concat;
use serde::{de::{self, Unexpected}, Deserialize};
use thiserror::Error;

pub mod validation;
use validation::MAC_ADDR_REGEX_STR;

#[derive(Clone, Debug, Default)]//, PartialEq, Eq)]
pub(crate) struct MacAddress{
    bytes: [u8; 6]
}

impl MacAddress {
    pub fn new(bytes: [u8; 6]) -> MacAddress {
        MacAddress{ bytes }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.bytes
    }

    #[inline]
    pub fn into_bytes(self) -> [u8; 6] {
        self.bytes
    }
}

impl From<[u8; 6]> for MacAddress {
    #[inline]
    fn from(v: [u8; 6]) -> Self {
        MacAddress::new(v)
    }
}

#[derive(Error, Debug)]
pub(crate) enum MacParseError{
    #[error("Invalid MAC Address: {invalid_mac:?}")]
    InvalidMac{ invalid_mac: Box<str> },
}

impl std::str::FromStr for MacAddress {
    type Err = MacParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if !validation::text_is_valid_mac(input.as_bytes()) {
            return Err(MacParseError::InvalidMac { invalid_mac: Box::from(input) });
        }

        let mut array = [0u8; 6];
        let mac_bytes_iter = input
            .split([':', '-'])
            .map(|b_str| {
                u8::from_str_radix(b_str, 16).expect("mac validation failed")
            });

        for (idx, b) in array.iter_mut().zip(mac_bytes_iter) {
            *idx = b
        }

        Ok(MacAddress::new(array))
    }
}

impl std::convert::TryFrom<&'_ str> for MacAddress {
    type Error = MacParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl std::convert::TryFrom<std::borrow::Cow<'_, str>> for MacAddress {
    type Error = MacParseError;

    fn try_from(value: std::borrow::Cow<'_, str>) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = write!(
            f,
            "{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}",
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            self.bytes[3],
            self.bytes[4],
            self.bytes[5]
        );

        Ok(())
    }
}

// impl serde::Serialize for MacAddress {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         serializer.collect_str(self)
//     }
// }

impl<'de> Deserialize<'de> for MacAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mac_str: &str = de::Deserialize::deserialize(deserializer)?;
        MacAddress::try_from(mac_str).map_err(|_| {
            let unexpected = Unexpected::Str(mac_str);
            const EXPECTED: &str = concat!("MAC Address in string format matching regex: ", MAC_ADDR_REGEX_STR);
            de::Error::invalid_value(unexpected, &EXPECTED)
        })
    }
}