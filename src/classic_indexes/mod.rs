mod categorical;
mod order;
mod tag;

pub use categorical::{Category, CategoryIndex, CategoryOption};

use crate::{
    data::ProductContainer,
    serialize::{Deserializable, Serializable},
};

pub use order::OrderIndex;
pub use tag::{Tag, TagIndex};

#[derive(PartialEq, Eq)]
pub struct ClassicIndexes<'a> {
    pub categories: CategoryIndex<'a>,
    pub tags: TagIndex<'a>,
    pub order: OrderIndex,
}

impl<'a> ClassicIndexes<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        data: &'a ProductContainer<'a>,
    ) -> Option<(&'i [u8], Self)> {
        let (input, categories) = CategoryIndex::deserialize_many(input, &data.products)?;
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
        categories: CategoryIndex<'a>,
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
        self.categories.serialize(output);
        self.tags.serialize(output);
        self.order.serialize(output);
    }
}
