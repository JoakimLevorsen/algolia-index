use std::hash::Hash;

use ahash::AHashMap;

use crate::Product;

pub trait HashExtractable {
    type Inner: Hash + Eq;

    fn extract(&self) -> &Self::Inner;
}

impl HashExtractable for Product<'_> {
    type Inner = String;

    fn extract(&self) -> &Self::Inner {
        &self.id
    }
}

impl HashExtractable for String {
    type Inner = String;

    fn extract(&self) -> &Self::Inner {
        &self
    }
}

pub struct ResultRanker<'a, H, Data: HashExtractable<Inner = H>> {
    confidence_for_data: AHashMap<&'a H, (&'a Data, f32)>,
}

impl<'a, H: Hash + Eq, Data: HashExtractable<Inner = H>> ResultRanker<'a, H, Data> {
    pub fn add<'b>(&'b mut self, data: &'a Data, confidence: f32)
    where
        'a: 'b,
    {
        let hashable = data.extract();
        self.confidence_for_data
            .entry(hashable)
            .and_modify(|(_, current)| *current += confidence)
            .or_insert((data, confidence));
    }

    pub fn export_data_by_confidence(mut self) -> Vec<(&'a Data, f32)> {
        let mut all_data: Vec<_> = self.confidence_for_data.drain().map(|(_, v)| v).collect();
        all_data.sort_by(|(_, a), (_, b)| {
            a.partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        });
        all_data
    }

    pub fn new() -> ResultRanker<'a, H, Data> {
        ResultRanker {
            confidence_for_data: AHashMap::new(),
        }
    }
}
