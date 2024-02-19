// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use alloc::borrow::ToOwned;
use core::marker::PhantomData;
use serde::{Deserializer, Serializer};

/// [`Cow`](alloc::borrow::Cow) adapter
///
/// This adapter allows to customize how a type inside [`Cow`](alloc::borrow::Cow) is serialized.
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
/// use std::borrow::Cow;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo<'a>(#[serde(with = "serdapt::Cow::<serdapt::Str>")] Cow<'a, i32>);
///
/// let v = serde_json::to_value(Foo(Cow::Owned(33))).unwrap();
/// assert_eq!(v, json!("33"));
/// ```
pub struct Cow<F>(PhantomData<F>);

impl<F> Cow<F> {
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

impl<'a, F, T> SerializeWith<alloc::borrow::Cow<'a, T>> for Cow<F>
where
    F: SerializeWith<T>,
    T: ToOwned + ?Sized,
{
    fn serialize_with<S: Serializer>(
        value: &alloc::borrow::Cow<'a, T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        F::serialize_with(value, serializer)
    }
}

impl<'de, 'a, F, T> DeserializeWith<'de, alloc::borrow::Cow<'a, T>> for Cow<F>
where
    F: DeserializeWith<'de, T::Owned>,
    T: ToOwned,
{
    fn deserialize_with<D>(deserializer: D) -> Result<alloc::borrow::Cow<'a, T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        F::deserialize_with(deserializer).map(alloc::borrow::Cow::Owned)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::check_serialization;
    use alloc::borrow::Cow;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Foo<'a>(#[serde(with = "crate::Cow::<crate::Str>")] Cow<'a, i32>);

    #[test]
    fn cow_adapter_roundtrips() {
        check_serialization(Foo(Cow::Owned(33)), json!("33"));
    }
}
