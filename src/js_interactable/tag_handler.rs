use std::sync::Arc;

use ahash::AHashMap;
use wasm_bindgen::prelude::*;

use crate::{classic_indexes::ClassicIndexes, data::Product};

#[wasm_bindgen]
pub struct TagHandler {
    handle: Arc<ClassicIndexes<'static>>,
    active: AHashMap<usize, ()>,
}

#[wasm_bindgen]
impl TagHandler {
    pub fn tags(&self) -> TagIter {
        TagIter {
            handle: self.handle.clone(),
            index: 0,
        }
    }

    pub fn toggle(&mut self, tag: JSTag) {
        use std::collections::hash_map::Entry;
        match self.active.entry(tag.get_id()) {
            Entry::Occupied(v) => v.remove(),
            Entry::Vacant(space) => {
                space.insert(());
            }
        }
    }
}

impl TagHandler {
    pub fn new(handle: Arc<ClassicIndexes<'static>>) -> TagHandler {
        TagHandler {
            handle,
            active: AHashMap::new(),
        }
    }

    pub fn is_valid(&self, product: &Product<'_>) -> bool {
        for id in self.active.keys() {
            let tag = self.handle.tags.get(*id).unwrap();
            if tag.contains(product) == false {
                return false;
            }
        }
        false
    }
}

#[wasm_bindgen]
pub struct TagIter {
    handle: Arc<ClassicIndexes<'static>>,
    index: usize,
}

#[wasm_bindgen]
impl TagIter {
    pub fn next(&mut self) -> Option<JSTag> {
        let _ = self.handle.tags.get(self.index)?;
        let tag = JSTag {
            handle: self.handle.clone(),
            index: self.index,
        };
        self.index += 1;
        Some(tag)
    }
}

#[wasm_bindgen]
pub struct JSTag {
    handle: Arc<ClassicIndexes<'static>>,
    index: usize,
}

#[wasm_bindgen]
impl JSTag {
    pub fn get_name(&self) -> String {
        self.handle.tags.get(self.index).unwrap().name.clone()
    }
}

impl JSTag {
    pub fn get_id(&self) -> usize {
        self.handle.tags.get(self.index).unwrap().get_id()
    }

    pub fn new(handle: Arc<ClassicIndexes<'static>>, index: usize) -> JSTag {
        JSTag { handle, index }
    }
}
