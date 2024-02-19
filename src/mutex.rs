// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use core::marker::PhantomData;
use serde::{ser::Error, Deserializer, Serializer};

/// Adapter for [`Mutex`](std::sync::Mutex)
///
/// # Example
/// ```
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
/// use std::sync::Mutex;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "sa::Mutex::<sa::Str>")] Mutex<i32>);
///
/// let v = serde_json::to_value(Foo(Mutex::new(33))).unwrap();
/// assert_eq!(v, json!("33"));
/// ```
pub struct Mutex<F>(PhantomData<F>);

impl<F> Mutex<F> {
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

impl<F, T> SerializeWith<std::sync::Mutex<T>> for Mutex<F>
where
    F: SerializeWith<T>,
    T: ?Sized,
{
    fn serialize_with<S: Serializer>(
        value: &std::sync::Mutex<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        F::serialize_with(&*value.try_lock().map_err(S::Error::custom)?, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, std::sync::Mutex<T>> for Mutex<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<std::sync::Mutex<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        F::deserialize_with(deserializer).map(std::sync::Mutex::new)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::sync::Mutex;

    #[derive(Debug, Deserialize, Serialize)]
    struct Foo(#[serde(with = "crate::Mutex::<crate::Str>")] Mutex<i32>);

    #[test]
    fn mutex_adapter_roundtrips() {
        let foo = Foo(Mutex::new(33));
        let serialized = serde_json::to_value(foo).unwrap();
        assert_eq!(serialized, json!("33"));
        let deserialized = serde_json::from_value::<Foo>(serialized).unwrap();
        assert_eq!(*deserialized.0.lock().unwrap(), 33);
    }

    #[test]
    fn serializing_fails_if_mutex_cannot_be_locked() {
        let foo = Foo(Mutex::new(33));
        let _lock = foo.0.lock().unwrap();
        serde_json::to_value(&foo).unwrap_err();
    }
}
