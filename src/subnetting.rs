use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub struct IpAddress {
    pub address: [u8; 4],
    pub mask: u8,
}

#[derive(Debug)]
pub enum IpAddressErrorKind {
    IncorrectFormat,
    MissingMask,
    OctetOutOfRange(String),
    MaskOutOfRange(String),
}

// Error type inspired by https://doc.rust-lang.org/stable/src/core/num/error.rs.html#87-114
#[derive(Debug)]
pub struct IpAddressParseError {
    pub(crate) kind: IpAddressErrorKind,
}

impl IpAddressParseError {
    pub fn kind(&self) -> &IpAddressErrorKind {
        &self.kind
    }

    fn __description(&self) -> String {
        match &self.kind {
            IpAddressErrorKind::IncorrectFormat => {
                "the string is not in a correct format".to_string()
            }
            IpAddressErrorKind::MissingMask => "the mask is missing".to_string(),
            IpAddressErrorKind::OctetOutOfRange(out) => format!("'{out}' is not a valid octet"),
            IpAddressErrorKind::MaskOutOfRange(out) => format!("'{out}' is not a valid mask"),
        }
    }
}

impl Display for IpAddressParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.__description().fmt(f)
    }
}

impl IpAddress {
    // Panics if the network mask is not in the range {0, 1, 2, ..., 32}
    pub fn new(address: [u8; 4], mask: u8) -> Self {
        if mask > 32 {
            panic!("{mask} is not a valid mask");
        }
        Self { address, mask }
    }
}

fn parse_octet(octet: &str) -> Result<u8, <IpAddress as FromStr>::Err> {
    // If the symbol '+' is present in the octet, then it is an error
    // although the method parse::<u8>() from str returns the Ok variant if that
    // symbol is followed by numbers.
    let bytes = octet.as_bytes();
    if !bytes.is_empty() && (bytes[0] == b'+' || bytes[0] == b'-') {
        Err(IpAddressParseError {
            kind: IpAddressErrorKind::IncorrectFormat,
        })
    } else {
        octet.parse::<u8>().map_err(|_| IpAddressParseError {
            kind: IpAddressErrorKind::OctetOutOfRange(octet.to_string()),
        })
    }
}

fn extract_address_and_mask(s: &str) -> Result<([u8; 4], u8), <IpAddress as FromStr>::Err> {
    let split = s.split(".").collect::<Vec<_>>();

    match split.get(..4) {
        Some(octets) => {
            let mut address = [0_u8; 4];
            let mut mask = 0_u8;

            for (idx, &octet) in octets.iter().enumerate() {
                // Last part, which is composed of the last octet and the mask
                if idx == 3 {
                    let mut last_iter = octet.split("/");

                    // Split iterator returns at least one Some
                    let octet_str = last_iter.next().unwrap();
                    address[idx] = parse_octet(octet_str)?;

                    let mask_str = last_iter.next().ok_or(IpAddressParseError {
                        kind: IpAddressErrorKind::MissingMask,
                    })?;

                    mask = mask_str.parse::<u8>().map_err(|_| IpAddressParseError {
                        kind: IpAddressErrorKind::MaskOutOfRange(mask_str.to_string()),
                    })?;
                } else {
                    // First three octets
                    address[idx] = parse_octet(octet)?;
                }
            }
            Ok((address, mask))
        }
        None => Err(IpAddressParseError {
            kind: IpAddressErrorKind::IncorrectFormat,
        }),
    }
}

impl FromStr for IpAddress {
    type Err = IpAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (address, mask) = extract_address_and_mask(&s)?;

        if mask <= 32 {
            Ok(Self::new(address, mask))
        } else {
            Err(IpAddressParseError {
                kind: IpAddressErrorKind::MaskOutOfRange(mask.to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_parse_address() {
        assert_eq!(
            "192.168.0.3/24".parse::<IpAddress>().unwrap(),
            IpAddress::new([192, 168, 0, 3], 24)
        );

        assert_eq!(
            "21.123.1.3/32".parse::<IpAddress>().unwrap(),
            IpAddress::new([21, 123, 1, 3], 32)
        );

        assert_eq!(
            "255.255.255.255/09".parse::<IpAddress>().unwrap(),
            IpAddress::new([255, 255, 255, 255], 9)
        );

        assert_eq!(
            "0.0.0.0/0".parse::<IpAddress>().unwrap(),
            IpAddress::new([0, 0, 0, 0], 0)
        );
    }

    #[test]
    fn incorrectly_formed() {
        assert!("21.123.1./32".parse::<IpAddress>().is_err());
        assert!("21/32".parse::<IpAddress>().is_err());
        assert!("300.23.1.23/32".parse::<IpAddress>().is_err());
        assert!("23.23.1.23/40".parse::<IpAddress>().is_err());
        assert!("23.../23".parse::<IpAddress>().is_err());
        assert!("23.13".parse::<IpAddress>().is_err());
        assert!("23.13..13/23".parse::<IpAddress>().is_err());
        assert!("123.+23.1.23/32".parse::<IpAddress>().is_err());
        assert!("213.-23.1.23/32".parse::<IpAddress>().is_err());
    }
}
