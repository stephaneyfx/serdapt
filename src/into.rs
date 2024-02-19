// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{Id, SerializeWith};
use core::marker::PhantomData;
use serde::Serializer;

/// Adapter to serialize using a [`From`](core::convert::From) conversion
///
/// This adapter works by converting to `T` and then serializing the converted value using adapter
/// `F`.
///
/// # Example
/// ```
/// use serde::Serialize;
/// use serde_json::json;
///
/// #[derive(Serialize)]
/// struct Foo(#[serde(with = "serdapt::Into::<u8>")] bool);
///
/// let v = serde_json::to_value(Foo(true)).unwrap();
/// assert_eq!(v, json!(1));
/// ```
pub struct Into<T, F = Id> {
    _convert: PhantomData<fn() -> T>,
    _f: PhantomData<F>,
}

impl<T, F> Into<T, F> {
    /// Serializes value with adapter
    pub fn serialize<U, S>(value: &U, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        Self: SerializeWith<U>,
    {
        Self::serialize_with(value, serializer)
    }
}

impl<T, U, F> SerializeWith<U> for Into<T, F>
where
    T: From<U>,
    U: Clone,
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(value: &U, serializer: S) -> Result<S::Ok, S::Error> {
        F::serialize_with(&value.clone().into(), serializer)
    }
}
