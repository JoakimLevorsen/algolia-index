use std::sync::{Arc, RwLock};

use colosseum::sync::Arena;
use ngram::{GramIndex, GramNode};
use product::Product;
use wasm_bindgen::prelude::*;

mod ngram;
mod preprocessor;
mod product;
mod serde_array;
mod serialize;

type Index = GramIndex<'static, char, Product, 8>;

lazy_static::lazy_static! {
    static ref SHARED_INDEX: RwLock<Option<Arc<Index>>> = RwLock::new(None);
    static ref NODE_ARENA: Arena<GramNode<'static, char>> = Arena::new();
    static ref DATA_ARENA: Arena<Product> = Arena::new();
}

#[wasm_bindgen]
pub fn initialize(input: &[u8]) -> bool {
    let index: GramIndex<'_, char, Product, 8> =
        GramIndex::deserialize(input, &NODE_ARENA, &DATA_ARENA).unwrap();

    SHARED_INDEX.write().unwrap().replace(Arc::new(index));
    true
}

#[wasm_bindgen]
pub fn search(input: String) -> Option<bool> {
    let lock = SHARED_INDEX.try_read().ok()?;
    let index = lock.as_ref()?.clone();

    let results = index.search(input.chars().flat_map(|c| c.to_lowercase()));

    None
}
