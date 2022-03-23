use std::collections::HashMap;

use colosseum::sync::Arena;

use crate::{
    ngram::{GramAtom, GramIndex, GramNode},
    Product,
};

use super::{
    collections::manual_hashmap_deserialize, ArenaDeserializable, ArenaDeserializableCollection,
    Deserializable, Serializable,
};

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

impl<G: GramAtom, Data: Ord + Serializable, const N: usize> Serializable
    for GramIndex<'_, G, Data, N>
{
    fn serialize(&self, output: &mut Vec<u8>) {
        self.roots.serialize(output);
        self.data.serialize(output);
    }
}

// impl<G: GramAtom, Data: Ord + Serializable, const N: usize> GramIndex<'_, G, Data, N> {}

impl<'arena, G: GramAtom, Data, const N: usize> GramIndex<'arena, G, Data, N>
where
    Data: Ord + ArenaDeserializable<'arena, Data>,
{
    pub fn deserialize<'input>(
        input: &'input [u8],
        node_arena: &'arena Arena<GramNode<'arena, G>>,
        data_arena: &'arena Arena<Data>,
    ) -> Option<Self> {
        let (input, roots) = HashMap::deserialize_arena(input, node_arena)?;
        let data: HashMap<[G; N], Vec<&'arena Data>> =
            manual_hashmap_deserialize(input, data_arena)?;
        Some(GramIndex { roots, data })
    }
}
