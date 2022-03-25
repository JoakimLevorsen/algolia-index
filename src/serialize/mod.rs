mod collections;
mod nodes;
mod primitives;
mod traits;

pub use traits::*;

#[cfg(test)]
use wasm_bindgen_test::wasm_bindgen_test;

#[test]
#[wasm_bindgen_test]
fn test_deserialize_wasm() {
    use crate::{data::SuperAlloc, GramIndex, Product};
    use colosseum::sync::Arena;
    use std::io::Read;

    let mut file = std::fs::File::open("./out.test").unwrap();

    let mut buff = Vec::new();

    let index_data = file.read_to_end(&mut buff).unwrap();

    let node_arena = Arena::new();

    lazy_static::lazy_static! {
        static ref SUPER_ARENA: SuperAlloc = SuperAlloc::new();
    }

    let index: GramIndex<char, Product, 7> =
        GramIndex::deserialize(&buff, &node_arena, &SUPER_ARENA).unwrap();
}

#[test]
fn test_serialize_and_deserialize() -> Result<(), Box<dyn std::error::Error>> {
    use crate::{
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

    let prods = optimize(products, &SUPER_ARENA);

    let iter = prods.products.iter().map(|p| {
        let Product {
            description,
            tags,
            title,
            vendor,
            ..
        } = p;

        IndexFeed {
            data: p,
            grams: [description, title, &vendor.name]
                .into_iter()
                .chain(tags.iter().map(|t| &t.name))
                .flat_map(|s| s.chars())
                .flat_map(|c| c.to_lowercase()),
        }
    });

    let mut arena = Arena::new();

    let index: GramIndex<char, Product, 8> = GramIndex::index_from(iter, &mut arena, prods);

    // We then serialize and deserialize
    let mut buff = Vec::new();
    index.serialize(&mut buff);

    let node_arena2 = Arena::new();

    lazy_static::lazy_static! {
        static ref SUPER_ARENA2: SuperAlloc = SuperAlloc::new();
    }

    let deserialized: GramIndex<char, Product, 8> =
        GramIndex::deserialize(&buff[..], &node_arena2, &SUPER_ARENA2).unwrap();

    Ok(())
}
