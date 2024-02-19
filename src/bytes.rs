// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, boxed::Box, rc::Rc, string::String, vec::Vec};
use core::{
    fmt::{self, Display},
    marker::PhantomData,
};
use serde::{de::Visitor, Deserializer, Serializer};

/// Adapter for contiguous byte sequences
///
/// If a type is not supported by this adapter and is convertible from `Vec<u8>`, the [`ByteVec`]
/// adapter should be used instead.
///
/// This allows optimized handling of byte sequences when serializing. This is similar to
/// [`serde_bytes`](https://docs.rs/serde_bytes). Practically, this adapter allows serialization of
/// bytes to go through [`Serializer::serialize_bytes`] and deserialization through
/// [`Deserializer::deserialize_bytes`] or [`Deserializer::deserialize_byte_buf`].
///
/// This adapter always serializes as a serde variable-length byte sequence, even if the collection
/// type to serialize has a statically known length.
///
/// # Example
/// ```
/// # #[cfg(feature = "std")] {
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo {
///     #[serde(with = "serdapt::Bytes")]
///     bytes: Vec<u8>,
/// }
/// # }
/// ```
pub struct Bytes;

impl Bytes {
    /// Serializes value as bytes
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ?Sized,
        S: Serializer,
        Self: SerializeWith<T>,
    {
        Self::serialize_with(value, serializer)
    }

    /// Deserializes value from bytes
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, T>,
    {
        Self::deserialize_with(deserializer)
    }
}

impl<T> SerializeWith<T> for Bytes
where
    T: AsRef<[u8]> + ?Sized,
{
    fn serialize_with<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(value.as_ref())
    }
}

#[cfg(feature = "alloc")]
impl<'de> DeserializeWith<'de, Vec<u8>> for Bytes {
    fn deserialize_with<D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(VecVisitor)
    }
}

#[cfg(feature = "alloc")]
impl<'de> DeserializeWith<'de, Box<[u8]>> for Bytes {
    fn deserialize_with<D>(deserializer: D) -> Result<Box<[u8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Bytes as DeserializeWith<'de, Vec<u8>>>::deserialize_with(deserializer).map(Into::into)
    }
}

#[cfg(feature = "alloc")]
impl<'de> DeserializeWith<'de, Rc<[u8]>> for Bytes {
    fn deserialize_with<D>(deserializer: D) -> Result<Rc<[u8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Bytes as DeserializeWith<'de, Vec<u8>>>::deserialize_with(deserializer).map(Into::into)
    }
}

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
impl<'de> DeserializeWith<'de, alloc::sync::Arc<[u8]>> for Bytes {
    fn deserialize_with<D>(deserializer: D) -> Result<alloc::sync::Arc<[u8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Bytes as DeserializeWith<'de, Vec<u8>>>::deserialize_with(deserializer).map(Into::into)
    }
}

impl<'de, const N: usize> DeserializeWith<'de, [u8; N]> for Bytes {
    fn deserialize_with<D>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ArrayVisitor::<N>)
    }
}

impl<'de: 'a, 'a> DeserializeWith<'de, &'a [u8]> for Bytes {
    fn deserialize_with<D>(deserializer: D) -> Result<&'a [u8], D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(SliceVisitor::default())
    }
}

impl<'de: 'a, 'a, const N: usize> DeserializeWith<'de, &'a [u8; N]> for Bytes {
    fn deserialize_with<D>(deserializer: D) -> Result<&'a [u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = deserializer.deserialize_bytes(SliceVisitor::default())?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::invalid_length(bytes.len(), &ArrayVisitor::<N>))
    }
}

#[cfg(feature = "alloc")]
impl<'de: 'a, 'a> DeserializeWith<'de, Cow<'a, [u8]>> for Bytes {
    fn deserialize_with<D>(deserializer: D) -> Result<Cow<'a, [u8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(CowVisitor::default())
    }
}

#[cfg(feature = "alloc")]
struct VecVisitor;

#[cfg(feature = "alloc")]
impl<'de> Visitor<'de> for VecVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bytes")
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_vec())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into_bytes())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.as_bytes().to_vec())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(4096));
        while let Some(b) = seq.next_element()? {
            bytes.push(b);
        }
        Ok(bytes)
    }
}

struct ArrayVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for ArrayVisitor<N> {
    type Value = [u8; N];

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a byte array of {N} bytes")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.try_into().map_err(|_| E::invalid_length(v.len(), &self))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v.as_bytes())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut bytes = [0u8; N];
        bytes.iter_mut().enumerate().try_for_each(|(i, out)| {
            *out = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
            Ok(())
        })?;
        Ok(bytes)
    }
}

#[derive(Default)]
struct SliceVisitor<'a>(PhantomData<&'a ()>);

impl<'de: 'a, 'a> Visitor<'de> for SliceVisitor<'a> {
    type Value = &'a [u8];

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bytes")
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.as_bytes())
    }
}

#[cfg(feature = "alloc")]
#[derive(Default)]
struct CowVisitor<'a>(PhantomData<&'a ()>);

