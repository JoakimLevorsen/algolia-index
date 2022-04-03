use crate::{
    data::Product,
    serialize::{Deserializable, Serializable},
};

#[derive(PartialEq, Eq)]
pub struct Tag<'a> {
    pub name: String,
    pub products: Vec<&'a Product<'a>>,
    id: usize,
}

#[derive(PartialEq, Eq)]
pub struct TagIndex<'a> {
    pub tags: Vec<Tag<'a>>,
}

impl<'a> Serializable for Tag<'a> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Tag { name, products, .. } = self;
        name.serialize(output);
        products.len().serialize(output);
        for p in products {
            p.serialization_id.serialize(output);
        }
    }
}

impl<'a> Tag<'a> {
    pub fn new(name: &str, products: Vec<&'a Product<'a>>, id: usize) -> Tag<'a> {
        Tag {
            name: name.to_string(),
            products,
            id,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
}

impl<'a> Serializable for TagIndex<'a> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let TagIndex { tags } = self;
        tags.serialize(output)
    }
}

impl<'a> TagIndex<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        existing_products: &'a Vec<Product<'a>>,
    ) -> Option<(&'i [u8], TagIndex<'a>)> {
        let (input, tag_len) = usize::deserialize(input)?;
        let mut input = input;
        let mut tags = Vec::with_capacity(tag_len);
        for id in 0..tag_len {
            let (new_input, name) = String::deserialize(input)?;
            let (new_input, products_amount) = usize::deserialize(new_input)?;

            input = new_input;

            let mut products = Vec::with_capacity(products_amount);
            for _ in 0..products_amount {
                let (new_input, id) = usize::deserialize(input)?;
                input = new_input;
                let product = existing_products.get(id)?;
                products.push(product);
            }

            tags.push(Tag { name, products, id })
        }

        Some((input, TagIndex { tags }))
    }

    pub fn get(&'a self, id: usize) -> Option<&'a Tag<'a>> {
        self.tags.get(id)
    }
}
