use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use ahash::AHashMap;
use colosseum::sync::Arena;

use crate::data::ProductContainer;

use super::{GramAtom, GramIndex, GramNode, IndexFeed};

#[derive(Debug, Clone)]
struct InnerMutableGramNode<G: GramAtom> {
    item: G,
    occurances: u32,
    items: AHashMap<G, MutableGramNode<G>>,
}

#[allow(clippy::cast_precision_loss)]
fn occurances_to_weight(total: u32, this: u32) -> f32 {
    (this as f32) / (total as f32)
}

const DATA_CUTOFF_PERCENTAGE: f32 = 0.8;

#[derive(Clone, Debug)]
struct MutableGramNode<G: GramAtom>(Rc<RefCell<InnerMutableGramNode<G>>>);

impl<G: GramAtom> MutableGramNode<G> {
    pub fn new(item: G, occurances: u32) -> Self {
        MutableGramNode(Rc::new(RefCell::new(InnerMutableGramNode {
            item,
            occurances,
            items: AHashMap::new(),
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

        by_occurances.sort_by(|a, b| a.cmp(b).reverse());

        let items =
            by_occurances
                .iter()
                .fold(AHashMap::with_capacity(children), |mut map, item| {
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

impl<'a, G: GramAtom, Data: Ord, const N: usize> GramIndex<'a, G, Data, N> {
    pub fn index_from<'arena, I, S>(
        source_iter: S,
        node_arena: &'arena Arena<GramNode<'arena, G>>,
        product_container: &'arena ProductContainer<'arena>,
    ) -> GramIndex<'arena, G, Data, N>
    where
        I: Iterator<Item = G> + Clone,
        S: Iterator<Item = IndexFeed<'arena, G, I, Data>>,
    {
        let mut root: AHashMap<G, MutableGramNode<G>> = AHashMap::new();
        let mut data_map: AHashMap<[G; N], Vec<&'arena Data>> = AHashMap::new();
        for IndexFeed { grams, data } in source_iter {
            let mut queue: VecDeque<MutableGramNode<G>> = VecDeque::with_capacity(N + 1);

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
                for gram_node in &mut queue {
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
                    lookback[i - 1] = lookback[i];
                }
                lookback[N - 1] = gram;

                // If we have the required grams, we add a data reference
                data_map
                    .entry(lookback)
                    .and_modify(|vec| vec.push(data))
                    .or_insert_with(|| vec![data]);
            }
        }

        let total_root_occurances = root
            .values()
            .map(|v| v.0.borrow().occurances)
            .reduce(|a, b| a + b)
            .unwrap_or(0);

        // We then transform the mutable tree into an immutable tree with no access control
        let roots: AHashMap<G, &'arena GramNode<'arena, G>> = root
            .into_iter()
            .map(|(k, v)| (k, v.immutalize(total_root_occurances, node_arena)))
            .collect();

        // We remove the data if it contains more than DATA_CUTOFF percent of products, since it doesn't carry enough information
        let products_amount = product_container.products.len();
        #[allow(clippy::cast_precision_loss)]
        let maximum_product_amount = (products_amount as f32) * DATA_CUTOFF_PERCENTAGE;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let maximum_product_amount = maximum_product_amount as usize;
        data_map.retain(|_, data| data.len() < maximum_product_amount);

        // We also sort the data for easier access later
        for (_, data) in data_map.iter_mut() {
            data.sort();
        }

        GramIndex {
            roots,
            data: data_map,
            product_container,
        }
    }
}
