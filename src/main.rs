// because proptest emits the tests from a macro, clippy will complain about dead code
#![allow(dead_code)]

use std::collections::HashMap;

use proptest::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {}

fn quicksort<T: PartialOrd + Copy>(slice: &[T]) -> Vec<T> {
    let v = slice.to_vec();
    if v.len() <= 1 {
        return v;
    }

    let pivot = v[0];
    let mut left = Vec::new();
    let mut right = Vec::new();

    for i in v.iter().skip(1) {
        if i < &pivot {
            left.push(*i);
        } else {
            right.push(*i);
        }
    }

    let mut sorted: Vec<T> = quicksort(&left);
    sorted.push(pivot);
    sorted.extend(quicksort(&right));

    sorted
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct Student {
    name: String,
    interests: Vec<String>,
    address: String,
}

impl Arbitrary for Student {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<String>(),
            prop::collection::vec(any::<String>(), 0..100),
            any::<String>(),
        )
            .prop_map(|(name, interests, address)| Student {
                name,
                interests,
                address,
            })
            .boxed()
    }
}

type StudentMap = HashMap<String, Student>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
enum NamedColor {
    Red,
    Green,
    Blue,
}

impl Arbitrary for NamedColor {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(NamedColor::Red),
            Just(NamedColor::Green),
            Just(NamedColor::Blue),
        ]
        .boxed()
    }
}

impl NamedColor {
    fn encode(&self) -> u8 {
        match self {
            NamedColor::Red => 0,
            NamedColor::Green => 1,
            NamedColor::Blue => 2,
        }
    }

    fn decode(code: &u8) -> Result<Self, ()> {
        match code {
            0 => Ok(NamedColor::Red),
            1 => Ok(NamedColor::Green),
            2 => Ok(NamedColor::Blue),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
enum Color {
    Named(NamedColor),
    Rgb { r: u8, g: u8, b: u8 },
    Cymk { c: u8, y: u8, m: u8, k: u8 },
}

impl Arbitrary for Color {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any::<NamedColor>().prop_map(Color::Named),
            (any::<u8>(), any::<u8>(), any::<u8>()).prop_map(|(r, g, b)| Color::Rgb { r, g, b }),
            (any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>())
                .prop_map(|(c, y, m, k)| Color::Cymk { c, y, m, k }),
        ]
        .boxed()
    }
}

impl Color {
    fn encode(&self) -> [u8; 5] {
        match self {
            Color::Named(c) => [0, c.encode(), 0, 0, 0],
            Color::Rgb { r, g, b } => [1, *r, *g, *b, 0],
            Color::Cymk { c, y, m, k } => [2, *c, *y, *m, *k],
        }
    }

    fn decode(code: &[u8]) -> Result<Self, ()> {
        match code {
            [0, c, 0, 0, 0] => Ok(Color::Named(NamedColor::decode(c)?)),
            [1, r, g, b, 0] => Ok(Color::Rgb {
                r: *r,
                g: *g,
                b: *b,
            }),
            [2, c, y, m, k] => Ok(Color::Cymk {
                c: *c,
                y: *y,
                m: *m,
                k: *k,
            }),
            _ => Err(()),
        }
    }
}

proptest! {
    #[test]
    fn sort_size(unsorted in prop::collection::vec(any::<u8>(), 0..100)) {
        let sorted = quicksort(&unsorted);
        assert_eq!(unsorted.len(), sorted.len());
    }

    #[test]
    fn add_no_decrease(a in any::<u8>(), without in prop::collection::vec(any::<u8>(), 0..100)) {
        let mut with = without.clone();
        with.push(a);

        assert!(with.len() >= without.len());
    }

    // if you compile this test with debug, it will take 10+ mins to run
    #[test]
    fn map_roundtrip(m in any::<StudentMap>()) {
        let serialized = serde_json::to_string(&m).unwrap();
        let deserialized: StudentMap = serde_json::from_str(&serialized).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn color_roundtrip(c in any::<Color>()) {
        let serialized = c.encode();
        let deserialized: Color = Color::decode(&serialized).unwrap();
        assert_eq!(c, deserialized);
    }

    #[test]
    fn color_deserialize(code in prop::collection::vec(any::<u8>(), 0..5)) {
        let deserialized: Result<Color, _> = Color::decode(&code);
        if let Ok(c) = deserialized {
            let serialized = c.encode();
            assert_eq!(code, serialized);
        }
    }
}
