use twsearch::_internal::cli::args::BenchmarkArgs;

use crate::{
    rewrite::rewrite_def_file, rust_api::rust_api_main_search, wrapper_options::reset_args_from,
};

pub fn benchmark(benchmark_args: BenchmarkArgs) -> Result<(), String> {
    reset_args_from(vec![&benchmark_args]);
    let (def_file, _temp1) = rewrite_def_file(&benchmark_args.def_args, &None)?;
    rust_api_main_search(&def_file, "");
    Ok(())
}
