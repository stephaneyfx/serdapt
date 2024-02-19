// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith, WithEncoding};
use core::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Adapter for [`Reverse`](core::cmp::Reverse)
///
/// # Example
/// ```
/// use core::cmp::Reverse;
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "sa::Reverse::<sa::Str>")] Reverse<i32>);
///
/// let v = serde_json::to_value(Foo(Reverse(3))).unwrap();
/// assert_eq!(v, json!("3"));
/// ```
pub struct Reverse<F>(PhantomData<F>);

impl<F> Reverse<F> {
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

impl<F, T> SerializeWith<core::cmp::Reverse<T>> for Reverse<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(
        value: &core::cmp::Reverse<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value = core::cmp::Reverse(WithEncoding::<&F, &T>::from(&value.0));
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, core::cmp::Reverse<T>> for Reverse<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<core::cmp::Reverse<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let x: core::cmp::Reverse<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(core::cmp::Reverse(x.0.into_inner()))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::check_serialization;
    use core::cmp::Reverse;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Foo(#[serde(with = "crate::Reverse::<crate::Str>")] Reverse<i32>);

    #[test]
    fn reverse_adapter_roundtrips() {
        check_serialization(Foo(Reverse(33)), json!("33"));
    }
}
