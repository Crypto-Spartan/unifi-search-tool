pub mod validation;

/*pub(crate) struct MACAddress{
    bytes: [u8; 6]
}

impl MACAddress {
    pub fn new(bytes: [u8; 6]) -> MACAddress {
        MACAddress{ bytes }
    }

    pub fn bytes(self) -> [u8; 6] {
        self.bytes
    }
}

impl From<[u8; 6]> for MACAddress {
    fn from(v: [u8; 6]) -> Self {
        MACAddress::new(v)
    }
}

impl std::str::FromStr for MACAddress {
    type Err = MACParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut array = [0u8; 6];

        let mut nth = 0;
        for byte in input.split(|c| c == ':' || c == '-') {
            if nth == 6 {
                return Err(MACParseError::InvalidLength);
            }

            array[nth] = u8::from_str_radix(byte, 16).map_err(|_| MACParseError::InvalidDigit)?;

            nth += 1;
        }

        if nth != 6 {
            return Err(MACParseError::InvalidLength);
        }

        Ok(MACAddress::new(array))
    }
}

impl std::convert::TryFrom<&'_ str> for MACAddress {
    type Error = MACParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl std::convert::TryFrom<std::borrow::Cow<'_, str>> for MACAddress {
    type Error = MACParseError;

    fn try_from(value: std::borrow::Cow<'_, str>) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl std::fmt::Display for MACAddress {
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
}*/

// impl serde::Serialize for MACAddress {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         serializer.collect_str(self)
//     }
// }
