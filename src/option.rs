// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith, WithEncoding};
use core::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Adapter for [`Option`](core::option::Option)
///
/// # Example
/// ```
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "sa::Option::<sa::Str>")] Option<i32>);
///
/// let v = serde_json::to_value(Foo(Some(33))).unwrap();
/// assert_eq!(v, json!("33"));
/// ```
pub struct Option<F>(PhantomData<F>);

impl<F> Option<F> {
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

impl<F, T> SerializeWith<core::option::Option<T>> for Option<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(
        value: &core::option::Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value = value.as_ref().map(WithEncoding::<&F, &T>::from);
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, core::option::Option<T>> for Option<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<core::option::Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let x: core::option::Option<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(x.map(WithEncoding::into_inner))
    }
}
