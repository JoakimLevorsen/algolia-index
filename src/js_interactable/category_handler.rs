use std::sync::Arc;

use crate::{classic_indexes::ClassicIndexes, data::Product};
use ahash::AHashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CategoryHandler {
    handle: Arc<ClassicIndexes<'static>>,
    active: AHashMap<(usize, usize), ()>,
    general_observers: Vec<js_sys::Function>,
    option_observers: AHashMap<(usize, usize), Vec<js_sys::Function>>,
}

#[wasm_bindgen]
impl CategoryHandler {
    pub fn toggle(&mut self, item: &ExportCategoryOption) {
        use std::collections::hash_map::Entry;
        let new_state = match self.active.entry(item.keys()) {
            Entry::Occupied(v) => {
                v.remove();
                false
            }
            Entry::Vacant(space) => {
                space.insert(());
                true
            }
        };

        let observers_for_this_option = self
            .option_observers
            .get(&item.keys())
            .map_or(&[][..], |vec| &vec[..])
            .iter();

        let all_observers = observers_for_this_option.chain(self.general_observers.iter());

        let new_js_state = JsValue::from(new_state);

        for observer in all_observers {
            if let Err(e) = observer.call1(&JsValue::UNDEFINED, &new_js_state) {
                println!("Failed to call handler with error: {:?}", e);
            }
        }
    }

    pub fn get_status(&self, item: &ExportCategoryOption) -> bool {
        self.active.contains_key(&item.keys())
    }

    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> CategoryIter {
        CategoryIter {
            handle: self.handle.clone(),
            index: 0,
        }
    }

    pub fn add_general_observer(&mut self, observer: js_sys::Function) {
        self.general_observers.push(observer);
    }

    pub fn add_option_lister(&mut self, option: &ExportCategoryOption, observer: js_sys::Function) {
        use std::collections::hash_map::Entry;
        match self.option_observers.entry(option.keys()) {
            Entry::Occupied(mut vec) => vec.get_mut().push(observer),
            Entry::Vacant(empty) => {
                empty.insert(vec![observer]);
            }
        }
    }

    pub fn remove_option_listener(
        &mut self,
        option: &ExportCategoryOption,
        observer: &js_sys::Function,
    ) {
        let current_observers = match self.option_observers.get_mut(&option.keys()) {
            Some(v) => v,
            None => return,
        };
        for (index, obs) in current_observers.iter().enumerate() {
            if observer == obs {
                current_observers.swap_remove(index);
                return;
            }
        }
    }
}

impl CategoryHandler {
    pub fn new(handle: Arc<ClassicIndexes<'static>>) -> CategoryHandler {
        CategoryHandler {
            handle,
            active: AHashMap::new(),
            general_observers: Vec::new(),
            option_observers: AHashMap::new(),
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
        true
    }
}

#[wasm_bindgen]
pub struct CategoryIter {
    handle: Arc<ClassicIndexes<'static>>,
    index: usize,
}

#[wasm_bindgen]
impl CategoryIter {
    pub fn next_item(&mut self) -> Option<CategoryOptionIter> {
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
    pub fn next_item(&mut self) -> Option<ExportCategoryOption> {
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
