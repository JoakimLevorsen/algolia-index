use wasm_bindgen::prelude::*;

pub struct FeatureFilter {
    data: FilterData,
    feature: String,
}

pub enum FilterData {
    Range { from: JsValue, to: JsValue },
    Exact(JsValue),
}

impl FeatureFilter {
    pub fn new_range(from: JsValue, to: JsValue, feature: String) -> Self {
        Self {
            data: FilterData::Range { from, to },
            feature,
        }
    }

    pub fn new_exactly(exactly: JsValue, feature: String) -> Self {
        Self {
            data: FilterData::Exact(exactly),
            feature,
        }
    }
}

impl FeatureFilter {
    pub fn parse(input: &js_sys::Object) -> Option<Vec<FeatureFilter>> {
        fn get_value(object: &JsValue, key: &str) -> Option<JsValue> {
            let v = js_sys::Reflect::get(object, &key.to_string().into()).ok()?;
            Some(v)
        }
        let mut out = Vec::new();
        for entry in js_sys::Object::entries(input).iter() {
            let entry = js_sys::Array::try_from(entry).ok()?;
            let feature = entry.get(0).as_string()?;
            let values = entry.get(1);
            if let Some(exact) = get_value(&values, "exact") {
                out.push(FeatureFilter {
                    feature,
                    data: FilterData::Exact(exact),
                });
            } else {
                let from = get_value(&values, "from")?;
                let to = get_value(&values, "to")?;
                out.push(FeatureFilter {
                    data: FilterData::Range { from, to },
                    feature,
                });
            }
        }
        Some(out)
    }
}
