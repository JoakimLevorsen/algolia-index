use std::collections::HashMap;

use colosseum::sync::Arena;

mod ngram;
mod preprocessor;
mod product;
mod serde_array;
mod serialize;

use crate::ngram::{GramIndex, IndexFeed};

use serialize::Serializable;

pub use product::Product;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            data: p,
            grams: [description, title, vendor]
                .into_iter()
                .chain(tags.into_iter())
                .flat_map(|s| s.chars())
                .flat_map(|c| c.to_lowercase()),
        }
    });

    let mut arena = Arena::new();

    let index: GramIndex<char, &Product, 7> = GramIndex::index_from(iter, &mut arena);

    // let json = match serde_json::to_string(&index) {
    //     Ok(v) => v,
    //     Err(e) => panic!("Got error {e}"),
    // };

    // println!("Popular: {:?}", index.most_popular_chain('o'));

    // let query = [' ', 'e', 'r', ' '];
    index.search("UNERsTuOD hvzdom".chars().flat_map(|c| c.to_lowercase()));

    let mut output = Vec::new();
    index.serialize(&mut output);

    std::fs::write("out.test", output)?;

    // let query = ['k', 'u', 'm', 's', 't', 'p', 'l', 'l'];

    // let index: NGramIndex<_, 8, _> = NGramIndex::new(iter);

    // for line in index.search("betydningsfuldt".chars()).into_iter() {
    //     println!("{line:?}")
    // }

    Ok(())
}
