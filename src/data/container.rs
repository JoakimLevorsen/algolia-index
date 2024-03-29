use colosseum::sync::Arena;
use std::sync::Arc;

use crate::serialize::{ArenaDeserializableCollection, Deserializable, Serializable};

use super::{vendor::VendorManager, FeatureSet, Product};

#[derive(PartialEq, Eq)]
pub struct ProductContainer<'a> {
    pub products: Vec<Product<'a>>,
    pub vendors: Arc<VendorManager<'a>>,
    pub extra_features: FeatureSet,
}

impl<'a> ProductContainer<'a> {
    pub fn new(
        products: Vec<Product<'a>>,
        vendors: Arc<VendorManager<'a>>,
        extra_features: FeatureSet,
    ) -> ProductContainer<'a> {
        ProductContainer {
            products,
            vendors,
            extra_features,
        }
    }

    pub fn deserialize<'input, 'outerarena>(
        input: &'input [u8],
        super_alloc: &'a SuperAlloc,
    ) -> Option<(&'input [u8], ProductContainer<'a>)>
    where
        'outerarena: 'a,
        'a: 'input,
    {
        let (input, vendors) =
            VendorManager::deserialize_arena(input, super_alloc.alloc(Arena::new()))?;

        let vendors = Arc::new(vendors);

        let (mut input, product_count) = usize::deserialize(input)?;
        let (input, products) = {
            let mut products = Vec::new();
            for id in 0..product_count {
                let (new_input, product) = Product::deserialize(input, id, &vendors)?;
                products.push(product);
                input = new_input;
            }
            (input, products)
        };
        let (input, extra_features) = FeatureSet::deserialize(input)?;

        Some((
            input,
            ProductContainer {
                products,
                vendors,
                extra_features,
            },
        ))
    }
}

impl<'a> Serializable for ProductContainer<'a> {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        let ProductContainer {
            products,
            vendors,
            extra_features,
        } = self;
        vendors.serialize(output);

        products.serialize(output);
        extra_features.serialize(output);
    }
}

use std::any::Any;

// A type to collect all allocators into one
pub struct SuperAlloc(Arena<Box<dyn Any + Send>>);

impl SuperAlloc {
    pub fn alloc<T: Any + Send>(&self, item: T) -> &T {
        let any = self.0.alloc(Box::new(item));

        let real_ref = any.downcast_ref().unwrap();

        real_ref
    }

    #[allow(clippy::mut_from_ref)]
    pub fn alloc_mut<T: Any + Send>(&self, item: T) -> &mut T {
        let any = self.0.alloc(Box::new(item));

        let real_ref = any.downcast_mut().unwrap();

        real_ref
    }

    pub fn new() -> SuperAlloc {
        SuperAlloc(Arena::new())
    }
}

impl Default for SuperAlloc {
    fn default() -> Self {
        Self::new()
    }
}
