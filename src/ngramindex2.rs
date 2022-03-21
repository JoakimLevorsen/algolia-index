use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use typed_arena::Arena;

use crate::ngramindex::{GramAtom, IndexFeed};

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

#[derive(Debug, Clone)]
struct InnerMutableGramNode<G: GramAtom> {
    item: G,
    occurances: u32,
    items: HashMap<G, MutableGramNode<G>>,
}

fn occurances_to_weight(total: u32, this: u32) -> f32 {
    (total as f32) / (this as f32)
}

#[derive(Clone)]
struct MutableGramNode<G: GramAtom>(Rc<RefCell<InnerMutableGramNode<G>>>);

impl<G: GramAtom> std::fmt::Debug for MutableGramNode<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MutableGramNode")
            .field(&self.0.borrow())
            .finish()
    }
}

impl<G: GramAtom> MutableGramNode<G> {
    pub fn new(item: G, occurances: u32) -> Self {
        MutableGramNode(Rc::new(RefCell::new(InnerMutableGramNode {
            item,
            occurances,
            items: HashMap::new(),
        })))
    }

    pub fn immutalize<'arena>(
        &self,
        parent_occurances: u32,
        arena: &'arena Arena<GramNode<'arena, G>>,
    ) -> &'arena GramNode<'arena, G> {
        let me = self.0.borrow();

        let children = me.items.len();

        let mut by_occurances: Vec<&'arena GramNode<'arena, G>> = me
            .items
            .iter()
            .map(|(_, v)| v.immutalize(me.occurances, arena))
            .collect();

        by_occurances.sort_by(|a, b| a.cmp(&b).reverse());

        let items = by_occurances
            .iter()
            .fold(HashMap::with_capacity(children), |mut map, item| {
                map.insert(item.item, *item);
                map
            });

        arena.alloc(GramNode {
            item: me.item,
            weight: occurances_to_weight(parent_occurances, me.occurances),
            by_occurances,
            items,
        })
    }
}

pub struct GramIndex<'a, G: GramAtom, Data: Ord, const N: usize = 8> {
    // Note we don't need popularity since we'll search for all parts of the gram
    pub roots: HashMap<G, &'a GramNode<'a, G>>,
    pub data: HashMap<[G; N], Vec<&'a Data>>,
}

