use std::collections::HashMap;

use proptest::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {}

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

proptest! {
    #[test]
    fn sort_size(unsorted in prop::collection::vec(any::<u8>(), 0..100)) {
        let mut sorted = unsorted.clone();
        sorted.sort();
        assert!(unsorted.len() == sorted.len());
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
        let serialized = bincode::serialize(&c).unwrap();
        let deserialized: Color = bincode::deserialize(&serialized).unwrap();
        assert_eq!(c, deserialized);
    }

    #[test]
    fn color_deserialize(code in prop::collection::vec(any::<u8>(), 0..8)) {
        let deserialized: Result<Color, _> = bincode::deserialize(&code);
        if let Ok(c) = deserialized {
            let serialized = bincode::serialize(&c).unwrap();
            assert_eq!(code, serialized);
        }
    }
}
