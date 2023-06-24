use serde::de::{Error, Visitor};
use serde::Deserialize;

pub struct EmptyStruct;

struct EmptyStructVisitor;

impl<'de> Visitor<'de> for EmptyStructVisitor {
    type Value = EmptyStruct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an empty map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        match map.next_key::<String>() {
            Ok(Some(key)) => Err(Error::unknown_field(&key, &[])),
            Ok(None) => Ok(Self::Value {}),
            Err(err) => Err(err),
        }
    }
}

impl<'de> Deserialize<'de> for EmptyStruct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("EmptyStruct", &[], EmptyStructVisitor)
    }
}
