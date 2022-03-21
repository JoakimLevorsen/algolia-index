use std::collections::HashMap;

use typed_arena::Arena;

use crate::{ngramindex::IndexFeed, ngramindex2::GramIndex};

mod ngramindex;
mod ngramindex2;
mod ngramindexconstructor;
mod preprocessor;
mod serialize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string("./test.json")?;

    let products: HashMap<String, Product> = serde_json::from_str(&file)?;

    let iter = products.values().map(
        |Product {
             description,
             tags,
             title,
             vendor,
             id,
         }| IndexFeed {
            data: id.clone(),
            grams: [description, title, vendor]
                .into_iter()
                .chain(tags.into_iter())
                .flat_map(|s| s.chars())
                .flat_map(|c| c.to_lowercase()),
        },
    );

    let mut arena = Arena::new();
    let mut data_arena = Arena::new();

    let index: GramIndex<char, String, 8> =
        ngramindex2::GramIndex::new(iter, &mut arena, &mut data_arena);

    println!("Popular: {:?}", index.most_popular_chain('n'));
    // let index: NGramIndex<_, 8, _> = NGramIndex::new(iter);

    // for line in index.search("betydningsfuldt".chars()).into_iter() {
    //     println!("{line:?}")
    // }

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct Product {
    description: String,
    tags: Vec<String>,
    title: String,
    vendor: String,
    id: String,
}
