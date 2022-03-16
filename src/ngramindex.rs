use std::{collections::HashMap, fmt::Debug, hash::Hash};

pub trait GramAtom: Default + Copy + Eq + Hash + Debug {}

impl<T> GramAtom for T where T: Copy + Default + Eq + Hash + Debug {}

pub struct IndexFeed<G: GramAtom, GI, Data>
where
    GI: Iterator<Item = G> + Clone,
{
    pub grams: GI,
    pub data: Data,
}

struct GramOccurances<G: GramAtom> {
    gram: G,
    occurances: usize,
}

pub struct GramData<G: GramAtom, const N: usize> {
    followed_by: [Vec<GramOccurances<G>>; N],
    occurances: usize,
}

pub struct NGramIndex<G: GramAtom, const N: usize, Data> {
    pub grams: HashMap<G, GramData<G, N>>,
    pub data: Vec<Data>,
}

impl<G: GramAtom, const N: usize, Data> NGramIndex<G, N, Data> {
    pub fn alomst_new<I, S>(source_iter: S) -> (HashMap<G, GramData<G, N>>, Vec<Data>)
    where
        I: Iterator<Item = G> + Clone,
        S: Iterator<Item = IndexFeed<G, I, Data>>,
    {
        struct HashGramData<G: GramAtom, const N: usize> {
            followed_by: [HashMap<G, usize>; N],
            occurances: usize,
        }

        impl<G: GramAtom, const N: usize> HashGramData<G, N> {
            pub fn new() -> HashGramData<G, N> {
                let mut followed_by: [HashMap<G, usize>; N] = { [(); N].map(|_| HashMap::new()) };
                for i in 0..N {
                    followed_by[i] = HashMap::new();
                }
                HashGramData {
                    followed_by,
                    occurances: 0,
                }
            }
        }

        let mut grams: HashMap<G, HashGramData<G, N>> = HashMap::new();

        'source_loop: for source in source_iter {
            let mut grams_iter = source.grams;
            let mut target = match grams_iter.next() {
                Some(v) => v,
                None => continue 'source_loop,
            };
            let mut followed_by = [G::default(); N];
            // We fill the lookback
            for i in 0..N {
                followed_by[i] = match grams_iter.next() {
                    Some(v) => v,
                    None => continue 'source_loop,
                };
            }

            'insert_loop: loop {
                // We register that target is followed by followed_by[0] and then followed_by[1] and so forth
                let stats = grams.entry(target).or_insert_with(HashGramData::new);

                stats.occurances += 1;

                for i in 0..N {
                    stats.followed_by[i]
                        .entry(followed_by[i])
                        .and_modify(|v| *v += 1)
                        .or_insert(1);
                }

                // We get the next atom, and move all elements by 1
                let next = match grams_iter.next() {
                    Some(v) => v,
                    None => break 'insert_loop,
                };
                target = followed_by[0];
                for i in 1..N {
                    followed_by[i - 1] = followed_by[i]
                }
                followed_by[N - 1] = next;
            }

            // We add the trailing grams
            for (i, v) in followed_by.iter().enumerate() {
                let stats = grams.entry(*v).or_insert_with(HashGramData::new);

                stats.occurances += 1;

                for j in (i + 1)..N {
                    stats.followed_by[j]
                        .entry(followed_by[j])
                        .and_modify(|v| *v += 1)
                        .or_insert(1);
                }
            }
        }

        // We then convert the HashGramData, into vecs instead
        let grams: HashMap<G, GramData<G, N>> = grams
            .into_iter()
            .map(
                |(
                    k,
                    HashGramData {
                        followed_by,
                        occurances,
                    },
                )| {
                    (
                        k,
                        GramData {
                            followed_by: followed_by.map(|v| {
                                let mut all: Vec<_> = v
                                    .into_iter()
                                    .map(|(gram, occurances)| GramOccurances { gram, occurances })
                                    .collect();
                                all.sort_by(|a, b| a.occurances.cmp(&b.occurances).reverse());
                                all
                            }),
                            occurances,
                        },
                    )
                },
            )
            .collect();

        (grams, Vec::new())
    }

    pub fn search<I: Iterator<Item = G>>(&self, input: I) -> Vec<[Option<(G, G)>; N]> {
        let mut lookback = [None; N];
        let mut out = Vec::with_capacity(input.size_hint().1.unwrap_or(0));
        for char in input {
            for i in 1..N {
                lookback[i - 1] = lookback[i];
            }
            lookback[N - 1] = Some(char);

            // We then get the hints for every lookback char
            let mut frame = [None; N];
            for i in 0..N {
                let this = match lookback[i] {
                    Some(v) => v,
                    None => break,
                };

                {
                    let mut chain = this;
                    let mut row = Vec::with_capacity(N);
                    let mut j = 0;
                    while let Some(Some(Some(next))) = self
                        .grams
                        .get(&chain)
                        .map(|data| data.followed_by.get(0).map(|v| v.first()))
                    {
                        chain = next.gram;
                        row.push(chain);
                        j += 1;
                        if j == 8 {
                            break;
                        }
                    }

                    println!("Chain:{i} {this:?} {chain:?} {row:?}");
                }

                let following = match self.grams.get(&this) {
                    Some(v) => v,
                    None => break,
                };

                frame[i] = following.followed_by[i]
                    .first()
                    .map(|found| (this, found.gram))
            }
            out.push(frame);
        }
        out
    }
}
