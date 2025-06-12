// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

//! # Overview
//! - [📦 crates.io](https://crates.io/crates/serdapt)
//! - [📖 Documentation](https://docs.rs/serdapt)
//! - [⚖ 0BSD license](https://spdx.org/licenses/0BSD.html)
//!
//! Tools to build composable adapters for `#[serde(with = ...)]`.
//!
//! `serde` allows customizing how fields are serialized when deriving `Serialize` and `Deserialize`
//! thanks to the `#[serde(with = "path")]` attribute. With such an attribute, `path::serialize`
//! and `path::deserialize` are the functions used for serialization. By using a type for `path`,
//! composable serialization adapters can be defined, e.g. to customize how items in a container
//! are serialized.
//!
//! These adapters can also simplify implementing `Serialize` and `Deserialize`.
//!
//! # Apply adapter
//! An adapter is applied by specifying the adapter path in `#[serde(with = "...")]`. The path
//! needs to be suitable as a prefix for functions, i.e. `path::serialize` and `path::deserialize`.
//! This means the turbofish is needed for generic adapters, e.g. `Outer::<Inner>`.
//!
//! ## Example
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Deserialize, PartialEq, Serialize)]
//! struct Foo {
//!     #[serde(with = "serdapt::Seq::<serdapt::Str>")]
//!     xs: Vec<i32>,
//! }
//!
//! let foo = Foo { xs: vec![3, 4] };
//! let v = serde_json::to_value(&foo).unwrap();
//! assert_eq!(v, serde_json::json!({ "xs": ["3", "4"] }));
//! assert_eq!(serde_json::from_value::<Foo>(v).unwrap(), foo);
//! ```
//!
//! # Define serialization adapter
//! 1. Define a type to represent the new adapter.
//! 1. Implement [`SerializeWith`] and [`DeserializeWith`] for this type. This allows adapter
//!    composability.
//! 1. Define `serialize` and `deserialize` inherent functions for this type, delegating to
//!    [`SerializeWith`] and [`DeserializeWith`] respectively. These are the functions the
//!    serde-generated code calls.
//!
//! ## Simple adapter example
//! ```
//! use serdapt::{DeserializeWith, SerializeWith};
//! use serde::{Deserialize, Deserializer, Serialize, Serializer};
//! use serde_json::json;
//!
//! #[derive(Deserialize, Serialize)]
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//!
//! struct Coords;
//!
//! impl Coords {
//!     fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
//!     where
//!         T: ?Sized,
//!         S: Serializer,
//!         Self: SerializeWith<T>,
//!     {
//!         Self::serialize_with(value, serializer)
//!     }
//!
//!     fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!         Self: DeserializeWith<'de, T>,
//!     {
//!         Self::deserialize_with(deserializer)
//!     }
//! }
//!
//! impl SerializeWith<(i32, i32)> for Coords {
//!     fn serialize_with<S>(&(x, y): &(i32, i32), serializer: S) -> Result<S::Ok, S::Error>
//!     where
//!         S: Serializer,
//!     {
//!         Serialize::serialize(&Point { x, y }, serializer)
//!     }
//! }
//!
//! impl<'de> DeserializeWith<'de, (i32, i32)> for Coords {
//!     fn deserialize_with<D>(deserializer: D) -> Result<(i32, i32), D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         let Point { x, y } = Deserialize::deserialize(deserializer)?;
//!         Ok((x, y))
//!     }
//! }
//!
//! #[derive(Debug, Deserialize, PartialEq, Serialize)]
//! struct Shape(#[serde(with = "serdapt::Seq::<Coords>")] Vec<(i32, i32)>);
//!
//! let original = Shape(vec![(1, 2), (3, 4)]);
//! let serialized = serde_json::to_value(&original).unwrap();
//! assert_eq!(serialized, json!([{ "x": 1, "y": 2 }, { "x": 3, "y": 4 }]));
//! let deserialized = serde_json::from_value::<Shape>(serialized).unwrap();
//! assert_eq!(deserialized, original);
//! ```
//!
//! ## Generic adapter example
//! ```
//! use core::marker::PhantomData;
//! use serdapt::{DeserializeWith, SerializeWith, WithEncoding};
//! use serde::{Deserialize, Deserializer, Serialize, Serializer};
//! use serde_json::json;
//!
//! #[derive(Debug, Deserialize, PartialEq, Serialize)]
//! struct Point<T> {
//!     x: T,
//!     y: T,
//! }
//!
//! struct Coords<F>(PhantomData<F>);
//!
//! impl<F> Coords<F> {
//!     fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
//!     where
//!         T: ?Sized,
//!         S: Serializer,
//!         Self: SerializeWith<T>,
//!     {
//!         Self::serialize_with(value, serializer)
//!     }
//!
//!     fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!         Self: DeserializeWith<'de, T>,
//!     {
//!         Self::deserialize_with(deserializer)
//!     }
//! }
//!
//! impl<F, T> SerializeWith<Point<T>> for Coords<F>
//! where
//!     F: SerializeWith<T>,
//! {
//!     fn serialize_with<S>(Point { x, y }: &Point<T>, serializer: S) -> Result<S::Ok, S::Error>
//!     where
//!         S: Serializer,
//!     {
//!         let p: Point<WithEncoding<&F, &T>> = Point {
//!             x: x.into(),
//!             y: y.into()
//!         };
//!         Serialize::serialize(&p, serializer)
//!     }
//! }
//!
//! impl<'de, F, T> DeserializeWith<'de, Point<T>> for Coords<F>
//! where
//!     F: DeserializeWith<'de, T>,
//! {
//!     fn deserialize_with<D>(deserializer: D) -> Result<Point<T>, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         let p: Point<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
//!         Ok(Point {
//!             x: p.x.into_inner(),
//!             y: p.y.into_inner(),
//!         })
//!     }
//! }
//!
//! #[derive(Debug, Deserialize, PartialEq, Serialize)]
//! struct Shape(
//!     #[serde(with = "serdapt::Seq::<Coords<serdapt::Str>>")] Vec<Point<i32>>,
//! );
//!
//! let original = Shape(vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]);
//! let serialized = serde_json::to_value(&original).unwrap();
//! assert_eq!(serialized, json!([{ "x": "1", "y": "2" }, { "x": "3", "y": "4" }]));
//! let deserialized = serde_json::from_value::<Shape>(serialized).unwrap();
//! assert_eq!(deserialized, original);
//! ```
//!
//! # Related project
//! [`serde_with`](https://crates.io/crates/serde_with) allows the same composability with the help
//! of an additional proc-macro, though it is also possible to use `#[serde(with = ...)]` directly.
//!
//! Some key differences are:
//! - `serdapt` is simpler and does not need any additional proc-macro, giving up on any ergonomics
//!   such a macro provides.
//! - It avoids a macro ordering issue that can lead to generated serialization code not using the
//!   requested adapter despite a sucessful compilation.
//! - It works seamlessly with conditional compilation.
//! - It is limited to supporting types in the standard library, with support for third-party types
//!   delegated to other crates, which solves dependency issues.
//!
//! # Contribute
//! All contributions shall be licensed under the [0BSD license](https://spdx.org/licenses/0BSD.html).

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod add_ref;
mod array;
mod bytes;
mod cell;
mod codec;
mod convert;
#[cfg(feature = "alloc")]
mod cow;
mod fold;
mod from;
mod human;
mod identity;
mod into;
mod map;
mod map_as_seq;
#[cfg(feature = "std")]
mod mutex;
mod option;
mod ptr;
mod range;
mod result;
mod reverse;
#[cfg(feature = "std")]
mod rwlock;
mod seq_as_map;
mod sequence;
mod str;
mod try_from;
mod try_into;
mod wrapping;

