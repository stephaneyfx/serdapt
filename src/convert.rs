// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

/// Adapter to serialize through a [`From`] conversion
///
/// The adapter works by converting to `T` and serializing the result using `F`. When
/// deserializing, a `T` is deserialized using `F` and a conversion to the desired type is
/// performed.
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
/// use std::net::Ipv4Addr;
///
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Foo(#[serde(with = "serdapt::Convert::<u32>")] Ipv4Addr);
///
/// let foo = Foo(Ipv4Addr::new(127, 0, 0, 1));
/// let serialized = serde_json::to_value(&foo).unwrap();
/// assert_eq!(serialized, json!(0x7f000001));
/// let deserialized = serde_json::from_value::<Foo>(serialized).unwrap();
/// assert_eq!(deserialized, foo);
/// ```
pub type Convert<T, F = crate::Id> = crate::Codec<crate::Into<T, F>, crate::From<T, F>>;

/// Adapter to serialize using [`From`] on a borrow of the source value
///
/// The adapter works by converting a borrow of the source value to `T` and serializing the result
/// using `F`. When deserializing, a `T` is deserialized using `F` and converted to the source
/// type.
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct ReversedBytes(Vec<u8>);
///
/// impl From<&Vec<u8>> for ReversedBytes {
///     fn from(bytes: &Vec<u8>) -> ReversedBytes {
///         let mut bytes = bytes.clone();
///         bytes.reverse();
///         ReversedBytes(bytes)
///     }
/// }
///
/// impl From<ReversedBytes> for Vec<u8> {
///     fn from(bytes: ReversedBytes) -> Vec<u8> {
///         let mut bytes = bytes.0;
///         bytes.reverse();
///         bytes
///     }
/// }
///
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Foo(#[serde(with = "serdapt::RefConvert::<ReversedBytes>")] Vec<u8>);
///
/// let foo = Foo(vec![1, 2, 3]);
/// let serialized = serde_json::to_value(&foo).unwrap();
/// assert_eq!(serialized, json!([3, 2, 1]));
/// let deserialized = serde_json::from_value::<Foo>(serialized).unwrap();
/// assert_eq!(deserialized, foo);
/// ```
pub type RefConvert<T, F = crate::Id> =
    crate::Codec<crate::AddRef<crate::Into<T, F>>, crate::From<T, F>>;

/// Adapter to serialize through a [`TryFrom`] conversion
///
/// The adapter works by attempting a conversion to `T` and serializing the result using `F`. When
/// deserializing, a `T` is deserialized using `F` and a conversion to the desired type is
/// attempted.
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Foo {
///     #[serde(with = "serdapt::TryConvert::<u32>")]
///     c: char,
/// }
///
/// let foo = Foo { c: 'a' };
/// let serialized = serde_json::to_value(&foo).unwrap();
/// assert_eq!(serialized, json!({ "c": b'a' }));
/// let deserialized = serde_json::from_value::<Foo>(serialized).unwrap();
/// assert_eq!(deserialized, foo);
/// ```
pub type TryConvert<T, F = crate::Id> = crate::Codec<crate::TryInto<T, F>, crate::TryFrom<T, F>>;

/// Adapter to serialize using [`TryFrom`] on a borrow of the source value
///
/// The adapter works by attempting a conversion from a borrow of the source value to `T` and
/// serializing the result using `F`. When deserializing, a `T` is deserialized using `F` and
/// converted to the source type.
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Debug, PartialEq)]
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
/// impl From<i32> for StrNumber {
///     fn from(value: i32) -> Self {
///         Self(value.to_string())
///     }
/// }
///
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Foo {
///     #[serde(with = "serdapt::RefTryConvert::<i32>")]
///     n: StrNumber,
/// }
///
/// let original = Foo { n: 33.into() };
/// let v = serde_json::to_value(&original).unwrap();
/// assert_eq!(v, json!({ "n": 33 }));
/// let deserialized = serde_json::from_value::<Foo>(v).unwrap();
/// assert_eq!(deserialized, original);
/// ```
pub type RefTryConvert<T, F = crate::Id> =
    crate::Codec<crate::AddRef<crate::TryInto<T, F>>, crate::TryFrom<T, F>>;
