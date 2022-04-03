use std::{collections::HashMap, sync::Arc};

use crate::classic_indexes::ClassicIndexes;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CategoryHandler {
    handle: Arc<ClassicIndexes<'static>>,
    activated: HashMap<usize, ()>,
}

#[wasm_bindgen]
impl CategoryHandler {
    pub fn toggle(&mut self, item: ExportCategoryOption) {
        use std::collections::hash_map::Entry;
        match self.activated.entry(item.id) {
            Entry::Occupied(v) => v.remove(),
            Entry::Vacant(space) => {
                space.insert(());
            }
        }
    }

    pub fn get_state(&self, item: ExportCategoryOption) -> bool {
        self.activated.contains_key(&item.id)
    }

    pub fn iter(&self) -> CategoryIter {
        CategoryIter {
            handle: self.handle.clone(),
            index: 0,
        }
    }
}

impl CategoryHandler {
    pub fn new(handle: Arc<ClassicIndexes<'static>>) -> CategoryHandler {
        CategoryHandler {
            handle,
            activated: HashMap::new(),
        }
    }
}

#[wasm_bindgen]
pub struct CategoryIter {
    handle: Arc<ClassicIndexes<'static>>,
    index: usize,
}

#[wasm_bindgen]
impl CategoryIter {
    pub fn next(&mut self) -> Option<CategoryOptionIter> {
        self.handle.categories.get(self.index)?;
        self.index += 1;
        Some(CategoryOptionIter {
            handle: self.handle.clone(),
            index: 0,
            category_index: self.index - 1,
        })
    }
}

#[wasm_bindgen]
pub struct CategoryOptionIter {
    handle: Arc<ClassicIndexes<'static>>,
    category_index: usize,
    index: usize,
}

#[wasm_bindgen]
impl CategoryOptionIter {
    pub fn next(&mut self) -> Option<ExportCategoryOption> {
        let out = self
            .handle
            .categories
            .get(self.category_index)?
            .options
            .get(self.index)?;
        self.index += 1;
        Some(ExportCategoryOption {
            name: out.name.clone(),
            id: out.serialization_id,
        })
    }

    pub fn name(&self) -> String {
        self.handle.categories[self.category_index].name.clone()
    }
}

#[wasm_bindgen]
pub struct ExportCategoryOption {
    name: String,
    id: usize,
}

#[wasm_bindgen]
impl ExportCategoryOption {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
