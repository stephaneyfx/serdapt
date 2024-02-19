// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith, WithEncoding};
use core::{
    marker::PhantomData,
    ops::{Bound, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Adapter for range-related types
///
/// The range index type is serialized with `F`.
///
/// # Example
/// ```
/// use core::ops::Range;
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo(#[serde(with = "sa::Range::<sa::Str>")] Range<u32>);
///
/// let v = serde_json::to_value(Foo(1..3)).unwrap();
/// assert_eq!(v, json!({ "start": "1", "end": "3" }));
/// ```
pub struct Range<F>(PhantomData<F>);

impl<F> Range<F> {
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

impl<F, T> SerializeWith<core::ops::Range<T>> for Range<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(
        value: &core::ops::Range<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value: core::ops::Range<WithEncoding<&F, &T>> = core::ops::Range {
            start: (&value.start).into(),
            end: (&value.end).into(),
        };
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, core::ops::Range<T>> for Range<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<core::ops::Range<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: core::ops::Range<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(core::ops::Range {
            start: r.start.into_inner(),
            end: r.end.into_inner(),
        })
    }
}

impl<F, T> SerializeWith<RangeFrom<T>> for Range<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(
        value: &RangeFrom<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value = RangeFrom {
            start: WithEncoding::<&F, &T>::from(&value.start),
        };
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, RangeFrom<T>> for Range<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<RangeFrom<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: RangeFrom<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(RangeFrom {
            start: r.start.into_inner(),
        })
    }
}

impl<F, T> SerializeWith<RangeTo<T>> for Range<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(value: &RangeTo<T>, serializer: S) -> Result<S::Ok, S::Error> {
        let value = RangeTo {
            end: WithEncoding::<&F, &T>::from(&value.end),
        };
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, RangeTo<T>> for Range<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<RangeTo<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: RangeTo<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(RangeTo {
            end: r.end.into_inner(),
        })
    }
}

impl<F, T> SerializeWith<RangeToInclusive<T>> for Range<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(
        value: &RangeToInclusive<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value = RangeTo {
            end: WithEncoding::<&F, &T>::from(&value.end),
        };
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, RangeToInclusive<T>> for Range<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<RangeToInclusive<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: RangeTo<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(RangeToInclusive {
            end: r.end.into_inner(),
        })
    }
}

impl<F, T> SerializeWith<RangeInclusive<T>> for Range<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(
        value: &RangeInclusive<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let value: RangeInclusive<WithEncoding<&F, &T>> =
            RangeInclusive::new(value.start().into(), value.end().into());
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, RangeInclusive<T>> for Range<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<RangeInclusive<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: RangeInclusive<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        let (start, end) = r.into_inner();
        Ok(RangeInclusive::new(start.into_inner(), end.into_inner()))
    }
}

impl<F, T> SerializeWith<Bound<T>> for Range<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(value: &Bound<T>, serializer: S) -> Result<S::Ok, S::Error> {
        let value: Bound<WithEncoding<&F, &T>> = match value {
            Bound::Included(x) => Bound::Included(x.into()),
            Bound::Excluded(x) => Bound::Excluded(x.into()),
            Bound::Unbounded => Bound::Unbounded,
        };
        Serialize::serialize(&value, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, Bound<T>> for Range<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<Bound<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bounds: Bound<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(match bounds {
            Bound::Included(x) => Bound::Included(x.into_inner()),
            Bound::Excluded(x) => Bound::Excluded(x.into_inner()),
            Bound::Unbounded => Bound::Unbounded,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::check_serialization;
    use core::ops::{Bound, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapRange(#[serde(with = "crate::Range::<crate::Str>")] Range<u32>);

    #[test]
    fn range_adapter_works_for_range() {
        check_serialization(WrapRange(1..3), json!({ "start": "1", "end": "3" }));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapRangeFrom(#[serde(with = "crate::Range::<crate::Str>")] RangeFrom<u32>);

    #[test]
    fn range_adapter_works_for_range_from() {
        check_serialization(WrapRangeFrom(1..), json!({ "start": "1" }));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapRangeInclusive(#[serde(with = "crate::Range::<crate::Str>")] RangeInclusive<u32>);

    #[test]
    fn range_adapter_works_for_range_inclusive() {
        check_serialization(
            WrapRangeInclusive(1..=3),
            json!({ "start": "1", "end": "3" }),
        );
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapRangeTo(#[serde(with = "crate::Range::<crate::Str>")] RangeTo<u32>);

    #[test]
    fn range_adapter_works_for_range_to() {
        check_serialization(WrapRangeTo(..3), json!({ "end": "3" }));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapRangeToInclusive(
        #[serde(with = "crate::Range::<crate::Str>")] RangeToInclusive<u32>,
    );

    #[test]
    fn range_adapter_works_for_range_to_inclusive() {
        check_serialization(WrapRangeToInclusive(..=3), json!({ "end": "3" }));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct WrapBound(#[serde(with = "crate::Range::<crate::Str>")] Bound<u32>);

    #[test]
    fn range_adapter_works_for_bound() {
        check_serialization(WrapBound(Bound::Included(3)), json!({ "Included": "3" }));
    }
}
