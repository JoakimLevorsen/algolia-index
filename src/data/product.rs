use std::sync::Arc;

use crate::{
    ngram::HashExtractable,
    serialize::{sequential_array, Deserializable, Serializable},
};

use super::vendor::{Vendor, VendorManager};

#[derive(Debug)]
pub struct Product<'a> {
    pub description: String,
    pub title: String,
    pub vendor: &'a Vendor,
    pub id: String,
    pub serialization_id: usize,
}

impl Product<'_> {
    pub fn get_id(&self) -> u64 {
        let mut out = 0;
        let mut multiplier = 1;
        for char in self.id.chars().rev() {
            let n = match char.to_digit(10) {
                Some(v) => v as u64,
                None => break,
            };
            out += n * multiplier;
            multiplier *= 10;
        }
        out
    }
}

impl Eq for Product<'_> {}

impl PartialEq for Product<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Descriptions might be of different length after serialization, so we just need the beginning to match
        match (self.description.len(), other.description.len()) {
            (0, 1..) | (1.., 0) => return false,
            (my_len, other_len) if my_len > other_len => {
                if self.description.starts_with(&other.description) == false {
                    return false;
                }
            }
            (my_len, other_len) if my_len < other_len => {
                if other.description.starts_with(&self.description) == false {
                    return false;
                }
            }
            _ => {
                if self.description != other.description {
                    return false;
                }
            }
        }

        self.title == other.title
            && self.vendor == other.vendor
            && self.id == other.id
            && self.serialization_id == other.serialization_id
    }
}

impl<'a> PartialOrd for Product<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Product<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // TODO: Improve
        self.id.cmp(&other.id)
    }
}

impl<'a> HashExtractable for &'a Product<'a> {
    type Inner = String;

    fn extract(&self) -> &Self::Inner {
        &self.id
    }
}

impl Serializable for Product<'_> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Product {
            description,
            title,
            vendor,
            id,
            ..
        } = self;

        crate::serialize::serialize_string_with_limit(description, 100, output);
        for field in [title, id] {
            field.serialize(output)
        }

        // Vendor and tags are just saved as their id's
        vendor.id.serialize(output);
    }
}

impl<'a> Product<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        serialization_id: usize,
        vendors: Arc<VendorManager<'a>>,
    ) -> Option<(&'i [u8], Self)> {
        let (input, description) = String::deserialize(input)?;
        let (input, title) = String::deserialize(input)?;
        let (input, id) = String::deserialize(input)?;

        let (input, vendor_id) = usize::deserialize(input)?;
        let vendor = *vendors.by_id.get(vendor_id)?;

        Some((
            input,
            Product {
                description,
                title,
                vendor,
                id,
                serialization_id,
            },
        ))
    }

    pub fn serialize_to_sequential_array(input: &[&Product<'_>], output: &mut Vec<u8>) {
        sequential_array::serialize(input.iter().map(|v| v.serialization_id), output);
    }

    pub fn deserialize_from_sequential_ids<'i>(
        input: &'i [u8],
        existing_products: &'a [Product<'a>],
    ) -> Option<(&'i [u8], Vec<&'a Product<'a>>, Vec<usize>)> {
        let (input, product_ids) = sequential_array::deserialize(input)?;

        let mut products = Vec::with_capacity(product_ids.len());
        for id in product_ids.iter() {
            let product = existing_products.get(*id)?;
            products.push(product);
        }

        Some((input, products, product_ids))
    }
}
