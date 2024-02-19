// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, Id};
use core::{fmt::Display, marker::PhantomData};
use serde::Deserializer;

/// Adapter to deserialize using a [`TryFrom`](core::convert::TryFrom) conversion
///
/// This adapter works by deserializing a value of type `T` using adapter `F`, and then attempting
/// a conversion from `T` to the target type.
///
/// # Example
/// ```
/// use serde::Deserialize;
/// use serde_json::json;
///
/// #[derive(Deserialize)]
/// struct Foo(#[serde(with = "serdapt::TryFrom::<u32>")] char);
///
/// let x = serde_json::from_value::<Foo>(json!(b'a')).unwrap();
/// assert_eq!(x.0, 'a');
/// ```
pub struct TryFrom<T, F = Id> {
    _convert: PhantomData<fn(T)>,
    _f: PhantomData<F>,
}

impl<T, F> TryFrom<T, F> {
    /// Deserializes value with adapter
    pub fn deserialize<'de, U, D>(deserializer: D) -> Result<U, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, U>,
    {
        Self::deserialize_with(deserializer)
    }
}

impl<'de, T, U, F> DeserializeWith<'de, U> for TryFrom<T, F>
where
    U: core::convert::TryFrom<T>,
    U::Error: Display,
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<U, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        F::deserialize_with(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize)]
    struct Foo(
        #[allow(unused)]
        #[serde(with = "crate::TryFrom::<u32>")]
        char,
    );

    #[test]
    fn try_from_adapter_fails_to_deserialize_if_conversion_fails() {
        serde_json::from_value::<Foo>(json!(0xd800)).unwrap_err();
    }
}
