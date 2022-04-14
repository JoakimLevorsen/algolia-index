use std::{fmt::Debug, hash::Hash};

use ahash::AHashMap;

use crate::{
    data::ProductContainer,
    serialize::{Deserializable, Serializable},
};

use super::result_ranker::{HashExtractable, ResultRanker};

pub trait GramAtom: Default + Copy + Eq + Hash + Debug + Serializable + Deserializable {}

impl<'a, T> GramAtom for T where
    T: Copy + Default + Eq + Hash + Debug + Serializable + Deserializable
{
}

pub struct IndexFeed<'a, G: GramAtom, GI, Data>
where
    GI: Iterator<Item = G> + Clone,
{
    pub grams: GI,
    pub data: &'a Data,
}

pub fn n_gram_with_1_padding<const N: usize, G: GramAtom, I: Iterator<Item = G>>(
    iter: &mut I,
) -> Option<[G; N]> {
    let mut base = [G::default(); N];
    for item in base.iter_mut().take(N - 1) {
        *item = iter.next()?;
    }
    Some(base)
}

#[derive(Debug, PartialEq)]
pub struct GramNode<'data, G: GramAtom> {
    pub item: G,
    pub weight: f32,
    pub by_occurances: Vec<&'data GramNode<'data, G>>,
    pub items: AHashMap<G, &'data GramNode<'data, G>>,
}

impl<G: GramAtom> Eq for GramNode<'_, G> {}

impl<G: GramAtom> Ord for GramNode<'_, G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight
            .partial_cmp(&other.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl<G: GramAtom> PartialOrd for GramNode<'_, G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq)]
pub struct GramIndex<'a, G: GramAtom, Data: Ord, const N: usize = 8> {
    // Note we don't need popularity since we'll search for all parts of the gram
    pub roots: AHashMap<G, &'a GramNode<'a, G>>,
    pub data: AHashMap<[G; N], Vec<&'a Data>>,
    pub product_container: &'a ProductContainer<'a>,
}

