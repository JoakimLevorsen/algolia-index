use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::serialize::{Deserializable, Serializable};

pub trait GramAtom: Default + Copy + Eq + Hash + Debug + Serializable + Deserializable {}

impl<'a, T> GramAtom for T where
    T: Copy + Default + Eq + Hash + Debug + Serializable + Deserializable
{
}

pub struct IndexFeed<G: GramAtom, GI, Data>
where
    GI: Iterator<Item = G> + Clone,
{
    pub grams: GI,
    pub data: Data,
}

#[derive(Debug, PartialEq)]
pub struct GramNode<'data, G: GramAtom> {
    pub item: G,
    pub weight: f32,
    pub by_occurances: Vec<&'data GramNode<'data, G>>,
    pub items: HashMap<G, &'data GramNode<'data, G>>,
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

pub struct GramIndex<'a, G: GramAtom, Data: Ord, const N: usize = 8> {
    // Note we don't need popularity since we'll search for all parts of the gram
    pub roots: HashMap<G, &'a GramNode<'a, G>>,
    pub data: HashMap<[G; N], Vec<&'a Data>>,
}

impl<'a, G: GramAtom, Data: Ord, const N: usize> GramIndex<'a, G, Data, N> {
    pub fn most_popular_chain(&self, input: G) -> Vec<G> {
        let mut out = vec![];
        let mut node = self.roots.get(&input);
        while let Some(next) = node {
            out.push(next.item);
            node = next.by_occurances.first();
        }
        out
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

    pub fn search(&self, query: [G; N]) -> Option<([G; N], f32)> {
        let root_node = self.roots.get(query.get(0)?)?;
        let mut previos = [G::default(); N];
        previos[0] = query[0];
        Self::recursive_search(&query[1..], root_node, None, 0, 2, 0.0, previos, 1)
    }

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

        let best_result = |a: Option<([G; N], f32)>, b| {
            match (a, b) {
                // If only one exists, we use the existing
                (Some(v), None) | (None, Some(v)) => Some(v),
                // If none exist, we don't have any
                (None, None) => None,
                (Some(a), Some(b)) => Some(if a.1 > b.1 { a } else { b }),
            }
        };

        let user_entered = input.get(0).map(|gram| node.items.get(gram)).flatten();
        let user_entered_gram = user_entered.clone().map(|v| v.item);

        let mut most_likely = None;

        let next_input = if input.len() == 0 { &[] } else { &input[1..] };

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
                cummulative_weight + user_entered.weight,
                previous_input,
                index + 1,
            );

            // If we found something, we keep the most likely
            most_likely = best_result(found, most_likely)
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
                .map(|v| *v);

            for next_node in most_popular {
                let mut previous_input = previous_input;
                previous_input[index] = next_node.item;

                let found = Self::recursive_search(
                    next_input,
                    next_node,
                    Some(node),
                    changes,
                    changes_limit,
                    cummulative_weight + next_node.weight,
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
                    cummulative_weight + next_node.weight,
                    previous_input,
                    index + 1,
                );

                // If we found something, we keep the most likely
                most_likely = best_result(best_result(found, repeat_found), most_likely)
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

                    most_likely = best_result(found, most_likely)
                }
            }
        }

        most_likely
    }
}

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    use crate::Product;
    use typed_arena::Arena;

    let file = std::fs::read_to_string("./test.json")?;

    let products: HashMap<String, Product> = serde_json::from_str(&file)?;

    let iter = products.values().map(
        |Product {
             description,
             tags,
             title,
             vendor,
             id,
         }| IndexFeed {
            data: id.clone(),
            grams: [description, title, vendor]
                .into_iter()
                .chain(tags.into_iter())
                .flat_map(|s| s.chars())
                .flat_map(|c| c.to_lowercase()),
        },
    );

    let mut arena = Arena::new();
    let mut data_arena = Arena::new();

    let index: GramIndex<char, String, 8> =
        GramIndex::index_from(iter, &mut arena, &mut data_arena);

    Ok(())
}
