use colosseum::sync::Arena;
use std::sync::Arc;

use crate::serialize::{ArenaDeserializableCollection, Deserializable, Serializable};

use super::{
    tags::{Tag, TagManager},
    vendor::VendorManager,
    Product,
};

pub struct ProductContainer<'a> {
    pub products: Vec<Product<'a>>,
    pub tags: Arc<TagManager<'a>>,
    pub vendors: Arc<VendorManager<'a>>,
}

impl<'a> ProductContainer<'a> {
    pub fn new(
        products: Vec<Product<'a>>,
        tags: Arc<TagManager<'a>>,
        vendors: Arc<VendorManager<'a>>,
    ) -> ProductContainer<'a> {
        ProductContainer {
            products,
            tags,
            vendors,
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
        let tag_arena: &'a Arena<Tag> = super_alloc.alloc(Arena::new());
        let (input, tags): (&'input [u8], TagManager<'a>) =
            TagManager::deserialize_arena(input, tag_arena)?;

        let tags = Arc::new(tags);

        let (input, vendors) =
            VendorManager::deserialize_arena(input, super_alloc.alloc(Arena::new()))?;

        let vendors = Arc::new(vendors);

        let (mut input, product_count) = usize::deserialize(input)?;
        let products = {
            let mut products = Vec::new();
            for id in 0..product_count {
                let (new_input, product) =
                    Product::deserialize(input, id, tags.clone(), vendors.clone())?;
                products.push(product);
                input = new_input;
            }
            products
        };

        Some((
            input,
            ProductContainer {
                products,
                tags,
                vendors,
            },
        ))
    }
}

impl<'a> Serializable for ProductContainer<'a> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let ProductContainer {
            products,
            tags,
            vendors,
        } = self;
        tags.serialize(output);
        vendors.serialize(output);

        products.serialize(output);
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
    // pub fn alloc<'a, T: Any>(&'a self, item: T) -> &'a T {
    //     let any = self.0.alloc(Box::new(item));

    //     let real_ref = any.downcast_ref().unwrap();

    //     real_ref
    // }

    pub fn alloc_mut<T: Any + Send>(&self, item: T) -> &mut T {
        let any = self.0.alloc(Box::new(item));

        let real_ref = any.downcast_mut().unwrap();

        real_ref
    }

    pub fn new() -> SuperAlloc {
        SuperAlloc(Arena::new())
    }
}
