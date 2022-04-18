use colosseum::sync::Arena;

use crate::{
    classic_indexes::ClassicIndexes,
    data::{Product, SuperAlloc},
    ngram::{GramAtom, GramIndex, GramNode},
};

use super::Serializable;

pub fn serialize_all<G: GramAtom, const N: usize>(
    ngram: &GramIndex<'_, G, Product<'_>, N>,
    classic: &ClassicIndexes<'_>,
) -> Vec<u8> {
    let mut out = Vec::new();
    let mut save = |input: u8| out.push(input);
    ngram.serialize(&mut save);
    classic.serialize(&mut save);
    out
}

pub fn deserialize_all<'arena, G: GramAtom, const N: usize>(
    input: &[u8],
    node_arena: &'arena Arena<GramNode<'arena, G>>,
    super_alloc: &'static SuperAlloc,
) -> Option<(
    GramIndex<'arena, G, Product<'arena>, N>,
    ClassicIndexes<'arena>,
)> {
    let (input, ngram) = GramIndex::deserialize(input, node_arena, super_alloc)?;
    let (_, classic) = ClassicIndexes::deserialize(input, ngram.product_container)?;

    Some((ngram, classic))
}
