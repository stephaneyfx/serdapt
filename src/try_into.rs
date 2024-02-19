// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{Id, SerializeWith};
use core::{fmt::Display, marker::PhantomData};
use serde::Serializer;

/// Adapter to serialize using a [`TryFrom`](core::convert::TryFrom) conversion
///
/// This adapter works by attempting a conversion to `T`, and then serializing the converted value
/// using adapter `F`.
///
/// # Example
/// ```
/// use serde::Serialize;
/// use serde_json::json;
///
/// #[derive(Debug, Serialize)]
/// struct Foo(#[serde(with = "serdapt::TryInto::<u8>")] u32);
///
/// let v = serde_json::to_value(Foo(33)).unwrap();
/// assert_eq!(v, json!(33));
/// ```
pub struct TryInto<T, F = Id> {
    _convert: PhantomData<fn() -> T>,
    _f: PhantomData<F>,
}

impl<T, F> TryInto<T, F> {
    /// Serializes value with adapter
    pub fn serialize<U, S>(value: &U, serializer: S) -> Result<S::Ok, S::Error>
    where
        U: ?Sized,
        S: Serializer,
        Self: SerializeWith<U>,
    {
        Self::serialize_with(value, serializer)
    }
}

impl<T, U, F> SerializeWith<U> for TryInto<T, F>
where
    T: TryFrom<U>,
    T::Error: Display,
    U: Clone,
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(value: &U, serializer: S) -> Result<S::Ok, S::Error> {
        F::serialize_with(
            &value
                .clone()
                .try_into()
                .map_err(serde::ser::Error::custom)?,
            serializer,
        )
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    struct Foo(#[serde(with = "crate::TryInto::<u8>")] u32);

    #[test]
    fn try_into_adapter_fails_to_serialize_if_conversion_fails() {
        serde_json::to_value(Foo(256)).unwrap_err();
    }
}
