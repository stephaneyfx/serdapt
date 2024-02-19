// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith, WithEncoding};
use core::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Adapter for [`Result`](core::result::Result)
///
/// # Example
/// ```
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "sa::Result::<sa::Str, sa::Id>")] Result<i32, ()>);
///
/// let v = serde_json::to_value(Foo(Ok(33))).unwrap();
/// assert_eq!(v, json!({ "Ok": "33" }));
/// ```
pub struct Result<F, G> {
    _f: PhantomData<F>,
    _g: PhantomData<G>,
}

impl<F, G> Result<F, G> {
    /// Serializes value with adapter
    pub fn serialize<T, S>(value: &T, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        T: ?Sized,
        S: Serializer,
        Self: SerializeWith<T>,
    {
        Self::serialize_with(value, serializer)
    }

    /// Deserializes value with adapter
    pub fn deserialize<'de, T, D>(deserializer: D) -> core::result::Result<T, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, T>,
    {
        Self::deserialize_with(deserializer)
    }
}

impl<F, G, T, E> SerializeWith<core::result::Result<T, E>> for Result<F, G>
where
    F: SerializeWith<T>,
    G: SerializeWith<E>,
{
    fn serialize_with<S: Serializer>(
        value: &core::result::Result<T, E>,
        serializer: S,
    ) -> core::result::Result<S::Ok, S::Error> {
        let value = value
            .as_ref()
            .map(WithEncoding::<&F, &T>::from)
            .map_err(WithEncoding::<&G, &E>::from);
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, G, T, E> DeserializeWith<'de, core::result::Result<T, E>> for Result<F, G>
where
    F: DeserializeWith<'de, T>,
    G: DeserializeWith<'de, E>,
{
    fn deserialize_with<D>(
        deserializer: D,
    ) -> core::result::Result<core::result::Result<T, E>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: core::result::Result<WithEncoding<F, T>, WithEncoding<G, E>> =
            Deserialize::deserialize(deserializer)?;
        Ok(r.map(WithEncoding::into_inner)
            .map_err(WithEncoding::into_inner))
    }
}
