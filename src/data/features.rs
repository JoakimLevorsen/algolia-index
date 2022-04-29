use ahash::AHashMap;
use wasm_bindgen::JsValue;

use crate::serialize::{Deserializable, Serializable};

use super::Product;

#[derive(PartialEq, Eq)]
pub struct FeatureSet(AHashMap<String, Feature>);

#[derive(PartialEq)]
pub enum Feature {
    String(Vec<String>),
    Float(Vec<f32>),
    Integer(Vec<u32>),
}

impl Feature {
    pub fn get(&self, index: usize) -> Option<FeatureValue<'_>> {
        match self {
            Feature::String(list) => {
                let item = list.get(index)?;
                Some(FeatureValue::String(item))
            }
            Feature::Float(list) => {
                let item = *list.get(index)?;
                Some(FeatureValue::Float(item))
            }
            Feature::Integer(list) => {
                let item = *list.get(index)?;
                Some(FeatureValue::Integer(item))
            }
        }
    }
}

pub enum FeatureValue<'a> {
    String(&'a str),
    Float(f32),
    Integer(u32),
}

impl Eq for Feature {}

impl FeatureSet {
    pub fn get_js(&self, product: &Product<'_>, key: &str) -> Option<JsValue> {
        #[allow(clippy::cast_possible_truncation)]
        let id = product.serialization_id;
        Some(match self.0.get(key)? {
            Feature::String(list) => list.get(id)?.as_str().into(),
            Feature::Float(list) => {
                let n = *list.get(id)?;
                n.into()
            }
            Feature::Integer(list) => {
                let n = *list.get(id)?;
                let n = f64::from(n);
                JsValue::from_f64(n)
            }
        })
    }

    pub fn get<'a>(&'a self, product: &Product<'_>, key: &str) -> Option<FeatureValue<'a>> {
        #[allow(clippy::cast_possible_truncation)]
        let id = product.serialization_id;
        Some(match self.0.get(key)? {
            Feature::String(list) => FeatureValue::String(list.get(id)?),
            Feature::Float(list) => FeatureValue::Float(*list.get(id)?),
            Feature::Integer(list) => FeatureValue::Integer(*list.get(id)?),
        })
    }

    pub fn get_feature(&self, key: &str) -> Option<&Feature> {
        self.0.get(key)
    }

    pub fn add_int(&mut self, key: &str, value: u32) {
        match self.0.get_mut(key) {
            Some(v) => match v {
                Feature::Integer(i) => i.push(value),
                _ => panic!("Expected Integer feature"),
            },
            None => {
                self.0
                    .insert(key.to_string(), Feature::Integer(vec![value]));
            }
        };
    }

    pub fn add_float(&mut self, key: &str, value: f32) {
        match self.0.get_mut(key) {
            Some(v) => match v {
                Feature::Float(i) => i.push(value),
                _ => panic!("Expected Integer feature"),
            },
            None => {
                self.0.insert(key.to_string(), Feature::Float(vec![value]));
            }
        };
    }

    pub fn add_string(&mut self, key: &str, value: String) {
        match self.0.get_mut(key) {
            Some(v) => match v {
                Feature::String(i) => i.push(value),
                _ => panic!("Expected Integer feature"),
            },
            None => {
                self.0.insert(key.to_string(), Feature::String(vec![value]));
            }
        };
    }

    pub fn new_empty() -> FeatureSet {
        FeatureSet(AHashMap::new())
    }
}

impl Serializable for Feature {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        match self {
            Feature::String(list) => {
                0u8.serialize(output);
                list.serialize(output);
            }
            Feature::Float(list) => {
                1u8.serialize(output);
                list.serialize(output);
            }
            Feature::Integer(list) => {
                2u8.serialize(output);
                list.serialize(output);
            }
        }
    }
}

impl Deserializable for Feature {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (input, id) = u8::deserialize(input)?;
        match id {
            0 => {
                let (input, data) = Deserializable::deserialize(input)?;
                Some((input, Feature::String(data)))
            }
            1 => {
                let (input, data) = Deserializable::deserialize(input)?;
                Some((input, Feature::Float(data)))
            }
            2 => {
                let (input, data) = Deserializable::deserialize(input)?;
                Some((input, Feature::Integer(data)))
            }
            _ => None,
        }
    }
}

impl Serializable for FeatureSet {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        self.0.serialize(output);
    }
}

impl Deserializable for FeatureSet {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (input, content) = Deserializable::deserialize(input)?;
        Some((input, FeatureSet(content)))
    }
}
