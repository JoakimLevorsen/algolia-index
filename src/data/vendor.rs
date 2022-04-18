use ahash::AHashMap;
use colosseum::sync::Arena;

use crate::serialize::{ArenaDeserializableCollection, Deserializable, Serializable};

use super::container::SuperAlloc;

#[derive(Debug, PartialEq, Eq)]
pub struct Vendor {
    pub name: String,
    pub id: usize,
}

pub struct VendorManager<'a> {
    tags: AHashMap<&'a str, &'a Vendor>,
    pub by_id: Vec<&'a Vendor>,
    alloc: &'a Arena<Vendor>,
}

impl<'a> PartialEq for VendorManager<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags && self.by_id == other.by_id
    }
}

impl<'a> Eq for VendorManager<'a> {}

impl<'a> VendorManager<'a> {
    pub fn get_or_insert(&'a mut self, name: &str) -> &'a Vendor {
        if let Some(tag) = self.tags.get(name) {
            *tag
        } else {
            let id = self.tags.len();
            let new_tag = &*self.alloc.alloc(Vendor {
                name: name.to_string(),
                id,
            });
            self.tags.insert(&new_tag.name, new_tag);
            self.by_id.push(new_tag);
            new_tag
        }
    }

    pub fn get(&'a self, name: &str) -> Option<&'a Vendor> {
        self.tags.get(name).copied()
    }

    pub fn insert(&mut self, name: &str) {
        if self.tags.contains_key(name) {
            return;
        }
        let id = self.tags.len();
        let new_tag = &*self.alloc.alloc(Vendor {
            name: name.to_string(),
            id,
        });
        self.tags.insert(&new_tag.name, new_tag);
        self.by_id.push(new_tag);
    }

    pub fn new(alloc: &'a SuperAlloc) -> VendorManager<'a> {
        let alloc = alloc.alloc(Arena::new());
        VendorManager {
            tags: AHashMap::new(),
            by_id: Vec::new(),
            alloc,
        }
    }
}

impl Serializable for Vendor {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        // Note we don't save id' since its implied by position in binary stream
        self.name.serialize(output);
    }
}

impl<'a> Serializable for VendorManager<'a> {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        let mut all_tags: Vec<_> = self.tags.values().collect();
        // We sort by the numbers, so we don't have to save those
        all_tags.sort_by(|a, b| a.id.cmp(&b.id));
        // Then we save just the tags
        all_tags.len().serialize(output);
        for tag in &all_tags {
            tag.serialize(output);
        }
    }
}

impl<'arena> ArenaDeserializableCollection<'arena, Vendor> for VendorManager<'arena> {
    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<Vendor>,
    ) -> Option<(&'input [u8], Self)>
    where
        'arena: 'input,
    {
        let (mut input, len) = usize::deserialize(input)?;
        let mut tags = AHashMap::with_capacity(len);
        let mut by_id = Vec::with_capacity(len);
        for id in 0..len {
            let (new_input, name) = String::deserialize(input)?;
            input = new_input;
            let tag = &*arena.alloc(Vendor { name, id });
            tags.insert(tag.name.as_str(), tag);
            by_id.push(tag);
        }

        Some((
            input,
            VendorManager {
                alloc: arena,
                tags,
                by_id,
            },
        ))
    }
}
