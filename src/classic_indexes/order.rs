use ahash::AHashMap;

use crate::{
    data::{FeatureSet, Product, ProductContainer},
    serialize::{Deserializable, Serializable},
};

#[derive(Debug, PartialEq, Eq)]
pub struct OrderIndex {
    orders: AHashMap<String, Vec<usize>>,
}

impl OrderIndex {
    pub fn add<'a, T: PartialOrd + 'a>(
        &mut self,
        products: &'a ProductContainer<'a>,
        maker: &'a dyn Fn(&'a Product, &'a FeatureSet) -> T,
        key: String,
    ) {
        self.add_custom_cmp(
            products,
            maker,
            &|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            key,
        );
    }

    pub fn add_custom_cmp<'a, T: 'a>(
        &mut self,
        products: &'a ProductContainer<'a>,
        maker: &'a dyn Fn(&'a Product, &'a FeatureSet) -> T,
        cmp: &'a dyn Fn(&T, &T) -> std::cmp::Ordering,
        key: String,
    ) {
        let features = &products.extra_features;
        let mut data_and_id: Vec<(T, usize)> = products
            .products
            .iter()
            .map(|prod| {
                let id = prod.serialization_id;
                let v = maker(prod, features);
                (v, id)
            })
            .collect();
        data_and_id.sort_by(|(a, _), (b, _)| cmp(a, b));

        let mut order: Vec<usize> = vec![0; data_and_id.len()];

        // We can then use data_serials order since that is the order these products should be sorted after
        #[allow(clippy::cast_possible_truncation)]
        for (order_position, (_, serialization_id)) in data_and_id.into_iter().enumerate() {
            order[serialization_id] = order_position;
        }

        self.orders.insert(key, order);
    }

    pub fn new() -> Self {
        Self {
            orders: AHashMap::new(),
        }
    }

    pub fn options(&self) -> impl Iterator<Item = &str> {
        self.orders.keys().map(String::as_str)
    }
}

impl Default for OrderIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl Deserializable for OrderIndex {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (input, orders) = Deserializable::deserialize(input)?;
        Some((input, OrderIndex { orders }))
    }
}

impl Serializable for OrderIndex {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        self.orders.serialize(output);
    }
}
