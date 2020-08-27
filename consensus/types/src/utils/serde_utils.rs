use serde::de::Error;
use serde::{Deserialize, Deserializer, Serializer};

pub const FORK_BYTES_LEN: usize = 4;
pub const GRAFFITI_BYTES_LEN: usize = 32;

/// Type for a slice of `GRAFFITI_BYTES_LEN` bytes.
///
/// Gets included inside each `BeaconBlockBody`.
pub type Graffiti = [u8; GRAFFITI_BYTES_LEN];

pub fn u8_from_hex_str<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    let start = match s.as_str().get(2..) {
        Some(start) => start,
        None => return Err(D::Error::custom("string length too small")),
    };
    u8::from_str_radix(&start, 16).map_err(D::Error::custom)
}

#[allow(clippy::trivially_copy_pass_by_ref)] // Serde requires the `byte` to be a ref.
pub fn u8_to_hex_str<S>(byte: &u8, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut hex: String = "0x".to_string();
    hex.push_str(&hex::encode(&[*byte]));

    serializer.serialize_str(&hex)
}

pub fn u32_from_hex_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let start = s
        .as_str()
        .get(2..)
        .ok_or_else(|| D::Error::custom("string length too small"))?;

    u32::from_str_radix(&start, 16)
        .map_err(D::Error::custom)
        .map(u32::from_be)
}

#[allow(clippy::trivially_copy_pass_by_ref)] // Serde requires the `num` to be a ref.
pub fn u32_to_hex_str<S>(num: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut hex: String = "0x".to_string();
    let bytes = num.to_le_bytes();
    hex.push_str(&hex::encode(&bytes));

    serializer.serialize_str(&hex)
}

pub fn fork_from_hex_str<'de, D>(deserializer: D) -> Result<[u8; FORK_BYTES_LEN], D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let mut array = [0 as u8; FORK_BYTES_LEN];

    let start = s
        .as_str()
        .get(2..)
        .ok_or_else(|| D::Error::custom("string length too small"))?;
    let decoded: Vec<u8> = hex::decode(&start).map_err(D::Error::custom)?;

    if decoded.len() != FORK_BYTES_LEN {
        return Err(D::Error::custom("Fork length too long"));
    }

    for (i, item) in array.iter_mut().enumerate() {
        if i > decoded.len() {
            break;
        }
        *item = decoded[i];
    }
    Ok(array)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn fork_to_hex_str<S>(bytes: &[u8; FORK_BYTES_LEN], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut hex_string: String = "0x".to_string();
    hex_string.push_str(&hex::encode(&bytes));

    serializer.serialize_str(&hex_string)
}

pub fn graffiti_to_hex_str<S>(bytes: &Graffiti, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut hex_string: String = "0x".to_string();
    hex_string.push_str(&hex::encode(&bytes));

    serializer.serialize_str(&hex_string)
}

pub fn graffiti_from_hex_str<'de, D>(deserializer: D) -> Result<Graffiti, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let mut array = Graffiti::default();

    let start = s
        .as_str()
        .get(2..)
        .ok_or_else(|| D::Error::custom("string length too small"))?;
    let decoded: Vec<u8> = hex::decode(&start).map_err(D::Error::custom)?;

    if decoded.len() > GRAFFITI_BYTES_LEN {
        return Err(D::Error::custom("Fork length too long"));
    }

    for (i, item) in array.iter_mut().enumerate() {
        if i > decoded.len() {
            break;
        }
        *item = decoded[i];
    }
    Ok(array)
}

pub mod u32_hex {
    use super::*;

    pub fn serialize<S>(num: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut hex: String = "0x".to_string();
        let bytes = num.to_le_bytes();
        hex.push_str(&hex::encode(&bytes));

        serializer.serialize_str(&hex)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let start = s
            .as_str()
            .get(2..)
            .ok_or_else(|| D::Error::custom("string length too small"))?;

        u32::from_str_radix(&start, 16)
            .map_err(D::Error::custom)
            .map(u32::from_be)
    }
}

pub mod u8_hex {
    use super::*;

    pub fn serialize<S>(byte: &u8, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut hex: String = "0x".to_string();
        hex.push_str(&hex::encode(&[*byte]));

        serializer.serialize_str(&hex)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u8, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        let start = match s.as_str().get(2..) {
            Some(start) => start,
            None => return Err(D::Error::custom("string length too small")),
        };
        u8::from_str_radix(&start, 16).map_err(D::Error::custom)
    }
}

pub mod fork_bytes_4 {
    use super::*;

    pub fn serialize<S>(bytes: &[u8; FORK_BYTES_LEN], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut hex_string: String = "0x".to_string();
        hex_string.push_str(&hex::encode(&bytes));

        serializer.serialize_str(&hex_string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; FORK_BYTES_LEN], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let mut array = [0 as u8; FORK_BYTES_LEN];

        let start = s
            .as_str()
            .get(2..)
            .ok_or_else(|| D::Error::custom("string length too small"))?;
        let decoded: Vec<u8> = hex::decode(&start).map_err(D::Error::custom)?;

        if decoded.len() != FORK_BYTES_LEN {
            return Err(D::Error::custom("Fork length too long"));
        }

        for (i, item) in array.iter_mut().enumerate() {
            if i > decoded.len() {
                break;
            }
            *item = decoded[i];
        }
        Ok(array)
    }
}

pub mod graffiti {
    use super::*;

    pub fn serialize<S>(bytes: &Graffiti, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut hex_string: String = "0x".to_string();
        hex_string.push_str(&hex::encode(&bytes));

        serializer.serialize_str(&hex_string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Graffiti, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let mut array = Graffiti::default();

        let start = s
            .as_str()
            .get(2..)
            .ok_or_else(|| D::Error::custom("string length too small"))?;
        let decoded: Vec<u8> = hex::decode(&start).map_err(D::Error::custom)?;

        if decoded.len() > GRAFFITI_BYTES_LEN {
            return Err(D::Error::custom("Fork length too long"));
        }

        for (i, item) in array.iter_mut().enumerate() {
            if i > decoded.len() {
                break;
            }
            *item = decoded[i];
        }
        Ok(array)
    }
}

pub mod quoted {
    use super::*;

    pub struct QuotedIntVisitor;
    impl<'a> serde::de::Visitor<'a> for QuotedIntVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a quoted or unquoted integer")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let s = if s.len() > 2 && s.starts_with("\"") && s.ends_with("\"") {
                &s[1..s.len() - 1]
            } else {
                s
            };
            s.parse().map_err(serde::de::Error::custom)
        }
    }

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(QuotedIntVisitor)
    }
}

pub mod quoted_u64_vec {
    use super::*;
    use serde::ser::SerializeSeq;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct QuotedIntWrapper {
        #[serde(with = "super::quoted")]
        int: u64,
    }

    pub struct QuotedIntVecVisitor;
    impl<'a> serde::de::Visitor<'a> for QuotedIntVecVisitor {
        type Value = Vec<u64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a list of quoted or unquoted integers")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'a>,
        {
            let mut vec = vec![];

            while let Some(val) = seq.next_element()? {
                let val: QuotedIntWrapper = val;
                vec.push(val.int);
            }

            Ok(vec)
        }
    }

    pub fn serialize<S>(value: &Vec<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for &int in value {
            seq.serialize_element(&QuotedIntWrapper { int })?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(QuotedIntVecVisitor)
    }
}
