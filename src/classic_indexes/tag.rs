use ahash::{AHashMap, AHashSet};

use crate::{
    data::{Product, ProductContainer},
    serialize::{Deserializable, Serializable},
};

#[derive(PartialEq, Eq)]
pub struct Tag<'a> {
    pub name: String,
    products: Vec<&'a Product<'a>>,
    products_by_serialization_id: AHashSet<usize>,
    id: usize,
}

#[derive(PartialEq, Eq)]
pub struct TagIndex<'a>(Vec<Tag<'a>>);

impl<'a> Serializable for Tag<'a> {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        let Tag { name, products, .. } = self;
        name.serialize(output);

        Product::serialize_to_sequential_array(products, output);
    }
}

impl<'a> Tag<'a> {
    pub fn new(name: &str, products: Vec<&'a Product<'a>>, id: usize) -> Tag<'a> {
        let products_by_serialization_id = products.iter().map(|p| p.serialization_id).collect();
        Tag {
            name: name.to_string(),
            products,
            products_by_serialization_id,
            id,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn contains(&self, product: &Product<'_>) -> bool {
        self.products_by_serialization_id
            .contains(&product.serialization_id)
    }
}

impl<'a> Serializable for TagIndex<'a> {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        let tags = &self.0;
        tags.serialize(output);
    }
}

impl<'a> TagIndex<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        existing_products: &'a [Product<'a>],
    ) -> Option<(&'i [u8], TagIndex<'a>)> {
        let (input, tag_len) = usize::deserialize(input)?;
        let mut input = input;
        let mut tags = Vec::with_capacity(tag_len);
        for id in 0..tag_len {
            let (new_input, name) = String::deserialize(input)?;

            let (new_input, products, product_ids) =
                Product::deserialize_from_sequential_ids(new_input, existing_products)?;

            input = new_input;

            tags.push(Tag {
                name,
                products,
                id,
                products_by_serialization_id: product_ids.into_iter().collect(),
            });
        }

        Some((input, TagIndex(tags)))
    }

    pub fn get(&'a self, id: usize) -> Option<&'a Tag<'a>> {
        self.0.iter();
        self.0.get(id)
    }

    pub fn index<'out>(
        tags_for_product: Vec<Vec<&str>>,
        container: &'out ProductContainer<'out>,
    ) -> TagIndex<'out> {
        let mut products_for_tag: AHashMap<&str, Vec<&Product>> = AHashMap::new();
        for (id, tags) in tags_for_product.into_iter().enumerate() {
            let product = &container.products[id];
            for tag in tags {
                products_for_tag
                    .entry(tag)
                    .and_modify(|v| v.push(product))
                    .or_insert_with(|| vec![product]);
            }
        }

        TagIndex(
            products_for_tag
                .into_iter()
                .enumerate()
                .map(|(id, (name, products))| crate::classic_indexes::Tag::new(name, products, id))
                .collect(),
        )
    }

    pub fn iter(&'a self) -> impl Iterator<Item = &'a Tag<'a>> {
        self.0.iter()
    }
}
