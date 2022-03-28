use std::collections::HashMap;

use indexer_lib::{
    data::{RawProduct, SuperAlloc},
    index_and_serialize,
};

lazy_static::lazy_static! {
    static ref SUPER_ARENA: SuperAlloc = SuperAlloc::new();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string("./test.json")?;

    let products: HashMap<String, RawProduct> = serde_json::from_str(&file)?;

    let products: Vec<_> = products.into_iter().map(|(_, v)| v).collect();

    let output = index_and_serialize(products, &SUPER_ARENA)?;

    std::fs::write("out.test", output)?;

    Ok(())
}
