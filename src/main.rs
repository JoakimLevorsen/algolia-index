use std::collections::HashMap;

use colosseum::sync::Arena;
use data::{optimize, RawProduct, SuperAlloc};

mod data;
mod ngram;
mod preprocessor;
mod serde_array;
mod serialize;

use crate::ngram::{GramIndex, IndexFeed};

use serialize::Serializable;

pub use data::Product;

lazy_static::lazy_static! {
    static ref SUPER_ARENA: SuperAlloc = SuperAlloc::new();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string("./test.json")?;

    let products: HashMap<String, RawProduct> = serde_json::from_str(&file)?;

    let products: Vec<_> = products.into_iter().map(|(_, v)| v).collect();

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

    let index: GramIndex<char, Product, 7> = GramIndex::index_from(iter, &mut arena, prods);

    index.search("UNERsTuOD hvzdom".chars().flat_map(|c| c.to_lowercase()));

    let mut output = Vec::new();
    index.serialize(&mut output);

    std::fs::write("out.test", output)?;

    Ok(())
}
