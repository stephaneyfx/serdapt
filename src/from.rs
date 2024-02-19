// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, Id};
use core::marker::PhantomData;
use serde::Deserializer;

/// Adapter to deserialize using a [`From`](core::convert::From) conversion
///
/// This adapter works by deserializing a value of type `T` using adapter `F`, and then converting
/// from `T` to the target type.
///
/// # Example
/// ```
/// use serde::Deserialize;
/// use serde_json::json;
///
/// #[derive(Deserialize)]
/// struct Foo(#[serde(with = "serdapt::From::<bool>")] u8);
///
/// let x = serde_json::from_value::<Foo>(json!(true)).unwrap();
/// assert_eq!(x.0, 1);
/// ```
pub struct From<T, F = Id> {
    _convert: PhantomData<fn(T)>,
    _f: PhantomData<F>,
}

impl<T, F> From<T, F> {
    /// Deserializes value with adapter
    pub fn deserialize<'de, U, D>(deserializer: D) -> Result<U, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, U>,
    {
        Self::deserialize_with(deserializer)
    }
}

impl<'de, T, U, F> DeserializeWith<'de, U> for From<T, F>
where
    U: core::convert::From<T>,
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<U, D::Error>
    where
        D: Deserializer<'de>,
    {
        F::deserialize_with(deserializer).map(Into::into)
    }
}
