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

    bench.iter(|| {
        let r = INDEX.search(query.chars().flat_map(char::to_lowercase));
        criterion::black_box(r)
    })
}

fn b3_chars_search(bench: &mut Bencher) {
    search(bench, 3)
}
fn b5_chars_search(bench: &mut Bencher) {
    search(bench, 5)
}
fn b7_chars_search(bench: &mut Bencher) {
    search(bench, 7)
}
fn c10_chars_search(bench: &mut Bencher) {
    search(bench, 10)
}
fn c15_chars_search(bench: &mut Bencher) {
    search(bench, 15)
}
fn c20_chars_search(bench: &mut Bencher) {
    search(bench, 20)
}
fn c25_chars_search(bench: &mut Bencher) {
    search(bench, 25)
}
fn c34_chars_search(bench: &mut Bencher) {
    search(bench, 34)
}

benchmark_group!(
    searching_by_query_len,
    b3_chars_search,
    b5_chars_search,
    b7_chars_search,
    c10_chars_search,
    c15_chars_search,
    c20_chars_search,
    c25_chars_search,
    c34_chars_search
);
benchmark_main!(searching_by_query_len);
