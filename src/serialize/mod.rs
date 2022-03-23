mod collections;
mod nodes;
mod primitives;
mod traits;

pub use traits::*;

use std::io::Read;
#[cfg(test)]
use wasm_bindgen_test::wasm_bindgen_test;

use crate::ngram::GramIndex;

#[test]
#[wasm_bindgen_test]
fn test_deserialize_wasm() {
    use crate::Product;
    use colosseum::sync::Arena;

    let mut file = std::fs::File::open("./out.test").unwrap();

    let mut buff = Vec::new();

    let index_data = file.read_to_end(&mut buff).unwrap();

    let node_arena = colosseum::sync::Arena::new();
    let data_arena = colosseum::sync::Arena::new();

    let index: GramIndex<char, Product, 7> =
        GramIndex::deserialize(&buff, &node_arena, &data_arena).unwrap();
}

#[test]
fn test_serialize_and_deserialize() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{ngram::IndexFeed, Product};
    use colosseum::sync::Arena;
    use std::collections::HashMap;

    let file = std::fs::read_to_string("./test.json")?;

    let products: HashMap<String, Product> = serde_json::from_str(&file)?;

    let data_arena = Arena::new();
    let products: Vec<&Product> = products
        .into_iter()
        .map(|(_, v)| v)
        .map(|p| &*data_arena.alloc(p))
        .collect();

    let iter = products.iter().map(|p| {
        let Product {
            description,
            tags,
            title,
            vendor,
            ..
        } = p;

        IndexFeed {
            data: *p,
            grams: [description, title, vendor]
                .into_iter()
                .chain(tags.into_iter())
                .flat_map(|s| s.chars())
                .flat_map(|c| c.to_lowercase()),
        }
    });

    let mut arena = Arena::new();

    let index: GramIndex<char, Product, 8> = GramIndex::index_from(iter, &mut arena);

    // We then serialize and deserialize
    let mut buff = Vec::new();
    index.serialize(&mut buff);

    let node_arena2 = Arena::new();
    let data_arena2 = Arena::new();

    let deserialized: GramIndex<char, Product, 8> =
        GramIndex::deserialize(&buff[..], &node_arena2, &data_arena2).unwrap();

    Ok(())
}
