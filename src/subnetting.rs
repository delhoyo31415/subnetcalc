use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub struct IpAddressBlock {
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

// Minimum bits needed to represent the quantity num
// Returns u8 because it is the minimum primitive type
// which can hold the maximum number of usize
fn minimum_bits_needed(mut num: usize) -> u8 {
    if num == 0 {
        panic!("num is equal to 0");
    }

    let mut counter = 0;
    num -= 1;
    while num != 0 {
        num >>= 1;
        counter += 1;
    }
    counter
}
impl IpAddressBlock {
    // Panics if the network mask is not in the range {0, 1, 2, ..., 32}
    pub fn new(address: [u8; 4], mask: u8) -> Self {
        if mask > 32 {
            panic!("{mask} is not a valid mask");
        }
        Self { address, mask }
    }

    pub fn from_u32_address(u32_addr: u32, mask: u8) -> Self {
        let address = [
            ((u32_addr >> 24) & 0xFF) as u8,
            ((u32_addr >> 16) & 0xFF) as u8,
            ((u32_addr >> 8) & 0xFF) as u8,
            ((u32_addr >> 0) & 0xFF) as u8,
        ];

        Self::new(address, mask)
    }

    pub fn subnet_flsm(&self, num_networks: usize) -> Vec<Self> {
        let new_mask = match self.new_mask_for(num_networks) {
            Some(mask) => mask,
            None => return Vec::new(),
        };

        let remaining_bits = 32 - new_mask;
        let as_u32 = self.address_as_u32();
        let bitmask = !((1 << remaining_bits) - 1);

        let mut network_id = (as_u32 & bitmask) >> remaining_bits;
        let mut blocks = Vec::with_capacity(num_networks);

        for _ in 0..num_networks {
            let new_as_u32 = network_id << remaining_bits;
            blocks.push(Self::from_u32_address(new_as_u32, new_mask));
            network_id += 1;
        }

        blocks
    }

    // Converts the array representing the address to a
    pub fn address_as_u32(&self) -> u32 {
        // 'self.address' is an array of four u8, so it is cheap to copy them
        self.address
            .into_iter()
            .fold(0_u32, |acc, octet| (acc << 8) + octet as u32)
    }

    fn new_mask_for(&self, num_networks: usize) -> Option<u8> {
        if num_networks == 0 {
            return None;
        }

        let bits_needed = minimum_bits_needed(num_networks);
        let mask = self.mask + bits_needed;

        if mask <= 32 {
            Some(mask)
        } else {
            None
        }
    }
}

fn parse_octet(octet: &str) -> Result<u8, <IpAddressBlock as FromStr>::Err> {
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

fn extract_address_and_mask(s: &str) -> Result<([u8; 4], u8), <IpAddressBlock as FromStr>::Err> {
    let octets = s.split('.').collect::<Vec<_>>();

    if octets.len() != 4 {
        return Err(IpAddressParseError {
            kind: IpAddressErrorKind::IncorrectFormat,
        });
    }

    let mut address = [0_u8; 4];
    let mut mask = 0_u8;

    for (idx, &octet) in octets.iter().enumerate() {
        // Last part, which is composed of the last octet and the mask
        if idx == 3 {
            let mut last_iter = octet.split('/');

            let octet_str = last_iter
                .next()
                .expect("split iterator must have  at least one element");
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

impl FromStr for IpAddressBlock {
    type Err = IpAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (address, mask) = extract_address_and_mask(s)?;

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
            "192.168.0.3/24".parse::<IpAddressBlock>().unwrap(),
            IpAddressBlock::new([192, 168, 0, 3], 24)
        );

        assert_eq!(
            "21.123.1.3/32".parse::<IpAddressBlock>().unwrap(),
            IpAddressBlock::new([21, 123, 1, 3], 32)
        );

        assert_eq!(
            "255.255.255.255/09".parse::<IpAddressBlock>().unwrap(),
            IpAddressBlock::new([255, 255, 255, 255], 9)
        );

        assert_eq!(
            "0.0.0.0/0".parse::<IpAddressBlock>().unwrap(),
            IpAddressBlock::new([0, 0, 0, 0], 0)
        );
    }

    #[test]
    fn incorrectly_ip() {
        assert!("21.123.1./32".parse::<IpAddressBlock>().is_err());
        assert!("21/32".parse::<IpAddressBlock>().is_err());
        assert!("300.23.1.23/32".parse::<IpAddressBlock>().is_err());
        assert!("23.23.1.23/40".parse::<IpAddressBlock>().is_err());
        assert!("23.../23".parse::<IpAddressBlock>().is_err());
        assert!("23.13".parse::<IpAddressBlock>().is_err());
        assert!("23.13..13/23".parse::<IpAddressBlock>().is_err());
        assert!("123.+23.1.23/32".parse::<IpAddressBlock>().is_err());
        assert!("213.-23.1.23/32".parse::<IpAddressBlock>().is_err());
    }

    #[test]
    fn correctly_creates_address_from_u32() {
        let addr = "201.70.64.0/24".parse::<IpAddressBlock>().unwrap();
        let as_u32 = addr
            .address
            .into_iter()
            .fold(0_u32, |acc, octet| (acc << 8) + octet as u32);

        assert_eq!(addr, IpAddressBlock::from_u32_address(as_u32, 24));
    }

    #[test]
    fn subnets_flsm_correctly() {
        let addr = "201.70.64.0/24".parse::<IpAddressBlock>().unwrap();
        let expected = vec![
            "201.70.64.0/27".parse::<IpAddressBlock>().unwrap(),
            "201.70.64.32/27".parse::<IpAddressBlock>().unwrap(),
            "201.70.64.64/27".parse::<IpAddressBlock>().unwrap(),
            "201.70.64.96/27".parse::<IpAddressBlock>().unwrap(),
            "201.70.64.128/27".parse::<IpAddressBlock>().unwrap(),
            "201.70.64.160/27".parse::<IpAddressBlock>().unwrap(),
        ];
        assert_eq!(addr.subnet_flsm(6), expected);

        let addr = "198.150.74.0/23".parse::<IpAddressBlock>().unwrap();
        let expected = vec![
            "198.150.74.0/25".parse::<IpAddressBlock>().unwrap(),
            "198.150.74.128/25".parse::<IpAddressBlock>().unwrap(),
            "198.150.75.0/25".parse::<IpAddressBlock>().unwrap(),
            "198.150.75.128/25".parse::<IpAddressBlock>().unwrap(),
        ];
        assert_eq!(addr.subnet_flsm(4), expected);

        let addr = "181.56.0.0/16".parse::<IpAddressBlock>().unwrap();
        let mut it = addr.subnet_flsm(1000).into_iter().rev();

        assert_eq!(
            it.next().unwrap(),
            "181.56.249.192/26".parse::<IpAddressBlock>().unwrap()
        );
        assert_eq!(
            it.next().unwrap(),
            "181.56.249.128/26".parse::<IpAddressBlock>().unwrap()
        );
    }
}
