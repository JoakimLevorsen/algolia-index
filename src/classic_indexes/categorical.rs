use ahash::AHashSet;

use crate::{
    data::{Product, ProductContainer, RawProductOption},
    serialize::{Deserializable, Serializable},
};

#[derive(PartialEq, Eq)]
pub struct CategoryIndex<'a>(pub Vec<Category<'a>>);

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
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
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

impl<'a> CategoryIndex<'a> {
    pub fn index(
        options_list: Vec<Vec<RawProductOption>>,
        container: &'a ProductContainer<'a>,
    ) -> CategoryIndex<'a> {
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
                            let new =
                                CategoryOption::new(raw_value.to_string(), next_serialization_id);
                            next_serialization_id += 1;
                            category.options.push(new);
                            let just_inserted = category.options.len() - 1;
                            category.options.get_mut(just_inserted).unwrap()
                        }
                    };

                    option.add(&container.products[i]);
                }
            }
        }

        CategoryIndex(categories)
    }

    pub fn deserialize_many<'i>(
        input: &'i [u8],
        products: &'a [Product<'a>],
    ) -> Option<(&'i [u8], CategoryIndex<'a>)> {
        let (mut input, len) = usize::deserialize(input)?;
        let mut cats = Vec::with_capacity(len);
        let mut next_option_serialization_id = 0;
        for _ in 0..len {
            let (new_input, cat) =
                Category::deserialize(input, products, &mut next_option_serialization_id)?;
            input = new_input;
            cats.push(cat);
        }
        Some((input, CategoryIndex(cats)))
    }

    pub fn get(&'a self, index: usize) -> Option<&'a Category<'a>> {
        self.0.get(index)
    }
}

impl Serializable for CategoryIndex<'_> {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        self.0.len().serialize(output);
        for cat in &self.0 {
            cat.serialize(output);
        }
    }
}

impl<'a> std::ops::Index<usize> for CategoryIndex<'a> {
    type Output = Category<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
