<!-- cargo-sync-readme start -->

# Overview
- [📦 crates.io](https://crates.io/crates/serdapt)
- [📖 Documentation](https://docs.rs/serdapt)
- [⚖ 0BSD license](https://spdx.org/licenses/0BSD.html)

Tools to build composable adapters for `#[serde(with = ...)]`.

`serde` allows customizing how fields are serialized when deriving `Serialize` and `Deserialize`
thanks to the `#[serde(with = "path")]` attribute. With such an attribute, `path::serialize`
and `path::deserialize` are the functions used for serialization. By using a type for `path`,
composable serialization adapters can be defined, e.g. to customize how items in a container
are serialized.

These adapters can also simplify implementing `Serialize` and `Deserialize`.

# Apply adapter
An adapter is applied by specifying the adapter path in `#[serde(with = "...")]`. The path
needs to be suitable as a prefix for functions, i.e. `path::serialize` and `path::deserialize`.
This means the turbofish is needed for generic adapters, e.g. `Outer::<Inner>`.

## Example
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Foo {
    #[serde(with = "serdapt::Seq::<serdapt::Str>")]
    xs: Vec<i32>,
}

let foo = Foo { xs: vec![3, 4] };
let v = serde_json::to_value(&foo).unwrap();
assert_eq!(v, serde_json::json!({ "xs": ["3", "4"] }));
assert_eq!(serde_json::from_value::<Foo>(v).unwrap(), foo);
```

# Define serialization adapter
1. Define a type to represent the new adapter.
1. Implement [`SerializeWith`] and [`DeserializeWith`] for this type. This allows adapter
   composability.
1. Define `serialize` and `deserialize` inherent functions for this type, delegating to
   [`SerializeWith`] and [`DeserializeWith`] respectively. These are the functions the
   serde-generated code calls.

## Simple adapter example
```rust
use serdapt::{DeserializeWith, SerializeWith};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;

#[derive(Deserialize, Serialize)]
struct Point {
    x: i32,
    y: i32,
}

struct Coords;

impl Coords {
    fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ?Sized,
        S: Serializer,
        Self: SerializeWith<T>,
    {
        Self::serialize_with(value, serializer)
    }

    fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, T>,
    {
        Self::deserialize_with(deserializer)
    }
}

impl SerializeWith<(i32, i32)> for Coords {
    fn serialize_with<S>(&(x, y): &(i32, i32), serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&Point { x, y }, serializer)
    }
}

impl<'de> DeserializeWith<'de, (i32, i32)> for Coords {
    fn deserialize_with<D>(deserializer: D) -> Result<(i32, i32), D::Error>
    where
        D: Deserializer<'de>,
    {
        let Point { x, y } = Deserialize::deserialize(deserializer)?;
        Ok((x, y))
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Shape(#[serde(with = "serdapt::Seq::<Coords>")] Vec<(i32, i32)>);

let original = Shape(vec![(1, 2), (3, 4)]);
let serialized = serde_json::to_value(&original).unwrap();
assert_eq!(serialized, json!([{ "x": 1, "y": 2 }, { "x": 3, "y": 4 }]));
let deserialized = serde_json::from_value::<Shape>(serialized).unwrap();
assert_eq!(deserialized, original);
```

## Generic adapter example
```rust
use core::marker::PhantomData;
use serdapt::{DeserializeWith, SerializeWith, WithEncoding};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Point<T> {
    x: T,
    y: T,
}

struct Coords<F>(PhantomData<F>);

impl<F> Coords<F> {
    fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ?Sized,
        S: Serializer,
        Self: SerializeWith<T>,
    {
        Self::serialize_with(value, serializer)
    }

    fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        Self: DeserializeWith<'de, T>,
    {
        Self::deserialize_with(deserializer)
    }
}

impl<F, T> SerializeWith<Point<T>> for Coords<F>
where
    F: SerializeWith<T>,
{
    fn serialize_with<S>(Point { x, y }: &Point<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let p: Point<WithEncoding<&F, &T>> = Point {
            x: x.into(),
            y: y.into()
        };
        Serialize::serialize(&p, serializer)
    }
}

impl<'de, F, T> DeserializeWith<'de, Point<T>> for Coords<F>
where
    F: DeserializeWith<'de, T>,
{
    fn deserialize_with<D>(deserializer: D) -> Result<Point<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p: Point<WithEncoding<F, T>> = Deserialize::deserialize(deserializer)?;
        Ok(Point {
            x: p.x.into_inner(),
            y: p.y.into_inner(),
        })
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Shape(
    #[serde(with = "serdapt::Seq::<Coords<serdapt::Str>>")] Vec<Point<i32>>,
);

let original = Shape(vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]);
let serialized = serde_json::to_value(&original).unwrap();
assert_eq!(serialized, json!([{ "x": "1", "y": "2" }, { "x": "3", "y": "4" }]));
let deserialized = serde_json::from_value::<Shape>(serialized).unwrap();
assert_eq!(deserialized, original);
```

# Related project
[`serde_with`](https://crates.io/crates/serde_with) allows the same composability with the help
of an additional proc-macro, though it is also possible to use `#[serde(with = ...)]` directly.

Some key differences are:
- `serdapt` is simpler and does not need any additional proc-macro, giving up on any ergonomics
  such a macro provides.
- It avoids a macro ordering issue that can lead to generated serialization code not using the
  requested adapter despite a sucessful compilation.
- It works seamlessly with conditional compilation.
- It is limited to supporting types in the standard library, with support for third-party types
  delegated to other crates, which solves dependency issues.

# Contribute
All contributions shall be licensed under the [0BSD license](https://spdx.org/licenses/0BSD.html).

<!-- cargo-sync-readme end -->
