// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith, WithEncoding};
use core::{fmt, marker::PhantomData};
use serde::{
    de::{SeqAccess, Visitor},
    Deserializer, Serializer,
};

/// Sequence adapter to customize how items are serialized
///
/// This adapter causes a sequence to be serialized such that its items are serialized with `F`.
///
/// # Example
/// ```
/// # #[cfg(feature = "std")] {
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "serdapt::Seq::<serdapt::Str>")] Vec<i32>);
///
/// let v = serde_json::to_value(Foo(vec![1, 2])).unwrap();
/// assert_eq!(v, json!(["1", "2"]));
/// # }
/// ```
pub struct Seq<F>(PhantomData<F>);

impl<F> Seq<F> {
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

impl<F, C, T> SerializeWith<C> for Seq<F>
where
    F: SerializeWith<T>,
    C: ?Sized,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
{
    fn serialize_with<S: Serializer>(container: &C, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(container.into_iter().map(WithEncoding::<&F, _>::from))
    }
}

impl<'de, F, C> DeserializeWith<'de, C> for Seq<F>
where
    F: DeserializeWith<'de, C::Item>,
    C: IntoIterator + FromIterator<C::Item>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<C, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SeqVisitor::<F, C>::new())
    }
}

struct SeqVisitor<F, C> {
    _f: PhantomData<F>,
    _c: PhantomData<fn() -> C>,
}

impl<F, C> SeqVisitor<F, C> {
    fn new() -> Self {
        SeqVisitor {
            _f: PhantomData,
            _c: PhantomData,
        }
    }
}

impl<'de, F, C> Visitor<'de> for SeqVisitor<F, C>
where
    F: DeserializeWith<'de, C::Item>,
    C: IntoIterator + FromIterator<C::Item>,
{
    type Value = C;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        core::iter::from_fn(|| {
            seq.next_element::<WithEncoding<F, C::Item>>()
                .map(|x| x.map(WithEncoding::into_inner))
                .transpose()
        })
        .collect()
    }
}

#[cfg(all(feature = "alloc", test))]
mod tests {
    use crate::test_utils::check_serialization;
    use alloc::{vec, vec::Vec};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Foo(#[serde(with = "crate::Seq::<crate::Str>")] Vec<i32>);

    #[test]
    fn seq_adapter_roundtrips() {
        check_serialization(Foo(vec![1, 2]), json!(["1", "2"]));
    }
}