pub use add_ref::AddRef;
pub use array::Array;
#[cfg(feature = "alloc")]
pub use bytes::ByteVec;
pub use bytes::Bytes;
pub use cell::Cell;
pub use codec::Codec;
pub use convert::{Convert, RefConvert, RefTryConvert, TryConvert};
#[cfg(feature = "alloc")]
pub use cow::Cow;
pub use fold::Fold;
pub use from::From;
pub use human::HumanOr;
pub use identity::Id;
pub use into::Into;
pub use map::Map;
pub use map_as_seq::MapAsSeq;
#[cfg(feature = "std")]
pub use mutex::Mutex;
pub use option::Option;
pub use ptr::Ptr;
pub use range::Range;
pub use result::Result;
pub use reverse::Reverse;
#[cfg(feature = "std")]
pub use rwlock::RwLock;
pub use seq_as_map::SeqAsMap;
pub use sequence::Seq;
pub use str::Str;
pub use try_from::TryFrom;
pub use try_into::TryInto;
pub use wrapping::Wrapping;

use core::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Trait for types that can be used as serialization adapters with `#[serde(with = ...)]`
///
/// This is the foundation to build composable serialization adapters.
pub trait SerializeWith<T: ?Sized> {
    /// Serializes `value` using `serializer`
    fn serialize_with<S: Serializer>(
        value: &T,
        serializer: S,
    ) -> core::result::Result<S::Ok, S::Error>;
}

