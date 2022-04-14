use ahash::AHashMap;
use colosseum::sync::Arena;

use crate::{
    data::{ProductContainer, SuperAlloc},
    ngram::{GramAtom, GramIndex, GramNode},
    Product,
};

use super::{ArenaDeserializable, ArenaDeserializableCollection, Deserializable, Serializable};

fn serialize_node<G: GramAtom>(node: &GramNode<'_, G>, output: &mut Vec<u8>) {
    Serializable::serialize(&node.item, output);
    node.weight.serialize(output);
    node.by_occurances.serialize(output);
}

impl<G: GramAtom> Serializable for GramNode<'_, G> {
    fn serialize(&self, output: &mut Vec<u8>) {
        serialize_node(self, output);
    }
}

impl<G: GramAtom> Serializable for &GramNode<'_, G> {
    fn serialize(&self, output: &mut Vec<u8>) {
        serialize_node(self, output);
    }
}

impl<'arena, G: GramAtom> ArenaDeserializable<'arena, Self> for GramNode<'arena, G> {
    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<Self>,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, item) = <G as Deserializable>::deserialize(input)?;
        let (input, weight) = f32::deserialize(input)?;
        let (input, by_occurances) = Vec::deserialize_arena(input, arena)?;
        let items = by_occurances.iter().fold(AHashMap::new(), |mut map, item| {
            map.insert(item.item, *item);
            map
        });

        let node: &'arena GramNode<'arena, G> = arena.alloc(GramNode {
            item,
            weight,
            by_occurances,
            items,
        });
        Some((input, node))
    }
}

impl<G: GramAtom, const N: usize> Serializable for GramIndex<'_, G, Product<'_>, N> {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.product_container.serialize(output);
        self.roots.serialize(output);

        // We replace the product refs with their serialization id and save as a sequential array
        self.data.len().serialize(output);
        for (key, products) in self.data.iter() {
            key.serialize(output);
            Product::serialize_to_sequential_array(products, output);
        }
    }
}

impl<'arena, G: GramAtom, const N: usize> GramIndex<'arena, G, Product<'arena>, N> {
    pub fn deserialize<'input, 'super_arena>(
        input: &'input [u8],
        node_arena: &'arena Arena<GramNode<'arena, G>>,
        super_alloc: &'static SuperAlloc,
    ) -> Option<(&'input [u8], Self)>
    where
        'super_arena: 'arena,
        'arena: 'input,
    {
        let (input, container) = ProductContainer::deserialize(input, super_alloc)?;
        let container = super_alloc.alloc(container);
        let (input, roots) = AHashMap::deserialize_arena(input, node_arena)?;

        let (mut input, data_len) = usize::deserialize(input)?;
        let mut data = AHashMap::with_capacity(data_len);
        for _ in 0..data_len {
            let (next_input, gram) = <[G; N]>::deserialize(input)?;
            let (next_input, products, _) =
                Product::deserialize_from_sequential_ids(next_input, &container.products)?;
            input = next_input;
            data.insert(gram, products);
        }

        Some((
            input,
            GramIndex {
                product_container: container,
                roots,
                data,
            },
        ))
    }
}
