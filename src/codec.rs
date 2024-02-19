// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use core::marker::PhantomData;
use serde::{Deserializer, Serializer};

/// Adapter to pair a serialization adapter with a deserialization adapter
///
/// This adapter serializes with `F` and deserializes with `G`.
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Foo {
///     #[serde(with = "serdapt::Codec::<serdapt::TryInto<u32>, serdapt::TryFrom<u32>>")]
///     c: char,
/// }
///
/// let foo = Foo { c: 'a' };
/// let serialized = serde_json::to_value(&foo).unwrap();
/// assert_eq!(serialized, json!({ "c": b'a' }));
/// let deserialized = serde_json::from_value::<Foo>(serialized).unwrap();
/// assert_eq!(deserialized, foo);
/// ```
pub struct Codec<F, G>(PhantomData<(F, G)>);

impl<F, G> Codec<F, G> {
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

impl<F, G, T> SerializeWith<T> for Codec<F, G>
where
    F: SerializeWith<T>,
    T: ?Sized,
{
    fn serialize_with<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        F::serialize_with(value, serializer)
    }
}

impl<'de, F, G, T> DeserializeWith<'de, T> for Codec<F, G>
where
    G: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        G::deserialize_with(deserializer)
    }
}
