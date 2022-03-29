use crate::{
    data::Product,
    serialize::{Deserializable, Serializable},
};

#[derive(PartialEq, Eq)]
pub struct Category<'a> {
    pub name: String,
    pub options: Vec<CategoryOption<'a>>,
    pub exclusive: bool,
}

#[derive(PartialEq, Eq)]
pub struct CategoryOption<'a> {
    pub name: String,
    pub content: Vec<&'a Product<'a>>,
    pub serialization_id: usize,
}

impl<'a> Serializable for Category<'a> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Category {
            name,
            options,
            exclusive,
        } = self;
        exclusive.serialize(output);
        name.serialize(output);
        options.len().serialize(output);
        for option in options {
            option.name.serialize(output);
            option.content.len().serialize(output);
            // We take the serialization ids, sort them, and save them as the difference from the previous id, this is because smaller numbers save more efficiently
            let mut products: Vec<_> = option.content.iter().map(|p| p.serialization_id).collect();
            products.sort();
            let mut last = None;
            for id in products {
                let id = if let Some(last) = last { id - last } else { id };
                id.serialize(output);
                last = Some(id);
            }
        }
    }
}

impl<'a> Category<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        all_products: &'a Vec<Product<'a>>,
        next_serialization_id: &mut usize,
    ) -> Option<(&'i [u8], Category<'a>)> {
        let (input, exclusive) = bool::deserialize(input)?;
        let (input, name) = String::deserialize(input)?;
        let (mut input, options_len) = usize::deserialize(input)?;

        let mut options = Vec::with_capacity(options_len);
        for _ in 0..options_len {
            let (input_after_name, name) = String::deserialize(input)?;
            let (input_after_len, products_len) = usize::deserialize(input_after_name)?;
            input = input_after_len;
            let mut content = Vec::with_capacity(products_len);
            let mut last = None;
            for _ in 0..products_len {
                let (new_input, found) = usize::deserialize(input)?;
                input = new_input;
                let id = if let Some(last) = last {
                    last + found
                } else {
                    found
                };
                // Then we insert a reference to the offset as the product
                content.push(all_products.get(id)?);
                last = Some(found);
            }

            let serialization_id = *next_serialization_id;
            *next_serialization_id += 1;

            options.push(CategoryOption {
                name,
                content,
                serialization_id,
            })
        }

        Some((
            input,
            Category {
                exclusive,
                name,
                options,
            },
        ))
    }

    pub fn deserialize_many<'i>(
        input: &'i [u8],
        products: &'a Vec<Product<'a>>,
    ) -> Option<(&'i [u8], Vec<Category<'a>>)> {
        let (mut input, len) = usize::deserialize(input)?;
        let mut cats = Vec::with_capacity(len);
        let mut next_option_serialization_id = 0;
        for _ in 0..len {
            let (new_input, cat) =
                Category::deserialize(input, &products, &mut next_option_serialization_id)?;
            input = new_input;
            cats.push(cat);
        }
        Some((input, cats))
    }

    pub fn serialize_many(input: &Vec<Category<'a>>, output: &mut Vec<u8>) {
        input.len().serialize(output);
        for cat in input.iter() {
            cat.serialize(output);
        }
    }

    pub fn new(name: String) -> Category<'a> {
        Category {
            name,
            options: Vec::new(),
            exclusive: false,
        }
    }
}

impl<'a> CategoryOption<'a> {
    pub fn new(name: String, serialization_id: usize) -> CategoryOption<'a> {
        CategoryOption {
            name,
            content: Vec::new(),
            serialization_id,
        }
    }
}
