#[macro_use]
extern crate bencher;

use bencher::Bencher;
use colosseum::sync::Arena;
use rand::prelude::*;

use indexer_lib::{
    data::{Product, RawProduct, SuperAlloc},
    ngram::{GramIndex, IndexFeed},
};

lazy_static::lazy_static! {
    static ref JSON_TEST_DATA: String = std::fs::read_to_string("./test.json").unwrap();
    static ref TEST_PRODUCTS: Vec<RawProduct<'static>> = {
        let products: ahash::AHashMap<String, RawProduct<'_>> = serde_json::from_str(&JSON_TEST_DATA).unwrap();

        products.into_iter().map(|(_, v)| v).collect()
    };
    static ref SUPER_ALLOC: SuperAlloc = SuperAlloc::new();
}

pub fn test(product_amount: usize, bench: &mut Bencher) {
    let mut products = TEST_PRODUCTS.clone();
    let mut rng = rand::thread_rng();
    products.shuffle(&mut rng);

    products.truncate(product_amount);

    let (prods, _) = indexer_lib::data::optimize(products, &SUPER_ALLOC);

    let arena = Arena::new();

    bench.iter(|| {
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

        let _: GramIndex<'_, char, Product, 3> = GramIndex::index_from(iter, &arena, prods);
    })
}

fn ten_products(bench: &mut Bencher) {
    test(10, bench)
}
fn twenty_products(bench: &mut Bencher) {
    test(20, bench)
}
fn fourty_products(bench: &mut Bencher) {
    test(40, bench)
}
fn eighty_products(bench: &mut Bencher) {
    test(80, bench)
}
fn onesixty_products(bench: &mut Bencher) {
    test(180, bench)
}
fn all_products(bench: &mut Bencher) {
    test(TEST_PRODUCTS.len(), bench)
}

benchmark_group!(
    indexing_based_on_product_amount,
    ten_products,
    twenty_products,
    fourty_products,
    eighty_products,
    onesixty_products,
    all_products
);
benchmark_main!(indexing_based_on_product_amount);
