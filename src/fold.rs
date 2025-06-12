use crate::{DeserializeWith, Id, WithEncoding};
use core::{fmt, marker::PhantomData, ops::Add};
use serde::{
    de::{SeqAccess, Visitor},
    Deserializer,
};

/// Sequence adapter to fold over its items when deserializing
///
/// This adapter causes a sequence to be deserialized such that its items are deserialized with `F`
/// and folded over with `A::add` and `A::default()` as the initial accumulator.
///
/// This avoids collecting when only the result of the fold is needed.
///
/// # Example
/// ```
/// # #[cfg(feature = "std")]
/// use serde::Deserialize;
/// use serde_json::json;
///
/// #[derive(Deserialize)]
/// struct Foo(#[serde(with = "serdapt::Fold::<i32, i32>")] i32);
///
/// let Foo(sum) = serde_json::from_value::<Foo>(json!([1, 2, 3])).unwrap();
/// assert_eq!(sum, 1 + 2 + 3);
/// ```
pub struct Fold<T, A, F = Id> {
    _convert: PhantomData<fn(T)>,
    _acc: PhantomData<fn() -> A>,
    _f: PhantomData<F>,
}

impl<T, A, F> Fold<T, A, F> {
    /// Deserializes value with adapter
    pub fn deserialize<'de, U, D>(deserializer: D) -> Result<U, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, U>,
    {
        Self::deserialize_with(deserializer)
    }
}

impl<'de, T, A, F> DeserializeWith<'de, A> for Fold<T, A, F>
where
    F: DeserializeWith<'de, T>,
    A: Default + Add<T, Output = A>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<A, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(FoldVisitor::<T, A, F>::new())
    }
}

struct FoldVisitor<T, A, F> {
    _convert: PhantomData<fn(T)>,
    _acc: PhantomData<fn() -> A>,
    _f: PhantomData<F>,
}

impl<T, A, F> FoldVisitor<T, A, F> {
    fn new() -> Self {
        Self {
            _convert: PhantomData,
            _acc: PhantomData,
            _f: PhantomData,
        }
    }
}

impl<'de, T, A, F> Visitor<'de> for FoldVisitor<T, A, F>
where
    F: DeserializeWith<'de, T>,
    A: Default + Add<T, Output = A>,
{
    type Value = A;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a sequence")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        core::iter::from_fn(|| seq.next_element::<WithEncoding<F, T>>().transpose())
            .try_fold(A::default(), |acc, x| Ok(acc + x?.into_inner()))
    }
}