/// Trait for types that can be used as deserialization adapters with `#[serde(with = ...)]`
///
/// This is the foundation to build composable deserialization adapters.
pub trait DeserializeWith<'de, T> {
    /// Deserializes a value using `deserializer`
    fn deserialize_with<D>(deserializer: D) -> core::result::Result<T, D::Error>
    where
        D: Deserializer<'de>;
}

impl<F, T> SerializeWith<&T> for &F
where
    F: SerializeWith<T>,
    T: ?Sized,
{
    fn serialize_with<S: Serializer>(
        value: &&T,
        serializer: S,
    ) -> core::result::Result<S::Ok, S::Error> {
        F::serialize_with(*value, serializer)
    }
}

impl<F, T> SerializeWith<&mut T> for &F
where
    F: SerializeWith<T>,
    T: ?Sized,
{
    fn serialize_with<S: Serializer>(
        value: &&mut T,
        serializer: S,
    ) -> core::result::Result<S::Ok, S::Error> {
        F::serialize_with(*value, serializer)
    }
}

/// Type bundling a value and how to (de)serialize it
///
/// It allows a value to be (de)serialized with `serde` in a custom manner.
pub struct WithEncoding<F, T: ?Sized> {
    encoding: PhantomData<F>,
    value: T,
}

impl<F, T> WithEncoding<F, T> {
    /// Returns inner value
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<F, T> core::convert::From<T> for WithEncoding<F, T> {
    fn from(value: T) -> Self {
        Self {
            encoding: PhantomData,
            value,
        }
    }
}

impl<F, T> Serialize for WithEncoding<F, T>
where
    F: SerializeWith<T>,
    T: ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        F::serialize_with(&self.value, serializer)
    }
}

impl<'de, F, T> Deserialize<'de> for WithEncoding<F, T>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        F::deserialize_with(deserializer).map(|x| x.into())
    }
}

macro_rules! impl_tuple {
    ($($types:ident $adapters:ident $xs:ident,)*) => {
        impl<$($types, $adapters),*> SerializeWith<($($types,)*)> for ($($adapters,)*)
        where
            $($adapters: SerializeWith<$types>,)*
        {
            fn serialize_with<S: Serializer>(
                value: &($($types,)*),
                serializer: S,
            ) -> core::result::Result<S::Ok, S::Error> {
                let ($($xs,)*) = value;
                Serialize::serialize(
                    &($(WithEncoding::<&$adapters, _>::from($xs),)*),
                    serializer,
                )
            }
        }

        impl<'de, $($types, $adapters),*> DeserializeWith<'de, ($($types,)*)> for ($($adapters,)*)
        where
            $($adapters: DeserializeWith<'de, $types>,)*
        {
            fn deserialize_with<D>(deserializer: D) -> core::result::Result<($($types,)*), D::Error>
            where
                D: Deserializer<'de>,
            {
                let ($($xs,)*) = <($(WithEncoding<$adapters, $types>,)*) as Deserialize>::deserialize(deserializer)?;
                Ok(($($xs.value,)*))
            }
        }
    };
}

macro_rules! impl_tuples {
    (
        $($tys:ident $adapters:ident $xs:ident,)*
        @ $ty_head:ident $adapter_head:ident $x_head:ident, $($ty_tail:ident $adapter_tail:ident $x_tail:ident,)*
    ) => {
        impl_tuple!($($tys $adapters $xs,)*);
        impl_tuples!($($tys $adapters $xs,)* $ty_head $adapter_head $x_head, @ $($ty_tail $adapter_tail $x_tail,)*);

    };
    ($($tys:ident $adapters:ident $xs:ident,)* @) => {
        impl_tuple!($($tys $adapters $xs,)*);

    };
    ($($tys:ident $adapters:ident $xs:ident,)*) => {
        impl_tuples!(@ $($tys $adapters $xs,)*);
    };
}

impl_tuples!(
    T0 A0 x0,
    T1 A1 x1,
    T2 A2 x2,
    T3 A3 x3,
    T4 A4 x4,
    T5 A5 x5,
    T6 A6 x6,
    T7 A7 x7,
    T8 A8 x8,
    T9 A9 x9,
    T10 A10 x10,
    T11 A11 x11,
    T12 A12 x12,
    T13 A13 x13,
    T14 A14 x14,
    T15 A15 x15,
);

#[cfg(test)]
mod test_utils {
    use core::fmt::Debug;
    use serde::{de::DeserializeOwned, Serialize};
    use serde_json::Value;

    #[track_caller]
    pub(crate) fn check_serialization<T>(x: T, expected: Value)
    where
        T: PartialEq + Debug + Serialize + DeserializeOwned,
    {
        let actual = serde_json::to_value(&x).unwrap();
        assert_eq!(actual, expected);
        let deserialized = serde_json::from_value::<T>(actual).unwrap();
        assert_eq!(x, deserialized);
    }
}
