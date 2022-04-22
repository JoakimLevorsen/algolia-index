#[macro_use]
extern crate bencher;

use bencher::Bencher;
use colosseum::sync::Arena;
use indexer_lib::{
    data::{Product, RawProduct, SuperAlloc},
    ngram::{GramIndex, IndexFeed},
};

const N: usize = 3;

lazy_static::lazy_static! {
    static ref JSON_TEST_DATA: String = std::fs::read_to_string("./test.json").unwrap();
    static ref TEST_PRODUCTS: Vec<RawProduct<'static>> = {
        let products: ahash::AHashMap<String, RawProduct<'_>> = serde_json::from_str(&JSON_TEST_DATA).unwrap();

        products.into_iter().map(|(_, v)| v).collect()
    };
    static ref SUPER_ALLOC: SuperAlloc = SuperAlloc::new();
    static ref INDEX: GramIndex<'static, char, Product<'static>, N> = make_index();
}

fn make_index() -> GramIndex<'static, char, Product<'static>, N> {
    let (prods, _) = indexer_lib::data::optimize(TEST_PRODUCTS.clone(), &SUPER_ALLOC);

    let arena = SUPER_ALLOC.alloc(Arena::new());

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

    GramIndex::index_from(iter, arena, prods)
}

const QUERY: &str = "Kunsplakter orang blå Nya hedegård";

fn search(bench: &mut Bencher, len: usize) {
    let query = &QUERY[0..len];

    bench.iter(|| INDEX.search(query.chars().flat_map(char::to_lowercase)))
}

fn three_chars(bench: &mut Bencher) {
    search(bench, 3)
}
fn five_chars(bench: &mut Bencher) {
    search(bench, 5)
}
fn seven_chars(bench: &mut Bencher) {
    search(bench, 7)
}
fn ten_chars(bench: &mut Bencher) {
    search(bench, 10)
}
fn fifteen_chars(bench: &mut Bencher) {
    search(bench, 15)
}
fn twenty_chars(bench: &mut Bencher) {
    search(bench, 20)
}
fn twenty_five_chars(bench: &mut Bencher) {
    search(bench, 25)
}
fn thirty_four_chars(bench: &mut Bencher) {
    search(bench, 34)
}

benchmark_group!(
    searching_by_query_len,
    three_chars,
    five_chars,
    seven_chars,
    ten_chars,
    fifteen_chars,
    twenty_chars,
    twenty_five_chars,
    thirty_four_chars
);
benchmark_main!(searching_by_query_len);
