// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith, WithEncoding};
use core::{fmt, marker::PhantomData, mem::MaybeUninit};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserializer, Serializer,
};

/// Adapter to customize how array items are serialized
///
/// This adapter serializes the array as a serde tuple. This implies the length is statically known
/// without looking at the serialized data when deserializing.
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo {
///     #[serde(with = "serdapt::Array::<serdapt::Str>")]
///     coords: [i32; 2],
/// }
///
/// let foo = Foo { coords: [1, 2] };
/// let v = serde_json::to_value(&foo).unwrap();
/// assert_eq!(v, json!({ "coords": ["1", "2"] }));
/// ```
pub struct Array<F>(PhantomData<F>);

impl<F> Array<F> {
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

impl<const N: usize, F, T> SerializeWith<[T; N]> for Array<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S: Serializer>(value: &[T; N], serializer: S) -> Result<S::Ok, S::Error> {
        let mut out = serializer.serialize_tuple(N)?;
        value
            .iter()
            .try_for_each(|x| out.serialize_element(&WithEncoding::<&F, &T>::from(x)))?;
        out.end()
    }
}

impl<'de, const N: usize, F, T> DeserializeWith<'de, [T; N]> for Array<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<N, F, T>::new())
    }
}

struct ArrayVisitor<const N: usize, F, T> {
    _f: PhantomData<F>,
    _a: PhantomData<fn() -> [T; N]>,
}

impl<const N: usize, F, T> ArrayVisitor<N, F, T> {
    fn new() -> Self {
        Self {
            _f: PhantomData,
            _a: PhantomData,
        }
    }
}

struct ExpectedArrayLength<const N: usize>;

impl<const N: usize> serde::de::Expected for ExpectedArrayLength<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "an array of length {N}")
    }
}

impl<'de, const N: usize, F, T> Visitor<'de> for ArrayVisitor<N, F, T>
where
    F: DeserializeWith<'de, T>,
{
    type Value = [T; N];

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        serde::de::Expected::fmt(&ExpectedArrayLength::<N>, f)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        MaybeUninitArray::<N, T>::new().fill(core::iter::from_fn(|| {
            seq.next_element::<WithEncoding<F, T>>()
                .map(|x| x.map(WithEncoding::into_inner))
                .transpose()
        }))
    }
}

struct MaybeUninitArray<const N: usize, T> {
    items: [MaybeUninit<T>; N],
    count: usize,
}

impl<const N: usize, T> MaybeUninitArray<N, T> {
    fn new() -> Self {
        Self {
            items: core::array::from_fn(|_| MaybeUninit::uninit()),
            count: 0,
        }
    }

    fn fill<I, E>(&mut self, it: I) -> Result<[T; N], E>
    where
        I: IntoIterator<Item = Result<T, E>>,
        E: serde::de::Error,
    {
        self.items.iter_mut().zip(it).try_for_each(|(out, x)| {
            out.write(x?);
            self.count += 1;
            Ok(())
        })?;
        if self.count != N {
            return Err(E::invalid_length(self.count, &ExpectedArrayLength::<N>));
        }

        self.count = 0;
        let items = core::mem::replace(
            &mut self.items,
            core::array::from_fn(|_| MaybeUninit::uninit()),
        );

        // Safety: All items in the array have been written to at this point
        Ok(items.map(|x| unsafe { x.assume_init() }))
    }
}

impl<const N: usize, T> Drop for MaybeUninitArray<N, T> {
    fn drop(&mut self) {
        // Safety: `count` indicates the number of items that were initialized
        self.items.iter_mut().take(self.count).for_each(|x| unsafe {
            x.assume_init_drop();
        });
    }
}

#[cfg(all(feature = "alloc", test))]
mod tests {
    use crate::test_utils::check_serialization;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Foo<const N: usize> {
        #[serde(with = "crate::Array::<crate::Str>")]
        xs: [i32; N],
    }

    #[test]
    fn empty_array_roundtrips() {
        check_serialization(Foo { xs: [] }, json!({ "xs": [] }));
    }

    #[test]
    fn array_roundtrips() {
        check_serialization(Foo { xs: [1, 2, 3] }, json!({ "xs": ["1", "2", "3"] }));
    }
}
