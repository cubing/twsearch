use twsearch::{CommandError, _internal::cli::BenchmarkArgs};

pub fn benchmark(benchmark_args: &BenchmarkArgs) -> Result<(), CommandError> {
    dbg!(benchmark_args);
    Ok(())
}
