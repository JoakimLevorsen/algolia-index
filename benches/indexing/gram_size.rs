#[macro_use]
extern crate bencher;

use bencher::Bencher;

use colosseum::sync::Arena;
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

fn test<const N: usize>(bench: &mut Bencher) {
    let (prods, _) = indexer_lib::data::optimize(TEST_PRODUCTS.clone(), &SUPER_ALLOC);

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

        let index: GramIndex<'_, char, Product, N> = GramIndex::index_from(iter, &arena, prods);

        criterion::black_box(index);
    });
}

fn b2_gram_indexing(bench: &mut Bencher) {
    test::<2>(bench);
}

fn b3_gram_indexing(bench: &mut Bencher) {
    test::<3>(bench);
}

fn b4_gram_indexing(bench: &mut Bencher) {
    test::<4>(bench);
}

fn b5_gram_indexing(bench: &mut Bencher) {
    test::<5>(bench);
}

fn b6_gram_indexing(bench: &mut Bencher) {
    test::<6>(bench);
}

fn b7_gram_indexing(bench: &mut Bencher) {
    test::<7>(bench);
}

fn b8_gram_indexing(bench: &mut Bencher) {
    test::<8>(bench);
}

fn b9_gram_indexing(bench: &mut Bencher) {
    test::<9>(bench);
}

benchmark_group!(
    indexing_based_on_gram_size,
    b2_gram_indexing,
    b3_gram_indexing,
    b4_gram_indexing,
    b5_gram_indexing,
    b6_gram_indexing,
    b7_gram_indexing,
    b8_gram_indexing,
    b9_gram_indexing
);
benchmark_main!(indexing_based_on_gram_size);
