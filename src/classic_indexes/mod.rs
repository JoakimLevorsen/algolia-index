mod categorical;
mod order;
mod tag;

pub use categorical::{Category, CategoryOption};

use crate::{
    data::ProductContainer,
    serialize::{Deserializable, Serializable},
};

pub use order::OrderIndex;
pub use tag::{Tag, TagIndex};

#[derive(PartialEq, Eq)]
pub struct ClassicIndexes<'a> {
    pub categories: Vec<Category<'a>>,
    pub tags: TagIndex<'a>,
    pub order: OrderIndex,
}

impl<'a> ClassicIndexes<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        data: &'a ProductContainer<'a>,
    ) -> Option<(&'i [u8], Self)> {
        let (input, categories) = Category::deserialize_many(input, &data.products)?;
        let (input, tags) = TagIndex::deserialize(input, &data.products)?;
        let (input, order) = OrderIndex::deserialize(input)?;
        Some((
            input,
            ClassicIndexes {
                categories,
                tags,
                order,
            },
        ))
    }

    pub fn new(
        categories: Vec<Category<'a>>,
        tags: TagIndex<'a>,
        order: OrderIndex,
    ) -> ClassicIndexes<'a> {
        ClassicIndexes {
            categories,
            tags,
            order,
        }
    }
}

impl<'a> Serializable for ClassicIndexes<'a> {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        Category::serialize_many(&self.categories, output);
        self.tags.serialize(output);
    }
}
