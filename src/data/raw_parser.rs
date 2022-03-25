use std::sync::Arc;

use crate::{
    data::{tags::TagManager, vendor::VendorManager},
    Product,
};

use super::{ProductContainer, SuperAlloc};

#[derive(serde::Deserialize, PartialEq, Eq, Debug)]
pub struct RawProduct<'a> {
    pub description: String,
    pub tags: Vec<&'a str>,
    pub title: String,
    pub vendor: &'a str,
    pub id: String,
}

pub fn optimize(
    input: Vec<RawProduct>,
    super_alloc: &'static SuperAlloc,
) -> &'static ProductContainer<'static> {
    let mut tag_manager = TagManager::new(super_alloc);
    let mut vendors = VendorManager::new(super_alloc);

    // We insert all the tags/vendors
    for product in input.iter() {
        for tag in &product.tags {
            tag_manager.insert(tag);
        }
        vendors.insert(product.vendor);
    }

    let tag_manager = Arc::new(tag_manager);
    let vendors = Arc::new(vendors);

    let mut products: Vec<Product<'static>> = Vec::with_capacity(input.len());

    let out = ProductContainer::new(Vec::new(), tag_manager, vendors);

    let out = super_alloc.alloc_mut(out);

    for (
        i,
        RawProduct {
            description,
            tags,
            title,
            vendor,
            id,
        },
    ) in input.into_iter().enumerate()
    {
        let my_vendor = out.vendors.get(vendor).unwrap();
        let mut my_tags = Vec::with_capacity(tags.len());
        for tag in tags {
            my_tags.push(out.tags.get(tag).unwrap())
        }

        let p = Product {
            description: description,
            tags: my_tags,
            vendor: my_vendor,
            title: title,
            id,
            serialization_id: i,
        };
        products.push(p);
    }

    out.products = products;
    out
}
