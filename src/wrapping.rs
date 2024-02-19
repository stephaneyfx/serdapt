// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith, WithEncoding};
use core::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Adapter for [`Wrapping`](core::num::Wrapping)
///
/// # Example
/// ```
/// use core::num::Wrapping;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "serdapt::Wrapping::<serdapt::Str>")] Wrapping<i32>);
///
/// let v = serde_json::to_value(Foo(Wrapping(33))).unwrap();
/// assert_eq!(v, json!("33"));
/// ```
pub struct Wrapping<F>(PhantomData<F>);

impl<F> Wrapping<F> {
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

impl<F, T> SerializeWith<core::num::Wrapping<T>> for Wrapping<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(
        value: &core::num::Wrapping<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value = core::num::Wrapping(WithEncoding::<&F, &T>::from(&value.0));
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, core::num::Wrapping<T>> for Wrapping<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<core::num::Wrapping<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let x: core::num::Wrapping<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(core::num::Wrapping(x.0.into_inner()))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::check_serialization;
    use core::num::Wrapping;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Foo(#[serde(with = "crate::Wrapping::<crate::Str>")] Wrapping<i32>);

    #[test]
    fn wrapping_adapter_roundtrips() {
        check_serialization(Foo(Wrapping(33)), json!("33"));
    }
}
