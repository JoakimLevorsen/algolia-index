use wasm_bindgen::prelude::*;

use crate::data::{FeatureValue, Product, ProductContainer};

#[wasm_bindgen]
pub struct ProductProducer {
    container: &'static ProductContainer<'static>,
    to_export: Vec<usize>,
    index: usize,
}

impl ProductProducer {
    pub fn new(container: &'static ProductContainer<'static>, to_export: Vec<usize>) -> Self {
        Self {
            container,
            to_export,
            index: 0,
        }
    }
}

#[wasm_bindgen]
impl ProductProducer {
    pub fn next_product(&mut self) -> Option<JsProduct> {
        let next_id = *self.to_export.get(self.index)?;
        let next = self.container.products.get(next_id)?;
        self.index += 1;

        let Product {
            description,
            title,
            id,
            ..
        } = next;

        let price = self.container.extra_features.get(next, "price")?;

        let price = match price {
            FeatureValue::Float(f) => f,
            _ => return None,
        };

        Some(JsProduct {
            title: title.clone(),
            description: description.clone(),
            id: id.clone(),
            price,
        })
    }
}

#[wasm_bindgen]
pub struct JsProduct {
    title: String,
    description: String,
    pub price: f32,
    id: String,
}

#[wasm_bindgen]
impl JsProduct {
    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
