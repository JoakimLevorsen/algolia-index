use std::collections::HashMap;

use typed_arena::Arena;

use crate::ngramindex::{GramIndex, IndexFeed};

mod ngramindex;
mod preprocessor;
mod serde_array;
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

    let index: GramIndex<char, String, 8> = GramIndex::new(iter, &mut arena, &mut data_arena);

    // let json = match serde_json::to_string(&index) {
    //     Ok(v) => v,
    //     Err(e) => panic!("Got error {e}"),
    // };

    // println!("Popular: {:?}", index.most_popular_chain('o'));
    let query = ['k', 'u', 's', 't', 'p', 'l', 'a', 't'];
    // let query = ['k', 'u', 'm', 's', 't', 'p', 'l', 'l'];
    println!("Result of search {:?}", index.search(query));
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
