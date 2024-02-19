// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::SerializeWith;
use core::marker::PhantomData;
use serde::Serializer;

/// Adapter to serialize values of type `T` with an adapter `F` that expects `&T`
///
/// # Example
/// ```
/// use serde::Serialize;
/// use serde_json::json;
///
/// struct StrNumber(String);
///
/// impl TryFrom<&StrNumber> for i32 {
///     type Error = std::num::ParseIntError;
///
///     fn try_from(value: &StrNumber) -> Result<Self, Self::Error> {
///         value.0.parse()
///     }
/// }
///
/// #[derive(Serialize)]
/// struct Foo {
///     #[serde(with = "serdapt::AddRef::<serdapt::TryInto<i32>>")]
///     n: StrNumber,
/// }
///
/// let v = serde_json::to_value(Foo {
///     n: StrNumber("33".into()),
/// })
/// .unwrap();
/// assert_eq!(v, json!({ "n": 33 }));
/// ```
pub struct AddRef<F>(PhantomData<F>);

impl<F> AddRef<F> {
    /// Serializes value with adapter
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ?Sized,
        S: Serializer,
        Self: SerializeWith<T>,
    {
        Self::serialize_with(value, serializer)
    }
}

impl<F, T> SerializeWith<T> for AddRef<F>
where
    F: for<'a> SerializeWith<&'a T>,
    T: ?Sized,
{
    fn serialize_with<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        F::serialize_with(&value, serializer)
    }
}
