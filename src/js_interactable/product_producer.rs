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
        let container = self.container;
        // Make sure this product exists
        let _ = container.products.get(next_id)?;
        self.index += 1;

        Some(JsProduct {
            container,
            serialization_id: next_id,
        })
    }
}

#[wasm_bindgen]
pub struct JsProduct {
    container: &'static ProductContainer<'static>,
    serialization_id: usize,
}

#[wasm_bindgen]
impl JsProduct {
    fn product(&self) -> &Product<'_> {
        &self.container.products[self.serialization_id]
    }

    pub fn numeric_feature(&self, key: &str) -> Option<f64> {
        let features = &self.container.extra_features;
        match features.get(self.product(), key)? {
            FeatureValue::Float(f) => Some(f64::from(f)),
            FeatureValue::Integer(i) => Some(f64::from(i)),
            FeatureValue::String(_) => None,
        }
    }

    pub fn string_feature(&self, key: &str) -> Option<String> {
        let features = &self.container.extra_features;
        if let Some(FeatureValue::String(string)) = features.get(self.product(), key) {
            Some(string.to_string())
        } else {
            None
        }
    }

    pub fn get_title(&self) -> String {
        self.product().title.clone()
    }

    pub fn get_description(&self) -> String {
        self.product().description.clone()
    }

    pub fn get_id(&self) -> String {
        self.product().id.clone()
    }
}
