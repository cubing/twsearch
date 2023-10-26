use cubing::alg::{Alg, Move};
use rand::{seq::SliceRandom, thread_rng};

// Hardcoded to 2 because we only need this for BLD right now.
const NUM_RANDOM_SUFFIX_CHOICES: usize = 2;

// TODO: figure out how to make these actually static
pub(crate) fn static_move_list(move_strings: &[&str]) -> Vec<Move> {
    move_strings
        .iter()
        .map(|s| s.parse::<Move>().unwrap())
        .collect()
}

// An empty move string corresponds to `None`.
// TODO: figure out how to make these actually static
pub(crate) fn static_move_opt_list(move_strings: &[&str]) -> Vec<Option<Move>> {
    move_strings
        .iter()
        .map(|s| match s {
            &"" => None,
            s => Some(s.parse::<Move>().unwrap()),
        })
        .collect()
}

pub(crate) fn add_random_suffixes_from(
    alg: Alg,
    suffixes_from: [Vec<Option<Move>>; NUM_RANDOM_SUFFIX_CHOICES],
) -> Alg {
    let mut rng = thread_rng();
    let mut nodes = alg.nodes;
    for suffix_from in &suffixes_from {
        if let Some(Some(r#move)) = suffix_from.choose(&mut rng) {
            nodes.push(r#move.clone().into())
        }
    }
    Alg { nodes }
}