impl<'a, G: GramAtom, Data: Ord, const N: usize> GramIndex<'a, G, Data, N> {
    pub fn new<'arena, I, S>(
        source_iter: S,
        node_arena: &'arena Arena<GramNode<'arena, G>>,
        data_arena: &'arena Arena<Data>,
    ) -> GramIndex<'arena, G, Data, N>
    where
        I: Iterator<Item = G> + Clone,
        S: Iterator<Item = IndexFeed<G, I, Data>>,
    {
        let mut root: HashMap<G, MutableGramNode<G>> = HashMap::new();
        let mut data_map: HashMap<[G; N], Vec<&'arena Data>> = HashMap::new();
        for IndexFeed { grams, data } in source_iter {
            let mut queue: VecDeque<MutableGramNode<G>> = VecDeque::with_capacity(N + 1);
            let data_ref = &*data_arena.alloc(data);

            let mut lookback = [G::default(); N];

            for gram in grams {
                // We increment occurances in root
                let root_node = root
                    .entry(gram)
                    .and_modify(|node| {
                        node.0.borrow_mut().occurances += 1;
                    })
                    .or_insert_with(|| MutableGramNode::new(gram, 1))
                    .clone();

                // We add our node to the queued list of previous nodes, and move them on
                for gram_node in queue.iter_mut() {
                    let child_node = gram_node
                        .0
                        .borrow_mut()
                        .items
                        .entry(gram)
                        .and_modify(|node| {
                            node.0.borrow_mut().occurances += 1;
                        })
                        .or_insert_with(|| MutableGramNode::new(gram, 1))
                        .clone();

                    // We now have the expected child node of the previous one. We'll use this in the next gram loop so we change the queue
                    *gram_node = child_node;
                }

                // And then we remove the oldest from that list, and add our root.
                queue.push_front(root_node);
                if queue.len() == N {
                    queue.pop_back();
                }

                // We also add the lookback
                for i in 1..N {
                    lookback[i - 1] = lookback[i]
                }
                lookback[N - 1] = gram;

                // If we have the required grams, we add a data reference
                data_map
                    .entry(lookback)
                    .and_modify(|vec| vec.push(data_ref))
                    .or_insert(vec![data_ref]);
            }
        }

        let total_root_occurances = root
            .values()
            .map(|v| v.0.borrow().occurances)
            .reduce(|a, b| a + b)
            .unwrap_or(0);

        // We then transform the mutable tree into an immutable tree with no access control
        let roots: HashMap<G, &'arena GramNode<'arena, G>> = root
            .into_iter()
            .map(|(k, v)| (k, v.immutalize(total_root_occurances, node_arena)))
            .collect();

        // We also sort the data for easier access later
        for (_, data) in data_map.iter_mut() {
            data.sort()
        }

        GramIndex {
            roots,
            data: data_map,
        }
    }

    pub fn most_popular_chain(&self, input: G) -> Vec<G> {
        let mut out = vec![];
        let mut node = self.roots.get(&input);
        while let Some(next) = node {
            out.push(next.item);
            node = next.by_occurances.first();
        }
        out
    }

    pub fn most_likely(&self, input: [G; N]) -> Option<()> {
        todo!()
    }

    fn gullible_search(&self, input: [G; N]) -> Option<&Vec<&Data>> {
        let mut grams = input.into_iter();
        let mut node = *self.roots.get(&grams.next()?)?;
        for gram in grams {
            node = *node.items.get(&gram)?;
        }

        // If we get to here the entire input exists in our tree
        self.data.get(&input)
    }

    const SKIP_THREASHOLD: usize = if usize::MIN + 3 > N { 1 } else { N - 3 };

    /*
    For any position in 0..N we test:
        The actual gram provided
        The 5 most popular grams from this node
        The 5 most popular grams from the previous node
        Skipping this node for the next

        And see what chain we get that has a high weight, and likeness to the original
    */

    fn run_likely(
        &self,
        searched: [G; N],
        i: usize,
        input: &[G],
        node: &GramNode<'_, G>,
        previous: Option<&GramNode<'_, G>>,
    ) -> Option<([G; N], f32)> {
        let expected_gram = match input.get(0) {
            Some(v) => *v,
            // Return the correct
            None => todo!(),
        };
        let current_expected = node.items.get(&expected_gram);
        let next_expected = input.get(1).map(|v| node.items.get(v)).flatten();
        let current_popular = node
            .by_occurances
            .iter()
            .filter(|v| v.item != expected_gram)
            .take(5);

        let blank_arr = Vec::new();

        let last_node_occurances = previous.map(|v| &v.by_occurances).unwrap_or(&blank_arr);
        let last_node_popular = last_node_occurances
            .iter()
            .filter(|v| v.item != expected_gram)
            .take(5);

        // We then try completing the chain with all those options to see the most likely
        let to_check = vec![current_expected, next_expected];
        let all = to_check
            .into_iter()
            .filter_map(|v| v)
            .chain(current_popular)
            .chain(last_node_popular);

        // We take all the fun lil nodes, and complete their chains
        let my_node = Some(node);
        let completed = all.filter_map(|node| {
            let mut new_searched = searched;
            new_searched[i] = node.item;
            self.run_likely(searched, i + 1, &input[1..], node, my_node)
        });

        // let my_input = *input.get(0)?;
        // let expected_following = *input.get(1)?;
        // let user_provided = node.items.get(&my_input);
        // let likely_replacements = node
        //     .by_occurances
        //     .iter()
        //     .filter(|node_in_chain| node_in_chain.item != my_input)
        //     .take(5)
        //     .chain(user_provided);

        // for node in likely_replacements {}

        todo!()
    }

    // fn internal_likely<'b>(
    //     &'b self,
    //     input: &'b [G],
    //     previous: PreviousSlice<'b>,
    // ) -> Option<((), f64)> {
    //     // If our slice is longer than 3, we also do a deeper examiniation
    //     let deep_option = if input.len() > 3 {
    //         let split = input.split_at(1);
    //         let previous = PreviousSlice(Some(&previous), split.1);
    //         self.internal_likely(split.0, previous)
    //     } else {
    //         None
    //     };

    //     todo!()
    // }

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
        let user_entered = input.get(0).map(|gram| node.items.get(gram)).flatten();
        let user_entered_gram = user_entered.clone().map(|v| v.item);

        let mut most_likely = None;

        // We try the current options, on the current node
        if let Some(user_entered) = user_entered {
            let mut previous_input = previous_input;
            previous_input[index] = user_entered.item;

            let found = Self::recursive_search(
                &input[1..],
                user_entered,
                Some(node),
                changes,
                changes_limit,
                cummulative_weight + user_entered.weight,
                previous_input,
                index + 1,
            );

            // If we found something, we keep the most likely
            most_likely = match (found, most_likely) {
                // If only one exists, we use the existing
                (Some(v), None) | (None, Some(v)) => Some(v),
                // If none exist, we don't have any
                (None, None) => None,
                (Some(a), Some(b)) => Some(if a.1 > b.1 { a } else { b }),
            }
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
                    &input[1..],
                    next_node,
                    Some(node),
                    changes,
                    changes_limit,
                    cummulative_weight + next_node.weight,
                    previous_input,
                    index + 1,
                );

                // If we found something, we keep the most likely
                most_likely = match (found, most_likely) {
                    // If only one exists, we use the existing
                    (Some(v), None) | (None, Some(v)) => Some(v),
                    // If none exist, we don't have any
                    (None, None) => None,
                    (Some(a), Some(b)) => Some(if a.1 > b.1 { a } else { b }),
                }
            }

            // First we try no-gram, meaning we just jump to the next user input
            // let no_gram = input.get(1).map(|gram| node.items.get(gram)).flatten();

            // We also try searching for the current gram, and the 5 most likely on the last node
        }

        most_likely
    }
}

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    use super::Product;

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

    let index: GramIndex<char, String, 8> = GramIndex::new(iter, &mut arena, &mut data_arena);

    if let Some(result) = index.gullible_search(['k', 'u', 'n', 's', 't', 'p', 'l', 'a']) {
        println!("Found {} items", result.len());
    }

    Ok(())
}
