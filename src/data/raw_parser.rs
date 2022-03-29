use std::sync::Arc;

use crate::{
    classic_indexes::{Category, CategoryOption, ClassicIndexes},
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
    pub options: Vec<RawProductOption<'a>>,
}

#[derive(serde::Deserialize, PartialEq, Eq, Debug)]
pub struct RawProductOption<'a> {
    pub name: &'a str,
    pub values: Vec<&'a str>,
}

pub fn optimize(
    input: Vec<RawProduct>,
    super_alloc: &'static SuperAlloc,
) -> (&'static ProductContainer<'static>, ClassicIndexes<'static>) {
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

    let mut options_list = Vec::with_capacity(input.len());
    for (
        i,
        RawProduct {
            description,
            tags,
            title,
            vendor,
            id,
            options,
        },
    ) in input.into_iter().enumerate()
    {
        let my_vendor = out.vendors.get(vendor).unwrap();
        let mut my_tags = Vec::with_capacity(tags.len());
        for tag in tags {
            my_tags.push(out.tags.get(tag).unwrap())
        }

        options_list.push(options);

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

    let mut categories: Vec<Category> = Vec::new();
    let mut next_serialization_id = 0;
    // Then we register the categories for the options now they're read only
    for (i, options) in options_list.into_iter().enumerate() {
        for raw_option in options {
            let category = match categories.iter_mut().find(|c| c.name == raw_option.name) {
                Some(v) => v,
                None => {
                    let new = Category::new(raw_option.name.to_string());
                    categories.push(new);
                    let just_inserted = categories.len() - 1;
                    categories.get_mut(just_inserted).unwrap()
                }
            };

            for raw_value in raw_option.values {
                let option = match category.options.iter_mut().find(|o| o.name == raw_value) {
                    Some(v) => v,
                    None => {
                        let new = CategoryOption::new(raw_value.to_string(), next_serialization_id);
                        next_serialization_id += 1;
                        category.options.push(new);
                        let just_inserted = category.options.len() - 1;
                        category.options.get_mut(just_inserted).unwrap()
                    }
                };

                option.content.push(&out.products[i])
            }
        }
    }

    (out, ClassicIndexes::new(categories))
}
