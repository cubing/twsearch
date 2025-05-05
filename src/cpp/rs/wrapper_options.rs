use twsearch::_internal::cli::args::{
    BenchmarkArgs, CanonicalAlgsArgs, CommonSearchArgs, EnableAutoAlwaysNeverValueEnum,
    GeneratorArgs, Generators, GodsAlgorithmArgs, MemoryArgs, MetricArgs, MetricEnum,
    PerformanceArgs, RequiredDefArgs, SchreierSimsArgs, ScrambleAndTargetPatternOptionalArgs,
    SearchCommandArgs, SearchCommandOptionalArgs, SearchPersistenceArgs,
    ServeArgsForIndividualSearch, ServeClientArgs, ServeCommandArgs, TimingTestArgs,
};

use std::{fmt::Display, process::exit};

use cubing::alg::{Alg, Move};

use crate::rust_api;

fn is_enabled_with_default_true(v: &Option<EnableAutoAlwaysNeverValueEnum>) -> bool {
    v.as_ref()
        .unwrap_or(&EnableAutoAlwaysNeverValueEnum::Auto)
        .enabled(|| true)
}

fn set_arg<T: Display>(arg_flag: &str, arg: &T) {
    rust_api::rust_api_set_arg(&format!("{} {}", arg_flag, arg));
}

fn set_boolean_arg(arg_flag: &str, arg: bool) {
    if arg {
        rust_api::rust_api_set_arg(arg_flag);
    }
}

fn set_optional_arg<T: Display>(arg_flag: &str, arg: &Option<T>) {
    if let Some(v) = arg {
        rust_api::rust_api_set_arg(&format!("{} {}", arg_flag, v));
    }
}

fn set_moves_arg(moves: &[Move]) {
    // TODO: Squishing together moves into a comma-separated string
    // isn't semantically fantastic. But the moves already passed
    // validation, so this is not as risky as if we were passing strings directly from the client.
    set_arg(
        "--moves",
        &moves
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<String>>()
            .join(","),
    );
}

pub fn reset_args_from(arg_structs: Vec<&dyn SetCppArgs>) {
    rust_api::rust_api_reset();
    for arg_struct in arg_structs {
        arg_struct.set_cpp_args();
    }
}

pub trait SetCppArgs {
    fn set_cpp_args(&self);
}

impl SetCppArgs for CommonSearchArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg(
            "--checkbeforesolve",
            is_enabled_with_default_true(&self.check_before_solve),
        );
        set_boolean_arg("--randomstart", self.random_start);
        set_boolean_arg("--alloptimal", self.all_optimal);
        set_optional_arg("--mindepth", &self.min_depth.map(|o| o.0));
        set_optional_arg("-m", &self.max_depth.map(|o| o.0));
        set_optional_arg("--startprunedepth", &self.start_prune_depth.map(|o| o.0));
        if self.continue_after.is_some() {
            panic!("Continuation is unsupported.")
        }
        if self.continue_at.is_some() {
            panic!("Continuation is unsupported.")
        }
        self.performance_args.set_cpp_args();
    }
}

impl SetCppArgs for SearchCommandArgs {
    fn set_cpp_args(&self) {
        self.optional.set_cpp_args();
        self.def_args.set_cpp_args();
    }
}

impl SetCppArgs for SearchCommandOptionalArgs {
    fn set_cpp_args(&self) {
        set_optional_arg("-c", &self.min_num_solutions);

        self.generator_args.set_cpp_args();
        self.search_args.set_cpp_args();
        self.search_persistence_args.set_cpp_args();
        self.metric_args.set_cpp_args();
    }
}

impl SetCppArgs for GeneratorArgs {
    fn set_cpp_args(&self) {
        let parsed = self.parse();
        match parsed {
            Generators::Default => {}
            Generators::Custom(parsed) => {
                if !parsed.algs.is_empty() {
                    panic!("Alg generators are unsupported.")
                }
                set_moves_arg(&parsed.moves);
            }
        };
    }
}

impl SetCppArgs for SearchPersistenceArgs {
    fn set_cpp_args(&self) {
        set_optional_arg("--writeprunetables", &self.write_prune_tables);
        if let Some(cache_dir) = &self.cache_dir {
            set_arg(
                "--cachedir",
                &cache_dir.to_str().expect("Invalid cache dir path."),
            );
        }
    }
}

