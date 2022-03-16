use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::ngramindex::{GramAtom, IndexFeed};

pub fn preprocessor<G: GramAtom, const N: usize, I, S, Data>(source: S, cutoff_percent: f64) -> S
where
    I: Iterator<Item = G> + Clone,
    S: Iterator<Item = IndexFeed<G, I, Data>> + Clone,
{
    let data: Vec<(Data, Vec<G>)> = source
        .map(|source| (source.data, source.grams.collect()))
        .collect();
    let mut total_seen = HashMap::new();

    for (_, grams) in data.iter() {
        let occurances = occurances_for_source::<_, N>(grams);
        for val in occurances {
            total_seen
                .entry(val)
                .and_modify(|v| *v += 1)
                .or_insert(1usize);
        }
    }

    let len = data.len() as f64;

    let mut with_percent: Vec<_> = total_seen
        .into_iter()
        .filter_map(|(gram, v)| {
            let seen_in_percent = (v as f64) / len;
            if seen_in_percent >= cutoff_percent {
                Some((gram, seen_in_percent))
            } else {
                None
            }
        })
        .collect();

    with_percent.sort_by(|(_, a), (_, b)| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal));

    for (gram, percent) in with_percent {
        println!("Saw: {gram:?} in {percent:.04}")
    }

    todo!()
}

fn occurances_for_source<'grams, G: GramAtom, const N: usize>(
    grams: &'grams [G],
) -> HashSet<&'grams [G]> {
    let mut seen = HashSet::new();

    for i in 0..(grams.len() - N) {
        let n_slice = &grams[i..(i + N)];
        for j in 2..N {
            seen.insert(&n_slice[0..j]);
        }
    }

    seen
}
