use std::sync::Arc;

use crate::{
    ngram::HashExtractable,
    serialize::{Deserializable, Serializable},
};

use super::{
    tags::{Tag, TagManager},
    vendor::{Vendor, VendorManager},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Product<'a> {
    pub description: String,
    pub tags: Vec<&'a Tag>,
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
            tags,
            title,
            vendor,
            id,
            ..
        } = self;

        for field in [description, title, id] {
            field.serialize(output)
        }

        // Vendor and tags are just saved as their id's
        vendor.id.serialize(output);
        tags.len().serialize(output);
        for tag in tags {
            tag.serialize(output);
        }
    }
}

impl<'a> Product<'a> {
    pub fn deserialize<'i>(
        input: &'i [u8],
        serialization_id: usize,
        tag_manager: Arc<TagManager<'a>>,
        vendors: Arc<VendorManager<'a>>,
    ) -> Option<(&'i [u8], Self)> {
        let (input, description) = String::deserialize(input)?;
        let (input, title) = String::deserialize(input)?;
        let (input, id) = String::deserialize(input)?;

        let (input, vendor_id) = usize::deserialize(input)?;
        let vendor = *vendors.by_id.get(vendor_id)?;

        let (input, tags_amount) = usize::deserialize(input)?;
        let mut tags = Vec::with_capacity(tags_amount);

        let mut input = input;
        for _ in 0..tags_amount {
            let (new_input, tag_id) = usize::deserialize(input)?;
            input = new_input;
            tags.push(*tag_manager.by_id.get(tag_id)?);
        }

        Some((
            input,
            Product {
                description,
                tags,
                title,
                vendor,
                id,
                serialization_id,
            },
        ))
    }
}
