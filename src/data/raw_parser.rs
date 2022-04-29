use serde::Deserialize;
use std::sync::Arc;

use ahash::AHashMap;

use crate::{
    classic_indexes::{CategoryIndex, ClassicIndexes, OrderIndex, TagIndex},
    data::vendor::VendorManager,
    Product,
};

use super::{FeatureSet, FeatureValue, ProductContainer, SuperAlloc};

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct CurrencyAmount {
    pub amount: String,
    #[serde(rename = "currencyCode")]
    pub currency_code: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ProductPrice {
    pub min: CurrencyAmount,
    pub max: CurrencyAmount,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct MediaItem<'a> {
    pub url: &'a str,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RawProduct<'a> {
    pub description: &'a str,
    pub tags: Vec<&'a str>,
    pub title: &'a str,
    pub vendor: &'a str,
    pub id: &'a str,
    pub options: Vec<RawProductOption<'a>>,
    pub price: ProductPrice,
    pub media: Vec<MediaItem<'a>>,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct RawProductOption<'a> {
    pub name: &'a str,
    pub values: Vec<&'a str>,
}

pub struct IntermediateRawProduct<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub tags: Vec<&'a str>,
    pub vendor: &'a str,
    pub id: &'a str,
    pub options: Vec<RawProductOption<'a>>,
    pub other_string: AHashMap<&'a str, &'a str>,
    pub other_numeric: AHashMap<&'a str, f32>,
}

fn to_intermediate(input: Vec<RawProduct<'_>>) -> impl Iterator<Item = IntermediateRawProduct<'_>> {
    input.into_iter().map(|raw| {
        let RawProduct {
            description,
            tags,
            title,
            vendor,
            id,
            options,
            mut media,
            price,
        } = raw;

        let mut other_string = AHashMap::with_capacity(1);

        if !media.is_empty() {
            other_string.insert("image_url", media.swap_remove(0).url);
        }

        let mut other_numeric = AHashMap::with_capacity(1);

        other_numeric.insert("price", price.min.amount.parse().unwrap());

        IntermediateRawProduct {
            title,
            description,
            tags,
            vendor,
            id,
            options,
            other_string,
            other_numeric,
        }
    })
}

pub fn optimize(
    input: Vec<RawProduct<'_>>,
    super_alloc: &'static SuperAlloc,
) -> (&'static ProductContainer<'static>, ClassicIndexes<'static>) {
    let mut vendors = VendorManager::new(super_alloc);

    // We insert all the tags/vendors
    for product in &input {
        vendors.insert(product.vendor);
    }

    let vendors: Arc<VendorManager<'static>> = Arc::new(vendors);
    let mut products: Vec<Product<'static>> = Vec::with_capacity(input.len());
    let out = ProductContainer::new(Vec::new(), vendors, FeatureSet::new_empty());
    let out = super_alloc.alloc_mut(out);
    let mut options_list = Vec::with_capacity(input.len());
    let mut tags_for_product = Vec::new();

    for (
        i,
        IntermediateRawProduct {
            title,
            description,
            tags,
            vendor,
            id,
            options,
            other_string,
            other_numeric,
        },
    ) in to_intermediate(input).enumerate()
    {
        let vendor = super_alloc.alloc(vendor.to_string());
        let my_vendor = out.vendors.get(vendor).unwrap();
        tags_for_product.push(tags);

        options_list.push(options);

        let p = Product {
            description: description.to_string(),
            vendor: my_vendor,
            title: title.to_string(),
            id: id.to_string(),
            serialization_id: i,
        };

        // We add the data from the "other"
        for (key, value) in other_string {
            let key = super_alloc.alloc(key.to_string());
            out.extra_features.add_string(key, value.to_string());
        }
        for (key, value) in other_numeric {
            let key = super_alloc.alloc(key.to_string());
            out.extra_features.add_float(key, value);
        }

        products.push(p);
    }

    out.products = products;
    // We make the products read only
    let out = &*out;

    let tag_index = TagIndex::index(tags_for_product, out);

    let mut order = OrderIndex::new();
    // Alphabetical
    order.add(out, &|product, _| &product.title, "Alphabetical".to_owned());
    // Price
    order.add(
        out,
        &|product, extra| match extra.get(product, "price") {
            Some(FeatureValue::Float(v)) => v,
            _ => panic!("Only expected price as float"),
        },
        "Price low to high".to_owned(),
    );
    order.add_custom_cmp(
        out,
        &|product, extra| match extra.get(product, "price") {
            Some(FeatureValue::Float(v)) => v,
            _ => panic!("Only expected price as float"),
        },
        &|a, b| {
            a.partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        },
        "Price high to low".to_owned(),
    );

    let categories = CategoryIndex::index(options_list, out);

    (out, ClassicIndexes::new(categories, tag_index, order))
}
