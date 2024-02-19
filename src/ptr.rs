// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use core::{marker::PhantomData, ops::Deref};
use serde::{Deserializer, Serializer};

/// Adapter for pointer-like types to customize how the inner type is serialized
///
/// This adapter works with `Box`, `Arc`, etc.
///
/// # Example
/// ```
/// # #[cfg(feature = "std")] {
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "sa::Ptr::<sa::Str>")] Box<i32>);
///
/// let v = serde_json::to_value(Foo(Box::new(33))).unwrap();
/// assert_eq!(v, json!("33"));
/// # }
/// ```
pub struct Ptr<F>(PhantomData<F>);

impl<F> Ptr<F> {
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

impl<F, T> SerializeWith<T> for Ptr<F>
where
    F: SerializeWith<T::Target>,
    T: Deref + ?Sized,
{
    fn serialize_with<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        F::serialize_with(value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, T> for Ptr<F>
where
    F: DeserializeWith<'de, T::Target>,
    T: Deref + From<T::Target>,
    T::Target: Sized,
{
    fn deserialize_with<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        F::deserialize_with(deserializer).map(Into::into)
    }
}

#[cfg(all(feature = "std", test))]
mod tests {
    use crate::test_utils::check_serialization;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::{rc::Rc, sync::Arc};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapBox(#[serde(with = "crate::Ptr::<crate::Str>")] Box<i32>);

    #[test]
    fn ptr_adapter_works_for_box() {
        check_serialization(WrapBox(Box::new(33)), json!("33"));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapArc(#[serde(with = "crate::Ptr::<crate::Str>")] Arc<i32>);

    #[test]
    fn ptr_adapter_works_for_arc() {
        check_serialization(WrapArc(Arc::new(33)), json!("33"));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapRc(#[serde(with = "crate::Ptr::<crate::Str>")] Rc<i32>);

    #[test]
    fn ptr_adapter_works_for_rc() {
        check_serialization(WrapRc(Rc::new(33)), json!("33"));
    }
}
