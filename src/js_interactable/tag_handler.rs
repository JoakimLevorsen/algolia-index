use std::sync::Arc;

use ahash::AHashMap;
use wasm_bindgen::prelude::*;

use crate::{classic_indexes::ClassicIndexes, data::Product};

#[wasm_bindgen]
pub struct TagHandler {
    handle: Arc<ClassicIndexes<'static>>,
    active: AHashMap<usize, ()>,
    general_observers: Vec<js_sys::Function>,
    tag_observers: AHashMap<usize, Vec<js_sys::Function>>,
}

#[wasm_bindgen]
impl TagHandler {
    pub fn tags(&self) -> TagIter {
        TagIter {
            handle: self.handle.clone(),
            index: 0,
        }
    }

    pub fn toggle(&mut self, tag: &JSTag) {
        use std::collections::hash_map::Entry;
        let new_state = match self.active.entry(tag.get_id()) {
            Entry::Occupied(v) => {
                v.remove();
                false
            }
            Entry::Vacant(space) => {
                space.insert(());
                true
            }
        };

        let observers_for_this_tag = self
            .tag_observers
            .get(&tag.get_id())
            .map_or(&[][..], |vec| &vec[..])
            .iter();

        let all_observers = observers_for_this_tag.chain(self.general_observers.iter());

        let new_js_state = JsValue::from(new_state);

        for observer in all_observers {
            observer.call1(&JsValue::UNDEFINED, &new_js_state).unwrap();
        }
    }

    pub fn get_status(&self, tag: &JSTag) -> bool {
        self.active.contains_key(&tag.index)
    }

    pub fn add_general_observer(&mut self, observer: js_sys::Function) {
        self.general_observers.push(observer);
    }

    pub fn add_tag_lister(&mut self, tag: &JSTag, observer: js_sys::Function) {
        use std::collections::hash_map::Entry;
        match self.tag_observers.entry(tag.get_id()) {
            Entry::Occupied(mut vec) => vec.get_mut().push(observer),
            Entry::Vacant(empty) => {
                empty.insert(vec![observer]);
            }
        }
    }

    pub fn remove_tag_listener(&mut self, tag: &JSTag, observer: &js_sys::Function) {
        let current_observers = match self.tag_observers.get_mut(&tag.get_id()) {
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

impl TagHandler {
    pub fn new(handle: Arc<ClassicIndexes<'static>>) -> TagHandler {
        TagHandler {
            handle,
            active: AHashMap::new(),
            general_observers: Vec::new(),
            tag_observers: AHashMap::new(),
        }
    }

    pub fn is_valid(&self, product: &Product<'_>) -> bool {
        for id in self.active.keys() {
            let tag = self.handle.tags.get(*id).unwrap();
            if tag.contains(product) == false {
                return false;
            }
        }
        true
    }
}

#[wasm_bindgen]
pub struct TagIter {
    handle: Arc<ClassicIndexes<'static>>,
    index: usize,
}

#[wasm_bindgen]
impl TagIter {
    pub fn next_tag(&mut self) -> Option<JSTag> {
        let _ = self.handle.tags.get(self.index)?;
        let tag = JSTag {
            handle: self.handle.clone(),
            index: self.index,
        };
        self.index += 1;
        Some(tag)
    }
}

#[derive(Clone)]
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
