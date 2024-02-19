// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use core::marker::PhantomData;
use serde::{ser::Error, Deserializer, Serializer};

/// Adapter for [`RwLock`](std::sync::RwLock)
///
/// # Example
/// ```
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
/// use std::sync::RwLock;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "sa::RwLock::<sa::Str>")] RwLock<i32>);
///
/// let v = serde_json::to_value(Foo(33.into())).unwrap();
/// assert_eq!(v, json!("33"));
/// ```
pub struct RwLock<F>(PhantomData<F>);

impl<F> RwLock<F> {
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

impl<F, T> SerializeWith<std::sync::RwLock<T>> for RwLock<F>
where
    F: SerializeWith<T>,
    T: ?Sized,
{
    fn serialize_with<S: Serializer>(
        value: &std::sync::RwLock<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        F::serialize_with(&*value.try_read().map_err(S::Error::custom)?, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, std::sync::RwLock<T>> for RwLock<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<std::sync::RwLock<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        F::deserialize_with(deserializer).map(std::sync::RwLock::new)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::sync::RwLock;

    #[derive(Deserialize, Serialize)]
    struct Foo(#[serde(with = "crate::RwLock::<crate::Str>")] RwLock<i32>);

    #[test]
    fn rwlock_adapter_roundtrips() {
        let foo = Foo(RwLock::new(33));
        let serialized = serde_json::to_value(foo).unwrap();
        assert_eq!(serialized, json!("33"));
        let deserialized = serde_json::from_value::<Foo>(serialized).unwrap();
        assert_eq!(*deserialized.0.read().unwrap(), 33);
    }

    #[test]
    fn serializing_locked_rwlock_fails() {
        let foo = Foo(RwLock::new(33));
        let _lock = foo.0.write().unwrap();
        serde_json::to_value(&foo).unwrap_err();
    }
}
