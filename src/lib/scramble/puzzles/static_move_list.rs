use std::str::FromStr;

use cubing::alg::{Alg, Move};
use rand::{rng, seq::IndexedRandom};

// Hardcoded to 2 because we only need this for BLD right now.
const NUM_RANDOM_SUFFIX_CHOICES: usize = 2;

// An empty input string corresponds to `None`.
// TODO: figure out how to make these actually static
pub(crate) fn static_parsed_opt_list<T: FromStr>(strings: &[&str]) -> Vec<Option<T>>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    strings
        .iter()
        .map(|s| match s {
            &"" => None,
            s => Some(s.parse::<T>().unwrap()),
        })
        .collect()
}

pub(crate) fn add_random_suffixes_from(
    alg: Alg,
    suffixes_from: &[Vec<Option<Move>>; NUM_RANDOM_SUFFIX_CHOICES],
) -> Alg {
    let mut rng = rng();
    let mut nodes = alg.nodes;
    for suffix_from in suffixes_from {
        if let Some(Some(r#move)) = suffix_from.choose(&mut rng) {
            nodes.push(r#move.clone().into())
        }
    }
    Alg { nodes }
}
