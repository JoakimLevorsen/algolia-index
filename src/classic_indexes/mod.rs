mod categorical;
mod tag;

pub use categorical::{Category, CategoryOption};

use crate::{data::ProductContainer, serialize::Serializable};

pub use self::tag::{Tag, TagIndex};

#[derive(PartialEq, Eq)]
pub struct ClassicIndexes<'a> {
    pub categories: Vec<Category<'a>>,
    pub tags: TagIndex<'a>,
}

impl<'a> ClassicIndexes<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        data: &'a ProductContainer<'a>,
    ) -> Option<(&'i [u8], Self)> {
        let (input, categories) = Category::deserialize_many(input, &data.products)?;
        let (input, tags) = TagIndex::deserialize(input, &data.products)?;
        Some((input, ClassicIndexes { categories, tags }))
    }

    pub fn new(categories: Vec<Category<'a>>, tags: TagIndex<'a>) -> ClassicIndexes<'a> {
        ClassicIndexes { categories, tags }
    }
}

impl<'a> Serializable for ClassicIndexes<'a> {
    fn serialize(&self, output: &mut Vec<u8>) {
        Category::serialize_many(&self.categories, output);
        self.tags.serialize(output);
    }
}
