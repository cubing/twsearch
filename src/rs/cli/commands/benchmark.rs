use cubing::{
    alg::Move,
    kpuzzle::{KPuzzle, KPuzzleDefinition},
};
use instant::Instant;
use rand::seq::SliceRandom;
use twsearch::{
    CommandError, PackedKPatternBuffer, PackedKPuzzle, PackedKTransformation, SearchMoveCache,
    _internal::cli::BenchmarkArgs,
};

use crate::io::read_to_json;

const NUM_RANDOM_MOVES: usize = 65536;
const NUM_TEST_TRANSFORMATIONS: usize = 100_000_000;

pub fn benchmark(benchmark_args: &BenchmarkArgs) -> Result<(), CommandError> {
    let def: KPuzzleDefinition =
        read_to_json(&benchmark_args.input_args.def_file).expect("Invalid definition"); // TODO: automatic error conversion.
    let kpuzzle = KPuzzle::try_new(def).expect("Invalid definition"); // TODO: automatic error conversion.
    let packed_kpuzzle = PackedKPuzzle::try_from(kpuzzle).expect("Invalid definition"); // TODO: automatic error conversion.

    let move_list: Vec<Move> = benchmark_args.moves_args.moves_parsed().unwrap_or_else(|| {
        packed_kpuzzle
            .data
            .kpuzzle
            .definition()
            .moves
            .keys()
            .cloned()
            .collect()
    });
    let search_move_cache = SearchMoveCache::try_new(
        &packed_kpuzzle,
        &move_list,
        &benchmark_args.metric_args.metric,
    )
    .expect("Could not get search move cache"); // TODO: automatic error conversion.

    let mut rng = rand::thread_rng();
    let random_move_list: Vec<&PackedKTransformation> = (0..NUM_RANDOM_MOVES)
        .map(|_| {
            &search_move_cache
                .flat
                .choose(&mut rng)
                .unwrap()
                .transformation
        })
        .collect();

    let mut pattern_buffer = PackedKPatternBuffer::from(packed_kpuzzle.default_pattern());
    for _ in 0..3 {
        let start_time = Instant::now();
        for i in 0..NUM_TEST_TRANSFORMATIONS {
            pattern_buffer.apply_transformation(random_move_list[i % NUM_RANDOM_MOVES]);
        }
        let end_time = Instant::now();
        let elapsed = end_time - start_time;
        let rate = std::convert::TryInto::<f64>::try_into(NUM_TEST_TRANSFORMATIONS as u32).unwrap()
            / elapsed.as_secs_f64()
            / std::convert::TryInto::<f64>::try_into(1_000_000).unwrap();
        println!(
            "Took {:?} for {} transformations ({:.2}M moves/s)",
            elapsed, NUM_TEST_TRANSFORMATIONS, rate
        );
    }

    Ok(())
}
