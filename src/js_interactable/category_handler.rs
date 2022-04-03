use std::{collections::HashMap, sync::Arc};

use crate::{classic_indexes::ClassicIndexes, data::Product};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CategoryHandler {
    handle: Arc<ClassicIndexes<'static>>,
    active: HashMap<(usize, usize), ()>,
}

#[wasm_bindgen]
impl CategoryHandler {
    pub fn toggle(&mut self, item: ExportCategoryOption) {
        use std::collections::hash_map::Entry;
        match self.active.entry(item.keys()) {
            Entry::Occupied(v) => v.remove(),
            Entry::Vacant(space) => {
                space.insert(());
            }
        }
    }

    pub fn get_state(&self, item: ExportCategoryOption) -> bool {
        self.active.contains_key(&item.keys())
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
            active: HashMap::new(),
        }
    }

    pub fn is_valid(&self, product: &Product<'_>) -> bool {
        for (category, option) in self.active.keys() {
            let category = &self.handle.categories[*category];
            let option = &category.options[*option];
            if option.contains(product) == false {
                return false;
            }
        }
        false
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
        let out = Some(ExportCategoryOption {
            name: out.name.clone(),
            cat_id: self.category_index,
            option_id: self.index,
        });
        self.index += 1;
        out
    }

    pub fn name(&self) -> String {
        self.handle.categories[self.category_index].name.clone()
    }
}

#[wasm_bindgen]
pub struct ExportCategoryOption {
    name: String,
    cat_id: usize,
    option_id: usize,
}

#[wasm_bindgen]
impl ExportCategoryOption {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl ExportCategoryOption {
    pub fn keys(&self) -> (usize, usize) {
        let ExportCategoryOption {
            cat_id, option_id, ..
        } = self;
        (*cat_id, *option_id)
    }
}
