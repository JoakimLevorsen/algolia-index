use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use typed_arena::Arena;

use crate::ngramindex::{GramAtom, IndexFeed};

#[derive(Debug, PartialEq, Eq)]
pub struct GramNode<'data, G: GramAtom> {
    item: G,
    occurances: u32,
    by_occurances: Vec<&'data GramNode<'data, G>>,
    items: HashMap<G, &'data GramNode<'data, G>>,
}

impl<G: GramAtom> Ord for GramNode<'_, G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.occurances.cmp(&other.occurances)
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
        arena: &'arena Arena<GramNode<'arena, G>>,
    ) -> &'arena GramNode<'arena, G> {
        let me = self.0.borrow();

        let children = me.items.len();

        let mut by_occurances: Vec<&'arena GramNode<'arena, G>> =
            me.items.iter().map(|(_, v)| v.immutalize(arena)).collect();

        by_occurances.sort_by(|a, b| a.cmp(&b).reverse());

        let items = by_occurances
            .iter()
            .fold(HashMap::with_capacity(children), |mut map, item| {
                map.insert(item.item, *item);
                map
            });

        arena.alloc(GramNode {
            item: me.item,
            occurances: me.occurances,
            by_occurances,
            items,
        })
    }
}

pub struct GramIndex<'a, G: GramAtom, Data: Ord, const N: usize = 8> {
    roots: HashMap<G, &'a GramNode<'a, G>>,
    data: HashMap<[G; N], Vec<&'a Data>>,
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

        // We then transform the mutable tree into an immutable tree with no access control
        let roots: HashMap<G, &'arena GramNode<'arena, G>> = root
            .into_iter()
            .map(|(k, v)| (k, v.immutalize(node_arena)))
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
}