impl SetCppArgs for PerformanceArgs {
    fn set_cpp_args(&self) {
        let num_threads = match self.num_threads {
            Some(num_threads) => num_threads,
            None => num_cpus::get(),
        };
        println!("Setting twsearch to use {} threads.", num_threads);
        rust_api::rust_api_set_arg(&format!("-t {}", num_threads));

        self.memory_args.set_cpp_args();
    }
}

impl SetCppArgs for MemoryArgs {
    fn set_cpp_args(&self) {
        set_optional_arg("-M", &self.memory_mebibytes);
    }
}

impl SetCppArgs for SchreierSimsArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("--schreiersims", true);
        self.performance_args.set_cpp_args();
    }
}

impl SetCppArgs for GodsAlgorithmArgs {
    fn set_cpp_args(&self) {
        if self.optional.start_pattern_args.start_pattern.is_some() {
            eprintln!("Unsupported flag for twsearch-cpp-wrapper: --start-pattern");
            exit(1);
        }
        self.optional.generator_args.set_cpp_args();
        set_boolean_arg("-g", true);
        set_boolean_arg("-F", self.optional.force_arrays);
        set_boolean_arg("-H", self.optional.hash_patterns);
        set_arg("-a", &self.optional.num_antipodes);
        self.optional.performance_args.set_cpp_args();
        self.optional.metric_args.set_cpp_args();
    }
}

impl SetCppArgs for TimingTestArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("-T", true);
        self.performance_args.set_cpp_args();
        self.metric_args.set_cpp_args();
    }
}

impl SetCppArgs for CanonicalAlgsArgs {
    fn set_cpp_args(&self) {
        if self.generator_args.generator_moves_string.is_some() {
            eprintln!(
                "Unsupported flag for `twsearch-cpp-wrapper canonical-algs`: --generator-moves"
            );
            exit(1);
        }
        if self.generator_args.generator_algs.is_some() {
            eprintln!(
                "Unsupported flag for `twsearch-cpp-wrapper canonical-algs`: --generator-algs"
            );
            exit(1);
        }
        set_boolean_arg("-C", true);
        self.performance_args.set_cpp_args();
        self.metric_args.set_cpp_args();
    }
}

impl SetCppArgs for MetricArgs {
    fn set_cpp_args(&self) {
        match self.metric {
            MetricEnum::Hand => {}
            MetricEnum::Quantum => {
                set_boolean_arg("-q", true);
            }
        }
    }
}

impl SetCppArgs for RequiredDefArgs {
    fn set_cpp_args(&self) {}
}

impl SetCppArgs for ScrambleAndTargetPatternOptionalArgs {
    fn set_cpp_args(&self) {
        if let Some(scramble_alg) = &self.scramble_alg {
            let parsed_alg = match scramble_alg.parse::<Alg>() {
                Ok(alg) => alg,
                Err(_) => panic!("Invalid scramble alg."),
            };
            // TODO: Use `cubing::kpuzzle` to handle nested input syntax
            set_arg("--scramblealg", &parsed_alg.to_string())
        };
        set_boolean_arg("-s", self.stdin_scrambles)
    }
}

impl SetCppArgs for ServeArgsForIndividualSearch<'_> {
    fn set_cpp_args(&self) {
        self.commandline_args.set_cpp_args();
        // set_arg("-c", &"1000");
        if let Some(client_args) = self.client_args {
            client_args.set_cpp_args();
        }
        // Unconditional args
        set_arg("--writeprunetables", &"never");
    }
}

impl SetCppArgs for ServeCommandArgs {
    fn set_cpp_args(&self) {
        self.performance_args.set_cpp_args();
    }
}

impl SetCppArgs for ServeClientArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg(
            "--checkbeforesolve",
            is_enabled_with_default_true(&self.check_before_solve),
        );
        set_boolean_arg("--randomstart", self.random_start.unwrap_or(true));
        set_optional_arg("--mindepth", &self.min_depth.map(|o| o.0));
        set_optional_arg("--maxdepth", &self.max_depth.map(|o| o.0));
        set_optional_arg("--startprunedepth", &self.start_prune_depth.map(|o| o.0));
        set_optional_arg("-q", &self.quantum_metric);
        if let Some(move_subset) = &self.generator_moves {
            set_moves_arg(move_subset);
        }
    }
}

impl SetCppArgs for BenchmarkArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("-T", true);
        self.memory_args.set_cpp_args();
        self.generator_args.set_cpp_args();
        self.metric_args.set_cpp_args();
    }
}