impl<'a, G: GramAtom, Data: Ord + HashExtractable + Debug, const N: usize>
    GramIndex<'a, G, Data, N>
{
    pub fn most_popular_chain(&self, input: G) -> Vec<G> {
        let mut out = vec![];
        let mut node = self.roots.get(&input);
        while let Some(next) = node {
            out.push(next.item);
            node = next.by_occurances.first();
        }
        out
    }

    pub fn search<I: Iterator<Item = G>>(&self, input: I) -> Vec<(&Data, f32)> {
        let mut results = ResultRanker::new();
        let mut ngram = [G::default(); N];
        for gram in input {
            for i in 1..N {
                ngram[i - 1] = ngram[i];
            }
            ngram[N - 1] = gram;
            let (ngram, confidence) = match self.search_gram(ngram) {
                Some(v) => v,
                None => continue,
            };
            let data = match self.data.get(&ngram) {
                Some(v) => v,
                None => continue,
            };
            for data in data {
                results.add(*data, confidence);
            }
        }
        results.export_data_by_confidence()
    }

    /*
    For the current step in the tree:
        The user entered gram
        No Gram
        The 5 most popular grams

    And these are tested on:
        This node
        Last node

    If skip limit is reached, we don't do the previous node or no node
    */

    pub fn search_gram(&self, query: [G; N]) -> Option<([G; N], f32)> {
        let root_node = self.roots.get(query.get(0)?)?;
        let mut previous = [G::default(); N];
        previous[0] = query[0];
        let changes_limit = u8::try_from(N / 3).unwrap();
        Self::recursive_search(
            &query[1..],
            root_node,
            None,
            0,
            changes_limit,
            1.0,
            previous,
            1,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn recursive_search(
        input: &[G],
        node: &GramNode<'_, G>,
        previous_node: Option<&GramNode<'_, G>>,
        changes: u8,
        changes_limit: u8,
        cummulative_weight: f32,
        previous_input: [G; N],
        index: usize,
    ) -> Option<([G; N], f32)>
    where
        G: 'a,
        Data: 'a,
    {
        if index == N {
            return Some((previous_input, cummulative_weight));
        }

        let best_result = |a: Option<([G; N], f32)>, b: Option<([G; N], f32)>| {
            match (a, b) {
                // If only one exists, we use the existing
                (Some(v), None) | (None, Some(v)) => Some(v),
                // If none exist, we don't have any
                (None, None) => None,
                (Some(a), Some(b)) => Some(if a.1 > b.1 { a } else { b }),
            }
        };

        let user_entered = input.get(0).and_then(|gram| node.items.get(gram));
        let user_entered_gram = user_entered.map(|v| v.item);

        let mut most_likely = None;

        let next_input = if input.is_empty() { &[] } else { &input[1..] };

        // We try the current options, on the current node
        if let Some(user_entered) = user_entered {
            let mut previous_input = previous_input;
            previous_input[index] = user_entered.item;

            let found = Self::recursive_search(
                next_input,
                user_entered,
                Some(node),
                changes,
                changes_limit,
                cummulative_weight * user_entered.weight,
                previous_input,
                index + 1,
            );

            // If we found something, we keep the most likely
            most_likely = best_result(found, most_likely);
        }

        // If more skips are allowed, we try those
        if changes_limit > changes {
            let changes = changes + 1;

            // We try the most popular options at this node
            let most_popular = node
                .by_occurances
                .iter()
                // We only want grams that do not match the
                .filter(|potential| Some(potential.item) != user_entered_gram)
                .copied();

            for next_node in most_popular {
                let mut previous_input = previous_input;
                previous_input[index] = next_node.item;

                let found = Self::recursive_search(
                    next_input,
                    next_node,
                    Some(node),
                    changes,
                    changes_limit,
                    cummulative_weight * next_node.weight,
                    previous_input,
                    index + 1,
                );

                // We also try running this with the same input as we got, thereby compensating for forgotten grams
                let repeat_found = Self::recursive_search(
                    input,
                    next_node,
                    Some(node),
                    changes + 1,
                    changes_limit,
                    cummulative_weight * next_node.weight,
                    previous_input,
                    index + 1,
                );

                // If we found something, we keep the most likely
                most_likely = best_result(best_result(found, repeat_found), most_likely);
            }

            // We try ignoring the previous gram, in case the user entered "abc" when they meant "ac"
            if let Some(current_input) = input.get(0) {
                if let Some(Some(last)) = previous_node.map(|node| node.items.get(current_input)) {
                    let mut previous_input = previous_input;
                    // We overwrite the old gram
                    previous_input[index - 1] = last.item;

                    let found = Self::recursive_search(
                        next_input,
                        *last,
                        None,
                        changes,
                        changes_limit,
                        cummulative_weight,
                        previous_input,
                        // We use the same index since we really looked at index - 1
                        index,
                    );

                    most_likely = best_result(found, most_likely);
                }
            }
        }

        most_likely
    }
}

#[cfg(test)]
mod test {
    use crate::data::Product;

    use super::GramIndex;

    fn make_index<'a, const N: usize>(
    ) -> Result<GramIndex<'a, char, Product<'a>, N>, Box<dyn std::error::Error>> {
        use crate::data::{optimize, RawProduct, SuperAlloc};
        use colosseum::sync::Arena;

        let file = std::fs::read_to_string("./test.json")?;

        let products: ahash::AHashMap<String, RawProduct> = serde_json::from_str(&file)?;

        let products: Vec<_> = products.into_iter().map(|(_, v)| v).collect();

        lazy_static::lazy_static! {
            static ref SUPER_ARENA: SuperAlloc = SuperAlloc::new();
        }

        let (prods, _) = optimize(products, &SUPER_ARENA);

        let iter = prods.products.iter().map(|p| {
            let Product {
                description,
                title,
                vendor,
                ..
            } = p;

            super::IndexFeed {
                data: p,
                grams: [description, title, &vendor.name]
                    .into_iter()
                    .flat_map(|s| s.chars())
                    .flat_map(char::to_lowercase),
            }
        });

        let arena = SUPER_ARENA.alloc(Arena::new());

        let index: GramIndex<char, Product, N> = GramIndex::index_from(iter, arena, prods);

        Ok(index)
    }

    #[test]
    fn test_index_generation() -> Result<(), Box<dyn std::error::Error>> {
        let index = make_index::<5>()?;
        std::mem::drop(index);
        Ok(())
    }

    #[test]
    #[ignore = "Only for testing statistics"]
    fn test_gram_popularity() -> Result<(), Box<dyn std::error::Error>> {
        let index = make_index::<5>()?;

        let mut products_for_gram = ahash::AHashMap::new();

        let mut ones_with_space = 0;

        for (gram, products) in index.data {
            if products.len() == 1 && gram.iter().copied().any(char::is_whitespace) {
                ones_with_space += 1;
            }
            products_for_gram
                .entry(products.len())
                .and_modify(|v| *v += 1)
                .or_insert(1u32);
        }

        println!(
            "{} single product n-grams contained atleast one space out of {}",
            ones_with_space,
            products_for_gram.get(&1).copied().unwrap_or(0)
        );

        let ordered_products_for_gram = {
            let mut prodcuts_for_gram: Vec<_> = products_for_gram.into_iter().collect();
            prodcuts_for_gram.sort_by(|a, b| a.0.cmp(&b.0));
            prodcuts_for_gram
        };

        let mut csv = "product_amount;ngrams\n".to_string();
        for (product_amount, ngrams) in ordered_products_for_gram {
            csv += &format!("{product_amount};{ngrams}\n");
        }

        std::fs::write("products_for_ngram.csv", csv)?;

        Ok(())
    }
}
