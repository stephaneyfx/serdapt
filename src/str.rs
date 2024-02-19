// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use core::{
    fmt::{self, Display},
    marker::PhantomData,
    str::FromStr,
};
use serde::{de::Visitor, Deserializer, Serializer};

/// Adapter to serialize types using their [`Display`] and [`FromStr`] implementations
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "serdapt::Str")] i32);
///
/// let v = serde_json::to_value(Foo(33)).unwrap();
/// assert_eq!(v, json!("33"));
/// ```
pub struct Str;

impl Str {
    /// Serializes value with adapter
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ?Sized,
        S: Serializer,
        Self: SerializeWith<T>,
    {
        Self::serialize_with(value, serializer)
    }

    /// Deserializes value with adapter
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, T>,
    {
        Self::deserialize_with(deserializer)
    }
}

impl<T> SerializeWith<T> for Str
where
    T: Display + ?Sized,
{
    fn serialize_with<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(value)
    }
}

impl<'de, T> DeserializeWith<'de, T> for Str
where
    T: FromStr,
    T::Err: Display,
{
    fn deserialize_with<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StrVisitor::new())
    }
}

struct StrVisitor<T>(PhantomData<fn() -> T>);

impl<T> StrVisitor<T> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T> Visitor<'de> for StrVisitor<T>
where
    T: FromStr,
    T::Err: Display,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(E::custom)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(core::str::from_utf8(v).map_err(E::custom)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::check_serialization;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Foo(#[serde(with = "crate::Str")] i32);

    #[test]
    fn str_adapter_roundtrips() {
        check_serialization(Foo(33), json!("33"));
    }
}
