mod categorical;

pub use categorical::{Category, CategoryOption};

use crate::{data::ProductContainer, serialize::Serializable};

#[derive(PartialEq, Eq)]
pub struct ClassicIndexes<'a> {
    categories: Vec<Category<'a>>,
}

impl<'a> ClassicIndexes<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        data: &'a ProductContainer<'a>,
    ) -> Option<(&'i [u8], Self)> {
        let (input, categories) = Category::deserialize_many(input, &data.products)?;
        Some((input, ClassicIndexes { categories }))
    }

    pub fn new(categories: Vec<Category<'a>>) -> ClassicIndexes<'a> {
        ClassicIndexes { categories }
    }
}

impl<'a> Serializable for ClassicIndexes<'a> {
    fn serialize(&self, output: &mut Vec<u8>) {
        Category::serialize_many(&self.categories, output);
    }
}
