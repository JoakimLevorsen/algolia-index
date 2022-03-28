use std::sync::{Arc, RwLock};

use colosseum::sync::Arena;
use data::{Product, SuperAlloc};
use ngram::{GramIndex, GramNode};
use wasm_bindgen::prelude::*;

pub mod data;
pub mod ngram;
pub mod preprocessor;
mod serde_array;
mod serialize;

type Index = GramIndex<'static, char, Product<'static>, 7>;

lazy_static::lazy_static! {
    static ref SHARED_INDEX: RwLock<Option<Arc<Index>>> = RwLock::new(None);
    static ref NODE_ARENA: Arena<GramNode<'static, char>> = Arena::new();
    static ref SUPER_ARENA: SuperAlloc = SuperAlloc::new();
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn initialize(input: &[u8]) -> bool {
    init_panic_hook();

    let index: GramIndex<'_, char, Product, 7> =
        GramIndex::deserialize(input, &NODE_ARENA, &SUPER_ARENA).unwrap();

    SHARED_INDEX.write().unwrap().replace(Arc::new(index));
    true
}

#[wasm_bindgen]
pub fn search(input: String) -> Option<Vec<u64>> {
    let lock = SHARED_INDEX.try_read().ok()?;
    let index = lock.as_ref()?.clone();

    let results = index.search(input.chars().flat_map(|c| c.to_lowercase()));

    let results = results
        .into_iter()
        .map(|(v, _)| v)
        .map(|p| p.get_id())
        .collect();

    Some(results)
}

use crate::data::{optimize, RawProduct};
use crate::ngram::IndexFeed;
use crate::serialize::Serializable;

pub fn index_and_serialize(
    products: Vec<RawProduct>,
    arena: &'static SuperAlloc,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let prods = optimize(products, arena);

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

    let index: GramIndex<char, Product, 5> = GramIndex::index_from(iter, &mut arena, prods);

    index.search("UNERsTuOD hvzdom".chars().flat_map(|c| c.to_lowercase()));

    let mut output = Vec::new();
    index.serialize(&mut output);

    Ok(output)
}
