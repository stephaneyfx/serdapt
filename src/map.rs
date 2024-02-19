// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith, WithEncoding};
use core::{fmt, marker::PhantomData};
use serde::{
    de::{MapAccess, Visitor},
    Deserializer, Serializer,
};

/// Map adapter to customize how keys and values are serialized
///
/// This adapter causes a map to be serialized such that its keys are serialized with `F` and its
/// values are serialized with `G`.
///
/// # Example
/// ```
/// # #[cfg(feature = "std")] {
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
/// use std::collections::BTreeMap;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "sa::Map::<sa::Str, sa::Bytes>")] BTreeMap<i32, Vec<u8>>);
///
/// let foo = Foo(BTreeMap::from_iter([(33, vec![0, 1]), (34, vec![2, 3])]));
/// let v = serde_json::to_value(&foo).unwrap();
/// assert_eq!(v, json!({ "33": [0, 1], "34": [2, 3] }));
/// # }
/// ```
pub struct Map<F, G>(PhantomData<(F, G)>);

impl<F, G> Map<F, G> {
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

impl<F, G, C, K, V> SerializeWith<C> for Map<F, G>
where
    F: SerializeWith<K>,
    G: SerializeWith<V>,
    C: ?Sized,
    for<'a> &'a C: IntoIterator<Item = (&'a K, &'a V)>,
{
    fn serialize_with<S: Serializer>(container: &C, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(container.into_iter().map(|(k, v)| {
            (
                WithEncoding::<&F, _>::from(k),
                WithEncoding::<&G, _>::from(v),
            )
        }))
    }
}

impl<'de, F, G, C, K, V> DeserializeWith<'de, C> for Map<F, G>
where
    F: DeserializeWith<'de, K>,
    G: DeserializeWith<'de, V>,
    C: IntoIterator<Item = (K, V)> + FromIterator<(K, V)>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<C, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MapVisitor::<F, G, C>::new())
    }
}

struct MapVisitor<F, G, C> {
    _f: PhantomData<(F, G)>,
    _c: PhantomData<fn() -> C>,
}

impl<F, G, C> MapVisitor<F, G, C> {
    fn new() -> Self {
        MapVisitor {
            _f: PhantomData,
            _c: PhantomData,
        }
    }
}

impl<'de, F, G, C, K, V> Visitor<'de> for MapVisitor<F, G, C>
where
    F: DeserializeWith<'de, K>,
    G: DeserializeWith<'de, V>,
    C: IntoIterator<Item = (K, V)> + FromIterator<(K, V)>,
{
    type Value = C;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        core::iter::from_fn(|| {
            map.next_entry::<WithEncoding<F, K>, WithEncoding<G, V>>()
                .map(|x| x.map(|(k, v)| (k.into_inner(), v.into_inner())))
                .transpose()
        })
        .collect()
    }
}

#[cfg(all(feature = "std", test))]
mod tests {
    use crate::{self as sa, test_utils::check_serialization};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::collections::{BTreeMap, HashMap};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapHashMap(
        #[serde(with = "sa::Map::<sa::Str, sa::Array<sa::Str>>")] HashMap<i32, [u8; 2]>,
    );

    #[test]
    fn map_adapter_works_with_hash_map() {
        check_serialization(
            WrapHashMap(HashMap::from_iter([(33, [0, 1]), (34, [0, 2])])),
            json!({ "33": ["0", "1"], "34": ["0", "2"] }),
        );
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapBTreeMap(
        #[serde(with = "sa::Map::<sa::Str, sa::Array<sa::Str>>")] BTreeMap<i32, [u8; 2]>,
    );

    #[test]
    fn map_adapter_works_with_btree_map() {
        check_serialization(
            WrapBTreeMap(BTreeMap::from_iter([(33, [0, 1]), (34, [0, 2])])),
            json!({ "33": ["0", "1"], "34": ["0", "2"] }),
        );
    }
}
