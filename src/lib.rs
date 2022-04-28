#![allow(clippy::bool_comparison)]
#![allow(clippy::unused_unit)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use std::sync::{Arc, Mutex};

use classic_indexes::ClassicIndexes;
use colosseum::sync::Arena;
use data::{Product, SuperAlloc};
use ngram::{GramIndex, GramNode};
use serialize::{deserialize_all, serialize_all};
use wasm_bindgen::prelude::*;

pub mod classic_indexes;
pub mod data;
pub mod js_interactable;
pub mod ngram;
pub mod preprocessor;
mod serde_array;
pub mod serialize;

const NGRAM_INDEX_SIZE: usize = 5;
type Index = GramIndex<'static, char, Product<'static>, NGRAM_INDEX_SIZE>;

lazy_static::lazy_static! {
    static ref SHARED_INDEX: Mutex<Option<Arc<Index>>> = Mutex::new(None);
    static ref SHARED_CLASSIC_INDEX: Mutex<Option<Arc<ClassicIndexes<'static>>>> = Mutex::new(None);
    static ref NODE_ARENA: Arena<GramNode<'static, char>> = Arena::new();
    static ref SUPER_ARENA: SuperAlloc = SuperAlloc::new();
}

// #[wasm_bindgen]
// struct SearchEngine<'engine> {
//     ngram: Arc<GramIndex<'engine, char, Product<'engine>, NGRAM_INDEX_SIZE>>,
//     classic: Arc<ClassicIndexes<'engine>>,
//     alloc: SuperAlloc,
// }

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn initialize(input: &[u8]) -> bool {
    init_panic_hook();

    let (index, classic) = deserialize_all(input, &NODE_ARENA, &SUPER_ARENA).unwrap();

    SHARED_INDEX.lock().unwrap().replace(Arc::new(index));
    SHARED_CLASSIC_INDEX
        .lock()
        .unwrap()
        .replace(Arc::new(classic));

    true
}

#[wasm_bindgen]
pub fn search(
    input: &str,
    categories: &CategoryHandler,
    tags: &TagHandler,
    order: Option<String>,
    feature_filter: &js_sys::Object,
) -> Option<ProductProducer> {
    let filters = FeatureFilter::parse(feature_filter)?;

    let lock = SHARED_INDEX.lock().ok()?;
    let index = lock.as_ref()?.clone();

    let results = index.search(input.chars().flat_map(char::to_lowercase));

    let mut results: Vec<_> = results
        .into_iter()
        .map(|(v, _)| v)
        // We remove the products of the wrong category or tag
        .filter(|p| categories.is_valid(p))
        .filter(|p| tags.is_valid(p))
        .map(|p| p.serialization_id)
        .collect();

    if let Some(order) = order {
        let lock = SHARED_CLASSIC_INDEX.lock().ok()?;
        let classic = lock.as_ref()?;
        let order = classic.order.get_orders(&order)?;
        results.sort_by_cached_key(|v| order[*v]);
    }

    Some(ProductProducer::new(index.product_container, results))
}

use js_interactable::{CategoryHandler, FeatureFilter, TagHandler};

#[wasm_bindgen]
pub fn get_categories() -> Option<CategoryHandler> {
    let read_lock = SHARED_CLASSIC_INDEX.lock().unwrap();

    let index = read_lock.as_ref()?;

    Some(CategoryHandler::new(index.clone()))
}

#[wasm_bindgen]
pub fn get_tags() -> Option<TagHandler> {
    let read_lock = SHARED_CLASSIC_INDEX.lock().unwrap();

    let index = read_lock.as_ref()?;

    Some(TagHandler::new(index.clone()))
}

#[wasm_bindgen]
pub fn get_orders() -> Option<Vec<JsValue>> {
    let lock = SHARED_CLASSIC_INDEX.lock().ok()?;
    let index = lock.as_ref()?;
    let options = index.order.options();
    let js_options = options.map(JsValue::from);
    Some(js_options.collect())
}

#[wasm_bindgen]
pub struct TagSuggestionResult {
    tag: js_interactable::JSTag,
    input: String,
}

#[wasm_bindgen]
impl TagSuggestionResult {
    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    pub fn get_tag(&self) -> js_interactable::JSTag {
        self.tag.clone()
    }
}

// Simple string match to find likely tags
#[wasm_bindgen]
pub fn tag_suggestion(query: &str) -> Option<TagSuggestionResult> {
    fn overlap(original: &str, other: &str, min_len: usize) -> f32 {
        let min_len_found = original.len().min(other.len());
        if min_len_found < min_len {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let min_len_found = min_len_found as f32;
        let overlap = original
            .chars()
            .zip(other.chars())
            .map(|(a, b)| if a == b { 1.0 } else { 0.0 })
            .reduce(|a, b| a + b)
            .unwrap_or(0.0);
        overlap / min_len_found
    }

    let read_lock = SHARED_CLASSIC_INDEX.lock().unwrap();

    let index = read_lock.as_ref()?;

    let mut most_likely: Option<(f32, usize, &str)> = None;
    for keyword in query.split(' ') {
        for tag in &index.tags.tags {
            let overlap = overlap(tag.name.as_str(), keyword, 3);
            if overlap > 0.8 {
                most_likely = match most_likely {
                    Some((current, _, _)) if overlap > current => {
                        Some((overlap, tag.get_id(), keyword))
                    }
                    None => Some((overlap, tag.get_id(), keyword)),
                    current => current,
                }
            }
        }
    }

    most_likely.map(|(_, tag_id, input)| TagSuggestionResult {
        tag: js_interactable::JSTag::new(index.clone(), tag_id),
        input: input.to_string(),
    })
}

use crate::data::{optimize, RawProduct};
use crate::js_interactable::ProductProducer;
use crate::ngram::IndexFeed;

pub fn index_and_serialize(
    products: Vec<RawProduct>,
    arena: &'static SuperAlloc,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    index_and_serialize_with_n::<NGRAM_INDEX_SIZE>(products, arena)
}

pub fn index_and_serialize_with_n<const N: usize>(
    products: Vec<RawProduct>,
    arena: &'static SuperAlloc,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (prods, classic_index) = optimize(products, arena);

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
                .flat_map(char::to_lowercase),
        }
    });

    let arena = Arena::new();

    let index: GramIndex<char, Product, N> = GramIndex::index_from(iter, &arena, prods);

    let results = index.search("kunst".chars().flat_map(char::to_lowercase));
    // let results = index.search("UNERsTuOD hvzdom".chars().flat_map(char::to_lowercase));

    println!("Got {} results", results.len());

    let output = serialize_all(&index, &classic_index);

    Ok(output)
}

#[cfg(feature = "indexing")]
#[wasm_bindgen]
pub fn index(input: &str) -> Option<Vec<u8>> {
    let products: ahash::AHashMap<String, RawProduct> = serde_json::from_str(input).ok()?;

    let products: Vec<_> = products.into_iter().map(|(_, v)| v).collect();

    let output = index_and_serialize(products, &SUPER_ARENA).ok()?;

    Some(output)
}
