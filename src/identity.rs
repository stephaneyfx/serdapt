// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Identity adapter
///
/// This adapter causes a type to be serialized with its own implementations of [`Serialize`] and
/// [`Deserialize`].
///
/// # Example
/// ```
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
/// use std::collections::BTreeMap;
///
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Foo(#[serde(with = "sa::Map::<sa::Str, sa::Id>")] BTreeMap<i32, i32>);
///
/// let foo = Foo(BTreeMap::from_iter([(33, 0), (34, 1)]));
/// let serialized = serde_json::to_value(&foo).unwrap();
/// assert_eq!(serialized, json!({ "33": 0, "34": 1 }));
/// let deserialized = serde_json::from_value::<Foo>(serialized).unwrap();
/// assert_eq!(deserialized, foo);
/// ```
pub struct Id;

impl Id {
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

impl<T> SerializeWith<T> for Id
where
    T: Serialize + ?Sized,
{
    fn serialize_with<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(value, serializer)
    }
}

impl<'de, T> DeserializeWith<'de, T> for Id
where
    T: Deserialize<'de>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
    }
}
