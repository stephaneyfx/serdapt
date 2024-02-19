// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, Id, SerializeWith, WithEncoding};
use core::marker::PhantomData;
use serde::{Deserializer, Serializer};

/// Adapter to serialize a map as a sequence
///
/// `F` is used to serialize keys and `G` is used to serialize values.
///
/// # Example
/// ```
/// # #[cfg(feature = "std")] {
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
/// use std::collections::BTreeMap;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "serdapt::MapAsSeq::<serdapt::Id>")] BTreeMap<String, i32>);
///
/// let v = serde_json::to_value(Foo(BTreeMap::from_iter([("foo".into(), 33)]))).unwrap();
/// assert_eq!(v, json!([["foo", 33]]));
/// # }
/// ```
pub struct MapAsSeq<F = Id, G = Id>(PhantomData<(F, G)>);

impl<F, G> MapAsSeq<F, G> {
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

impl<F, G, C, K, V> SerializeWith<C> for MapAsSeq<F, G>
where
    F: SerializeWith<K>,
    G: SerializeWith<V>,
    C: ?Sized,
    for<'a> &'a C: IntoIterator<Item = (&'a K, &'a V)>,
{
    fn serialize_with<S: Serializer>(container: &C, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(container.into_iter().map(WithEncoding::<(&F, &G), _>::from))
    }
}

impl<'de, F, G, C, K, V> DeserializeWith<'de, C> for MapAsSeq<F, G>
where
    F: DeserializeWith<'de, K>,
    G: DeserializeWith<'de, V>,
    C: IntoIterator<Item = (K, V)> + FromIterator<(K, V)>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<C, D::Error>
    where
        D: Deserializer<'de>,
    {
        crate::Seq::<(F, G)>::deserialize_with(deserializer)
    }
}

#[cfg(all(feature = "alloc", test))]
mod tests {
    use crate::test_utils::check_serialization;
    use alloc::collections::BTreeMap;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Foo(#[serde(with = "crate::MapAsSeq::<crate::Id>")] BTreeMap<String, i32>);

    #[test]
    fn map_as_seq_adapter_roundtrips() {
        check_serialization(
            Foo(BTreeMap::from_iter([("foo".into(), 33)])),
            json!([["foo", 33]]),
        );
    }
}