#[cfg(feature = "alloc")]
impl<'de: 'a, 'a> Visitor<'de> for CowVisitor<'a> {
    type Value = Cow<'a, [u8]>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bytes")
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Borrowed(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Owned(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Owned(v.to_vec()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Borrowed(v.as_bytes()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Owned(v.into_bytes()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Owned(v.as_bytes().to_vec()))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        VecVisitor.visit_seq(seq).map(Into::into)
    }
}

/// Adapter for contiguous byte sequences that can be converted from `Vec<u8>`
///
/// [`Bytes`] is preferred if the type to serialize is an array. It avoids allocating a `Vec` when
/// deserializing.
///
/// This allows optimized handling of byte sequences when serializing. This is similar to
/// [`serde_bytes`](https://docs.rs/serde_bytes). Practically, this adapter allows serialization of
/// bytes to go through [`Serializer::serialize_bytes`] and deserialization through
/// [`Deserializer::deserialize_bytes`] or [`Deserializer::deserialize_byte_buf`].
///
/// This adapter always serializes as a serde variable-length byte sequence, even if the collection
/// type to serialize has a statically known length.
///
/// # Example
/// ```
/// # #[cfg(feature = "std")] {
/// use serde::{Deserialize, Serialize};
///
/// struct ByteWrapper(Vec<u8>);
///
/// impl From<Vec<u8>> for ByteWrapper {
///     fn from(v: Vec<u8>) -> Self {
///         Self(v)
///     }
/// }
///
/// impl AsRef<[u8]> for ByteWrapper {
///     fn as_ref(&self) -> &[u8] {
///         &self.0
///     }
/// }
///
/// #[derive(Deserialize, Serialize)]
/// struct Foo {
///     #[serde(with = "serdapt::ByteVec")]
///     bytes: ByteWrapper,
/// }
/// # }
/// ```
#[cfg(feature = "alloc")]
pub struct ByteVec;

impl ByteVec {
    /// Serializes value as bytes
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ?Sized,
        S: Serializer,
        Self: SerializeWith<T>,
    {
        Self::serialize_with(value, serializer)
    }

    /// Deserializes value from bytes
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, T>,
    {
        Self::deserialize_with(deserializer)
    }
}

#[cfg(feature = "alloc")]
impl<T> SerializeWith<T> for ByteVec
where
    T: AsRef<[u8]> + ?Sized,
{
    fn serialize_with<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        Bytes::serialize_with(value, serializer)
    }
}

#[cfg(feature = "alloc")]
impl<'de, T> DeserializeWith<'de, T> for ByteVec
where
    T: TryFrom<Vec<u8>>,
    T::Error: Display,
{
    fn deserialize_with<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Bytes::deserialize_with(deserializer)?;
        bytes.try_into().map_err(serde::de::Error::custom)
    }
}

#[cfg(all(feature = "alloc", test))]
mod tests {
    use crate::test_utils::check_serialization;
    use alloc::{borrow::Cow, boxed::Box, rc::Rc, vec, vec::Vec};
    use core::fmt::Debug;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct VecWrapper(#[serde(with = "crate::Bytes")] Vec<u8>);

    #[test]
    fn byte_vec_roundtrips() {
        check_serialization(VecWrapper(vec![1, 2, 3]), json!([1, 2, 3]));
    }

    #[test]
    fn empty_vec_roundtrips() {
        check_serialization(VecWrapper(Vec::new()), json!([]));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct SliceWrapper<'a>(#[serde(with = "crate::Bytes")] &'a [u8]);

    #[test]
    fn byte_slice_roundtrips() {
        let original = SliceWrapper(b"foobar");
        let serialized = bincode::serialize(&original).unwrap();
        let deserialized = bincode::deserialize::<SliceWrapper<'_>>(&serialized).unwrap();
        assert_eq!(deserialized, original);
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct BoxWrapper(#[serde(with = "crate::Bytes")] Box<[u8]>);

    #[test]
    fn boxed_bytes_roundtrip() {
        check_serialization(BoxWrapper(vec![1, 2, 3].into()), json!([1, 2, 3]));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct RcWrapper(#[serde(with = "crate::Bytes")] Rc<[u8]>);

    #[test]
    fn rced_bytes_roundtrip() {
        check_serialization(RcWrapper(vec![1, 2, 3].into()), json!([1, 2, 3]));
    }

    #[cfg(target_has_atomic = "ptr")]
    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct ArcWrapper(#[serde(with = "crate::Bytes")] alloc::sync::Arc<[u8]>);

    #[cfg(target_has_atomic = "ptr")]
    #[test]
    fn arced_bytes_roundtrip() {
        check_serialization(ArcWrapper(vec![1, 2, 3].into()), json!([1, 2, 3]));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct ArrayWrapper<const N: usize>(#[serde(with = "crate::Bytes")] [u8; N]);

    #[test]
    fn byte_array_roundtrips() {
        check_serialization(ArrayWrapper([1, 2, 3]), json!([1, 2, 3]));
    }

    #[test]
    fn empty_byte_array_roundtrips() {
        check_serialization(ArrayWrapper([]), json!([]));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct RefArrayWrapper<'a, const N: usize>(
        #[serde(borrow = "'a", with = "crate::Bytes")] &'a [u8; N],
    );

    #[test]
    fn by_ref_byte_array_roundtrips() {
        let original = RefArrayWrapper(b"foobar");
        let serialized = bincode::serialize(&original).unwrap();
        let deserialized = bincode::deserialize::<RefArrayWrapper<'_, 6>>(&serialized).unwrap();
        assert_eq!(deserialized, original);
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct CowWrapper<'a>(#[serde(borrow = "'a", with = "crate::Bytes")] Cow<'a, [u8]>);

    #[test]
    fn cow_bytes_roundtrip() {
        let original = CowWrapper(Cow::Borrowed(b"foobar"));
        let serialized = bincode::serialize(&original).unwrap();
        let deserialized = bincode::deserialize::<CowWrapper<'_>>(&serialized).unwrap();
        assert_eq!(deserialized, original);
        let CowWrapper(Cow::Borrowed(_)) = deserialized else {
            panic!("Expected Cow::Borrowed");
        };
    }
}
