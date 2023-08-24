use twsearch::_internal::cli::{
    CanonicalAlgsArgs, CommonSearchArgs, EnableAutoAlwaysNeverValueEnum, GodsAlgorithmArgs,
    InputDefAndOptionalScrambleFileArgs, MetricArgs, MovesArgs, PerformanceArgs, SchreierSimsArgs,
    SearchCommandArgs, SearchPersistenceArgs, ServeArgsForIndividualSearch, ServeClientArgs,
    ServeCommandArgs, TimingTestArgs,
};

use std::fmt::Display;

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
        set_optional_arg("--mindepth", &self.min_depth);
        set_optional_arg("-m", &self.max_depth);
        set_optional_arg("--startprunedepth", &self.start_prune_depth);
        self.performance_args.set_cpp_args();
    }
}

impl SetCppArgs for SearchCommandArgs {
    fn set_cpp_args(&self) {
        set_optional_arg("-c", &self.min_num_solutions);

        self.moves_args.set_cpp_args();
        self.search_args.set_cpp_args();
        self.search_persistence_args.set_cpp_args();
        self.input_args.set_cpp_args();
        self.metric_args.set_cpp_args();
    }
}

impl SetCppArgs for MovesArgs {
    fn set_cpp_args(&self) {
        if let Some(moves_parsed) = &self.moves_parsed() {
            set_moves_arg(moves_parsed);
        }
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
        self.moves_args.set_cpp_args();
        set_boolean_arg("-g", true);
        set_boolean_arg("-F", self.force_arrays);
        set_boolean_arg("-H", self.hash_states);
        set_arg("-a", &self.num_antipodes);
        self.performance_args.set_cpp_args();
        self.metric_args.set_cpp_args();
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
        set_boolean_arg("-C", true);
        self.performance_args.set_cpp_args();
        self.metric_args.set_cpp_args();
    }
}

impl SetCppArgs for MetricArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("-q", self.quantum_metric);
    }
}

impl SetCppArgs for InputDefAndOptionalScrambleFileArgs {
    fn set_cpp_args(&self) {
        match &self.scramble_alg {
            Some(scramble_alg) => {
                let parsed_alg = match scramble_alg.parse::<Alg>() {
                    Ok(alg) => alg,
                    Err(_) => panic!("Invalid scramble alg."),
                };
                // TODO: Use `cubing::kpuzzle` to handle nested input syntax
                set_arg("--scramblealg", &parsed_alg.to_string())
            }
            None => (),
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
        set_optional_arg("--mindepth", &self.min_depth);
        set_optional_arg("--maxdepth", &self.max_depth);
        set_optional_arg("--startprunedepth", &self.start_prune_depth);
        set_optional_arg("-q", &self.quantum_metric);
        if let Some(move_subset) = &self.move_subset {
            set_moves_arg(move_subset);
        }
    }
}
