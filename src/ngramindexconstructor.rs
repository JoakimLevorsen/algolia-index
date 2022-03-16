use crate::ngramindex::{GramAtom, IndexFeed, NGramIndex};

impl<G: GramAtom, const N: usize, Data> NGramIndex<G, N, Data> {
    pub fn new<I: Iterator<Item = G> + Clone, S: Iterator<Item = IndexFeed<G, I, Data>>>(
        source_iter: S,
    ) -> NGramIndex<G, N, Data> {
        let (grams, data) = NGramIndex::alomst_new(source_iter);
        NGramIndex { grams, data }
    }
}
