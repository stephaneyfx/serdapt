// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use core::marker::PhantomData;
use serde::{Deserializer, Serializer};

/// Adapter for custom serialization when the serialization format is human-readable
///
/// If the serialization format is human-readable, `F` is used to serialize. Otherwise `G` is used.
///
/// # Example
/// ```
/// use bincode::Options as _;
/// use std::{fmt::{self, Display}, str::FromStr};
/// use serdapt as sa;
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
///
/// struct Color(u32);
///
/// impl Display for Color {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "#{:06x}", self.0)
///     }
/// }
///
/// impl FromStr for Color {
///     type Err = std::num::ParseIntError;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         u32::from_str_radix(s.trim_start_matches('#'), 16).map(Color)
///     }
/// }
///
/// impl From<u32> for Color {
///     fn from(n: u32) -> Color {
///         Color(n)
///     }
/// }
///
/// impl From<Color> for u32 {
///     fn from(c: Color) -> u32 {
///         c.0
///     }
/// }
///
/// #[derive(Debug, Deserialize, PartialEq, Serialize)]
/// struct Palette {
///     #[serde(with = "sa::Seq::<sa::HumanOr<sa::Convert<Color, sa::Str>, sa::Id>>")]
///     colors: Vec<u32>,
/// }
///
/// let palette = Palette { colors: vec![0x000000, 0xffffff] };
/// let serialized = serde_json::to_value(&palette).unwrap();
/// assert_eq!(serialized, json!({ "colors": ["#000000", "#ffffff"] }));
/// let deserialized = serde_json::from_value::<Palette>(serialized).unwrap();
/// assert_eq!(deserialized, palette);
/// let bincode_config = bincode::options().with_fixint_encoding();
/// let serialized = bincode_config.serialize(&palette).unwrap();
/// assert_eq!(
///     serialized,
///     2u64
///         .to_le_bytes()
///         .into_iter()
///         .chain(0u32.to_le_bytes())
///         .chain(0xffffffu32.to_le_bytes())
///         .collect::<Vec<_>>(),
/// );
/// let deserialized = bincode_config.deserialize::<Palette>(&serialized).unwrap();
/// assert_eq!(deserialized, palette);
/// ```
pub struct HumanOr<F, G>(PhantomData<(F, G)>);

impl<F, G> HumanOr<F, G> {
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

impl<F, G, T> SerializeWith<T> for HumanOr<F, G>
where
    F: SerializeWith<T>,
    G: SerializeWith<T>,
    T: ?Sized,
{
    fn serialize_with<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            F::serialize_with(value, serializer)
        } else {
            G::serialize_with(value, serializer)
        }
    }
}

impl<'de, F, G, T> DeserializeWith<'de, T> for HumanOr<F, G>
where
    F: DeserializeWith<'de, T>,
    G: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            F::deserialize_with(deserializer)
        } else {
            G::deserialize_with(deserializer)
        }
    }
}
