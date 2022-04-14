use ahash::AHashSet;

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
    pub serialization_id: usize,
    content: Vec<&'a Product<'a>>,
    products_by_serialization_id: AHashSet<usize>,
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

            Product::serialize_to_sequential_array(&option.content, output);
        }
    }
}

impl<'a> Category<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        all_products: &'a [Product<'a>],
        next_serialization_id: &mut usize,
    ) -> Option<(&'i [u8], Category<'a>)> {
        let (input, exclusive) = bool::deserialize(input)?;
        let (input, name) = String::deserialize(input)?;
        let (mut input, options_len) = usize::deserialize(input)?;

        let mut options = Vec::with_capacity(options_len);
        for _ in 0..options_len {
            let (input_after_name, name) = String::deserialize(input)?;

            let (new_input, products, product_ids) =
                Product::deserialize_from_sequential_ids(input_after_name, all_products)?;

            let serialization_id = *next_serialization_id;
            *next_serialization_id += 1;

            input = new_input;

            options.push(CategoryOption {
                name,
                content: products,
                serialization_id,
                products_by_serialization_id: product_ids.into_iter().collect(),
            });
        }

        Some((
            input,
            Category {
                name,
                options,
                exclusive,
            },
        ))
    }

    pub fn deserialize_many<'i>(
        input: &'i [u8],
        products: &'a [Product<'a>],
    ) -> Option<(&'i [u8], Vec<Category<'a>>)> {
        let (mut input, len) = usize::deserialize(input)?;
        let mut cats = Vec::with_capacity(len);
        let mut next_option_serialization_id = 0;
        for _ in 0..len {
            let (new_input, cat) =
                Category::deserialize(input, products, &mut next_option_serialization_id)?;
            input = new_input;
            cats.push(cat);
        }
        Some((input, cats))
    }

    pub fn serialize_many(input: &[Category<'a>], output: &mut Vec<u8>) {
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
            products_by_serialization_id: AHashSet::new(),
            serialization_id,
        }
    }

    pub fn add(&mut self, product: &'a Product) {
        // Insert if not added
        if self
            .products_by_serialization_id
            .insert(product.serialization_id)
        {
            self.content.push(product);
        }
    }

    pub fn contains(&self, product: &Product<'_>) -> bool {
        self.products_by_serialization_id
            .contains(&product.serialization_id)
    }
}
