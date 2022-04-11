mod all_indexes;
mod collections;
mod nodes;
mod primitives;
mod traits;

pub mod sequential_array;

pub use all_indexes::{deserialize_all, serialize_all};
pub use collections::serialize_string_with_limit;
pub use traits::*;

#[cfg(test)]
use wasm_bindgen_test::wasm_bindgen_test;

#[test]
#[wasm_bindgen_test]
fn test_serialize_deserialize_wasm() {
    test_serialize_and_deserialize().unwrap();
}

#[test]
fn test_serialize_and_deserialize() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{
        classic_indexes::ClassicIndexes,
        data::{optimize, RawProduct},
        ngram::{GramIndex, IndexFeed},
        Product, SuperAlloc,
    };
    use colosseum::sync::Arena;
    use std::collections::HashMap;

    let file = std::fs::read_to_string("./test.json")?;

    let products: HashMap<String, RawProduct> = serde_json::from_str(&file)?;

    let products: Vec<_> = products.into_iter().map(|(_, v)| v).collect();

    lazy_static::lazy_static! {
        static ref SUPER_ARENA: SuperAlloc = SuperAlloc::new();
    }

    let (prods, classic) = optimize(products, &SUPER_ARENA);

    let iter = prods.products.iter().map(|p| {
        let Product {
            description,
            title,
            vendor,
            ..
        } = p;

        IndexFeed {
            data: p,
            grams: [description, title, &vendor.name]
                .into_iter()
                .flat_map(|s| s.chars())
                .flat_map(|c| c.to_lowercase()),
        }
    });

    let mut arena = Arena::new();

    let index: GramIndex<char, Product, 8> = GramIndex::index_from(iter, &mut arena, prods);

    // We then serialize and deserialize
    let buff = serialize_all(&index, &classic);

    let node_arena2 = Arena::new();

    lazy_static::lazy_static! {
        static ref SUPER_ARENA2: SuperAlloc = SuperAlloc::new();
    }

    let (deserialized_ngram, deserialized_classic): (GramIndex<char, Product, 8>, ClassicIndexes) =
        deserialize_all(&buff[..], &node_arena2, &SUPER_ARENA2).unwrap();

    assert!(index == deserialized_ngram);
    assert!(classic == deserialized_classic);

    Ok(())
}
