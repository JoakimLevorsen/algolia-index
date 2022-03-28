use crate::serialize::{ArenaDeserializableCollection, Deserializable, Serializable};
use colosseum::sync::Arena;
use std::collections::HashMap;

use super::container::SuperAlloc;

#[derive(Debug, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub id: usize,
}

// Functionally just associating a Tag with a number, with a hashmap for lookups
pub struct TagManager<'a> {
    tags: HashMap<&'a str, &'a Tag>,
    pub by_id: Vec<&'a Tag>,
    alloc: &'a Arena<Tag>,
}

impl<'a> PartialEq for TagManager<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags && self.by_id == other.by_id
    }
}

impl<'a> Eq for TagManager<'a> {}

impl<'a> TagManager<'a> {
    pub fn get_or_insert(&'a mut self, name: &str) -> &'a Tag {
        if let Some(tag) = self.tags.get(name) {
            *tag
        } else {
            let id = self.tags.len();
            let new_tag = &*self.alloc.alloc(Tag {
                name: name.to_string(),
                id,
            });
            self.tags.insert(&new_tag.name, new_tag);
            self.by_id.push(new_tag);
            new_tag
        }
    }

    pub fn get(&'a self, name: &str) -> Option<&'a Tag> {
        self.tags.get(name).map(|v| *v)
    }

    pub fn insert(&mut self, name: &str) {
        if self.tags.contains_key(name) {
            return;
        }
        let id = self.tags.len();
        let new_tag = &*self.alloc.alloc(Tag {
            name: name.to_string(),
            id,
        });
        self.tags.insert(&new_tag.name, new_tag);
        self.by_id.push(new_tag);
    }

    pub fn new(alloc: &'a SuperAlloc) -> TagManager<'a> {
        let alloc = alloc.alloc(Arena::new());
        TagManager {
            tags: HashMap::new(),
            by_id: Vec::new(),
            alloc,
        }
    }
}

impl Serializable for Tag {
    fn serialize(&self, output: &mut Vec<u8>) {
        // Note we don't save id' since its implied by position in binary stream
        self.name.serialize(output)
    }
}

impl<'a> Serializable for TagManager<'a> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let mut all_tags: Vec<_> = self.tags.values().collect();
        // We sort by the numbers, so we don't have to save those
        all_tags.sort_by(|a, b| a.id.cmp(&b.id));
        // Then we save just the tags
        all_tags.len().serialize(output);
        for tag in all_tags.iter() {
            tag.serialize(output)
        }
    }
}

impl<'arena> ArenaDeserializableCollection<'arena, Tag> for TagManager<'arena> {
    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<Tag>,
    ) -> Option<(&'input [u8], Self)>
    where
        'arena: 'input,
    {
        let (mut input, len) = usize::deserialize(input)?;
        let mut tags = HashMap::with_capacity(len);
        let mut by_id = Vec::with_capacity(len);
        for id in 0..len {
            let (new_input, name) = String::deserialize(input)?;
            input = new_input;
            let tag = &*arena.alloc(Tag { name, id });
            tags.insert(tag.name.as_str(), tag);
            by_id.push(tag);
        }

        Some((
            input,
            TagManager {
                alloc: arena,
                tags,
                by_id,
            },
        ))
    }
}
