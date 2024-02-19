// Copyright (c) 2024 Stephane Raux. Distributed under the 0BSD license.

use crate::{DeserializeWith, SerializeWith};
use core::{cell::RefCell, marker::PhantomData};
use serde::{ser::Error, Deserializer, Serializer};

/// Adapter for cell-like types
///
/// # Example
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_json::json;
/// use std::cell::Cell;
///
/// #[derive(Deserialize, Serialize)]
/// struct IntCell(#[serde(with = "serdapt::Cell::<serdapt::Str>")] Cell<i32>);
///
/// let v = serde_json::to_value(IntCell(Cell::new(33))).unwrap();
/// assert_eq!(v, json!("33"));
/// ```
pub struct Cell<F>(PhantomData<F>);

impl<F> Cell<F> {
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

impl<F, T> SerializeWith<core::cell::Cell<T>> for Cell<F>
where
    F: SerializeWith<T>,
    T: Copy,
{
    fn serialize_with<S: Serializer>(
        value: &core::cell::Cell<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        F::serialize_with(&value.get(), serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, core::cell::Cell<T>> for Cell<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<core::cell::Cell<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        F::deserialize_with(deserializer).map(Into::into)
    }
}

impl<F, T> SerializeWith<RefCell<T>> for Cell<F>
where
    F: SerializeWith<T>,
    T: ?Sized,
{
    fn serialize_with<S: Serializer>(value: &RefCell<T>, serializer: S) -> Result<S::Ok, S::Error> {
        F::serialize_with(&*value.try_borrow().map_err(S::Error::custom)?, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, RefCell<T>> for Cell<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<RefCell<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        F::deserialize_with(deserializer).map(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::check_serialization;
    use core::cell::{Cell, RefCell};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct CellWrapper(#[serde(with = "crate::Cell::<crate::Str>")] Cell<i32>);

    #[test]
    fn adapted_cell_roundtrips() {
        check_serialization(CellWrapper(3.into()), json!("3"));
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct RefCellWrapper(#[serde(with = "crate::Cell::<crate::Str>")] RefCell<i32>);

    #[test]
    fn adapted_ref_cell_roundtrips() {
        check_serialization(RefCellWrapper(3.into()), json!("3"));
    }

    #[test]
    fn serializing_mutably_borrowed_ref_cell_returns_error() {
        let cell = RefCellWrapper(3.into());
        let mut m = cell.0.borrow_mut();
        serde_json::to_string(&cell).unwrap_err();
        *m = 4;
    }
}
