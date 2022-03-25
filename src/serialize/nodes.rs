use std::collections::HashMap;

use colosseum::sync::Arena;

use crate::{
    data::{ProductContainer, SuperAlloc},
    ngram::{GramAtom, GramIndex, GramNode},
    Product,
};

use super::{ArenaDeserializable, ArenaDeserializableCollection, Deserializable, Serializable};

impl<G: GramAtom> Serializable for GramNode<'_, G> {
    fn serialize(&self, output: &mut Vec<u8>) {
        Serializable::serialize(&self.item, output);
        self.weight.serialize(output);
        self.by_occurances.serialize(output);
    }
}

impl<G: GramAtom> Serializable for &GramNode<'_, G> {
    fn serialize(&self, output: &mut Vec<u8>) {
        Serializable::serialize(&self.item, output);
        self.weight.serialize(output);
        self.by_occurances.serialize(output);
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
        let items = by_occurances.iter().fold(HashMap::new(), |mut map, item| {
            map.insert(item.item, *item);
            map
        });

        let node: &'arena GramNode<'arena, G> = arena.alloc(GramNode {
            item,
            weight,
            items,
            by_occurances,
        });
        Some((input, node))
    }
}

impl<G: GramAtom, const N: usize> Serializable for GramIndex<'_, G, Product<'_>, N> {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.product_container.serialize(output);
        self.roots.serialize(output);
        // We replace the product refs with their serialization id
        let data: HashMap<[G; N], Vec<usize>> = self
            .data
            .iter()
            .map(|(k, v)| (*k, v.iter().map(|p| p.serialization_id).collect()))
            .collect();
        data.serialize(output);
    }
}

// impl<G: GramAtom, Data: Ord + Serializable, const N: usize> GramIndex<'_, G, Data, N> {}

impl<'arena, G: GramAtom, const N: usize> GramIndex<'arena, G, Product<'arena>, N> {
    pub fn deserialize<'input, 'super_arena>(
        input: &'input [u8],
        node_arena: &'arena Arena<GramNode<'arena, G>>,
        super_alloc: &'static SuperAlloc,
    ) -> Option<Self>
    where
        'super_arena: 'arena,
    {
        let (input, container) = ProductContainer::deserialize(input, super_alloc)?;
        let container = super_alloc.alloc(container);
        let (input, roots) = HashMap::deserialize_arena(input, node_arena)?;
        // We make the ref array
        let (_, bin_data) = HashMap::<[G; N], Vec<usize>>::deserialize(input)?;

        // We then transform this data based on the data container
        let mut data = HashMap::with_capacity(bin_data.len());
        for (k, v) in bin_data.into_iter() {
            let mut products = Vec::with_capacity(v.len());
            for v in v {
                products.push(container.products.get(v)?)
            }
            data.insert(k, products);
        }
        Some(GramIndex {
            product_container: container,
            roots,
            data,
        })
    }
}
