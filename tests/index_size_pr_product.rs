use colosseum::sync::Arena;
use indexer_lib::{
    data::{optimize, Product, RawProduct, SuperAlloc},
    ngram::{GramIndex, IndexFeed},
    serialize::serialize_all,
};
use rand::prelude::SliceRandom;

lazy_static::lazy_static! {
    static ref JSON_TEST_DATA: String = std::fs::read_to_string("./test.json").unwrap();
    static ref TEST_PRODUCTS: Vec<RawProduct<'static>> = {
        let products: ahash::AHashMap<String, RawProduct<'_>> = serde_json::from_str(&JSON_TEST_DATA).unwrap();

        products.into_iter().map(|(_, v)| v).collect()
    };
    static ref SUPER_ALLOC: SuperAlloc = SuperAlloc::new();
}

#[test]
#[ignore = "Only for statistics around index sizes"]
fn index_size_pr_product() -> Result<(), Box<dyn std::error::Error>> {
    let sizes = [1, 5, 10, 20, 40, 80, 160, 320];

    let mut csv = "product_amount;size_in_kb\n".to_string();

    for size in sizes {
        let mut products = TEST_PRODUCTS.clone();
        let mut rng = rand::thread_rng();
        products.shuffle(&mut rng);

        products.truncate(size);

        let products_amount = products.len();

        let (prods, classic_index) = optimize(products, &SUPER_ALLOC);

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

        let index: GramIndex<char, Product, 5> = GramIndex::index_from(iter, &arena, prods);

        let output = serialize_all(&index, &classic_index);

        csv += &format!("{};{}\n", products_amount, (output.len() as f32) / 1000.0);
    }

    std::fs::write("index_size_pr_product.csv", csv)?;

    Ok(())
}
