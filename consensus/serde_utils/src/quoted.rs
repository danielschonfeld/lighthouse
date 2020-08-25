use serde::{Deserializer, Serializer};

pub struct QuotedIntVisitor;
impl<'a> serde::de::Visitor<'a> for QuotedIntVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string containing digits or an int fitting into u64"
        )
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
