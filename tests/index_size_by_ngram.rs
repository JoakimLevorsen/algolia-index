use colosseum::sync::Arena;
use indexer_lib::{
    data::{optimize, Product, RawProduct, SuperAlloc},
    ngram::{GramIndex, IndexFeed},
    serialize::serialize_all,
};

lazy_static::lazy_static! {
    static ref JSON_TEST_DATA: String = std::fs::read_to_string("./test.json").unwrap();
    static ref TEST_PRODUCTS: Vec<RawProduct<'static>> = {
        let products: ahash::AHashMap<String, RawProduct<'_>> = serde_json::from_str(&JSON_TEST_DATA).unwrap();

        products.into_iter().map(|(_, v)| v).collect()
    };
    static ref SUPER_ALLOC: SuperAlloc = SuperAlloc::new();
}

fn do_test<const N: usize>(csv: &mut String) {
    let (prods, classic_index) = optimize(TEST_PRODUCTS.clone(), &SUPER_ALLOC);

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

    let index: GramIndex<char, Product, N> = GramIndex::index_from(iter, &arena, prods);

    let output = serialize_all(&index, &classic_index);

    csv.push_str(&format!("{N};{}\n", (output.len() as f32) / 1000.0));
}

#[test]
#[ignore = "Only for statistics around index sizes"]
fn index_size_by_ngram() -> Result<(), Box<dyn std::error::Error>> {
    let mut csv = "n-gram;size_in_kb\n".to_string();

    do_test::<2>(&mut csv);
    do_test::<3>(&mut csv);
    do_test::<4>(&mut csv);
    do_test::<5>(&mut csv);
    do_test::<6>(&mut csv);
    do_test::<7>(&mut csv);
    do_test::<8>(&mut csv);
    do_test::<9>(&mut csv);

    std::fs::write("index_size_by_ngram.csv", csv)?;

    Ok(())
}
