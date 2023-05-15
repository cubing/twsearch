use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use cubing::alg::{Alg, Move};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::stdout;
use std::path::PathBuf;
use std::process::exit;

use crate::rust_api;

/// twsearch-rs — a native Rust wrapper for `twsearch` functionality.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "twsearch-rs")]
pub struct TwsearchArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a single search.
    Search(SearchCommandArgs),
    /// Run a search server.
    /// Use with: https://experiments.cubing.net/cubing.js/twsearch/text-ui.html
    Serve(ServeCommandArgs),

    // TOOD: Detect identical pieces and warn/error (and give advice on how to run the "same" search with fully-distinguishable pieces).
    /// Run the Schreier-Sims algorithm to calculate the number of reachable states.
    ///
    /// Warning: Does NOT account for identical pieces.
    SchreierSims(SchreierSimsArgs),
    /// Enumerate the entire state graph and print antipodes.
    GodsAlgorithm(GodsAlgorithmArgs),
    /// Run a timing test for given definition.
    TimingTest(TimingTestArgs),
    // Enumerate canonical algs (move sequences) at iterative depths.
    CanonicalAlgs(CanonicalAlgsArgs),

    /// Print completions for the given shell.
    Completions(CompletionsArgs),
}

pub trait SetCppArgs {
    fn set_cpp_args(&self);
}

#[derive(Args, Debug)]
pub struct CommonSearchArgs {
    /// Check that a position is legal before attempting to solve it. This may take extra time or memory for large puzzles.
    #[clap(long/*, visible_alias = "checkbeforesolve" */)]
    pub check_before_solve: bool,

    /// Randomize the search order. This can produce different solutions the
    /// same run of each input, which is desirable for some use cases.
    #[clap(long/*, visible_alias = "randomstart"`*/)]
    pub random_start: bool,

    /// Depth to start the pruning table. This can avoid multiple pruning table
    /// expansions that can already be anticipated by starting with a sufficient
    /// depth.
    #[clap(long/*, visible_alias = "startprunedepth" */, id = "DEPTH")]
    pub start_prune_depth: Option<usize>,

    /// Start solution search at this depth.
    #[clap(long/* , visible_alias = "mindepth" */)]
    pub min_depth: Option<usize>,

    /// Stop solution search at this depth.
    #[clap(long/* , visible_alias = "maxdepth" */)]
    pub max_depth: Option<usize>,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

impl SetCppArgs for CommonSearchArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("--randomstart", self.check_before_solve);
        set_boolean_arg("--checkbeforesolve", self.random_start);
        set_optional_arg("--mindepth", &self.min_depth);
        set_optional_arg("-m", &self.max_depth);
        set_optional_arg("--startprunedepth", &self.start_prune_depth);
        self.performance_args.set_cpp_args();
    }
}

#[derive(Args, Debug)]
pub struct SearchCommandArgs {
    #[clap(long/* , visible_short_alias = 't' */)]
    pub min_num_solutions: Option<u32>,

    #[command(flatten)]
    pub moves_args: MovesArgs,
    #[command(flatten)]
    pub search_args: CommonSearchArgs,
    #[command(flatten)]
    pub search_persistence_args: SearchPersistenceArgs,
    #[command(flatten)]
    pub metric_args: MetricArgs,

    // We place this last show it shows at the end of `--help` (and therefore just above the next shell prompt).
    #[command(flatten)]
    pub input_args: InputDefAndOptionalScrambleFileArgs,
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

// TODO: generalize this to a "definition modification" args struct?
#[derive(Args, Debug)]
pub struct MovesArgs {
    /// A comma-separated list of moves to use. All multiples of these
    /// moves are considered. For example, `--moves U,F,R2` only permits
    /// half-turns on R, and all possible turns on U and F.
    #[clap(long)]
    pub moves: Option<String>,
}

impl SetCppArgs for MovesArgs {
    fn set_cpp_args(&self) {
        if let Some(moves) = &self.moves {
            let moves: Vec<Move> = moves
                .split(',')
                .by_ref()
                .map(|move_str| match move_str.parse::<Move>() {
                    Ok(r#move) => r#move,
                    Err(err) => {
                        eprintln!("Invalid move ({}): {}", err, move_str);
                        panic!("Exiting due to invalid move.")
                    }
                })
                .collect();
            set_moves_arg(&moves);
        }
    }
}

#[derive(Args, Debug)]
pub struct SearchPersistenceArgs {
    #[clap(long/* , visible_alias = "writeprunetables" */)]
    pub write_prune_tables: Option<WritePruneTables>,

    #[clap(long/* , visible_alias = "cachedir" */)]
    pub cache_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum WritePruneTables {
    Auto,
    Never,
    Always,
}

impl Display for WritePruneTables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WritePruneTables::Auto => "auto",
            WritePruneTables::Never => "never",
            WritePruneTables::Always => "always",
        };
        write!(f, "{}", s)
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

#[derive(Args, Debug)]
pub struct PerformanceArgs {
    /// Defaults to the number of logical CPU cores available.
    #[clap(long, help_heading = "Performance"/* , visible_short_alias = 't' */)]
    pub num_threads: Option<usize>,

    /// Memory to use in MiB. See `README.md` for advice on how to tune memory usage.
    #[clap(long = "memory-MiB", help_heading = "Performance"/* , visible_short_alias = 'm' */, id = "MEBIBYTES")]
    pub memory_mebibytes: Option<usize>,
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

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Print completions for the given shell.
    /// These can be loaded/stored permanently (e.g. when using Homebrew), but they can also be sourced directly, e.g.:
    ///
    ///  twsearch-rs completions fish | source # fish
    ///  source <(twsearch-rs completions zsh) # zsh
    #[clap(verbatim_doc_comment, id = "SHELL")]
    shell: Shell,
}

#[derive(Args, Debug)]
pub struct SchreierSimsArgs {
    #[command(flatten)]
    pub input_args: InputDefFileOnlyArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

impl SetCppArgs for SchreierSimsArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("--schreiersims", true);
        self.performance_args.set_cpp_args();
    }
}

#[derive(Args, Debug)]
pub struct GodsAlgorithmArgs {
    #[command(flatten)]
    pub input_args: InputDefFileOnlyArgs,

    #[command(flatten)]
    pub moves_args: MovesArgs,

    #[clap(long/* , visible_short_alias = 'a' */, default_value_t = 20)]
    pub num_antipodes: u32, // TODO: Change this to `Option<u32>` while still displaying a semantic default value?

    /// Force the use of arrays rather than bitmaps.
    #[clap(long/* , visible_short_alias = 'F' */)]
    pub force_arrays: bool,

    /// Use 128-bit hash to encode states rather than actual packed state representation.
    #[clap(long/* , visible_short_alias = 'H' */)]
    pub hash_states: bool,

    #[command(flatten)]
    pub metric_args: MetricArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
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

#[derive(Args, Debug)]
pub struct TimingTestArgs {
    #[command(flatten)]
    pub input_args: InputDefFileOnlyArgs,

    #[command(flatten)]
    pub metric_args: MetricArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

impl SetCppArgs for TimingTestArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("-T", true);
        self.performance_args.set_cpp_args();
        self.metric_args.set_cpp_args();
    }
}

#[derive(Args, Debug)]
pub struct CanonicalAlgsArgs {
    #[command(flatten)]
    pub input_args: InputDefFileOnlyArgs,

    #[command(flatten)]
    pub metric_args: MetricArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

impl SetCppArgs for CanonicalAlgsArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("-C", true);
        self.performance_args.set_cpp_args();
        self.metric_args.set_cpp_args();
    }
}

#[derive(Args, Debug)]
pub struct MetricArgs {
    #[clap(long/* , visible_short_alias = 'q' */)]
    pub quantum_metric: bool,
}

impl SetCppArgs for MetricArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("-q", self.quantum_metric);
    }
}

#[derive(Args, Debug)]
pub struct InputDefFileOnlyArgs {
    #[clap()]
    pub def_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct InputDefAndOptionalScrambleFileArgs {
    #[command(flatten)]
    pub def_file_wrapper_args: InputDefFileOnlyArgs,
    /// Solve all the scrambles from the given file.
    #[clap(help_heading = "Scramble input", group = "scramble_input")]
    pub scramble_file: Option<PathBuf>,
    /// Solve a single scramble specified directly as an argument.
    #[clap(long/*, visible_alias = "scramblealg" */, help_heading = "Scramble input", group = "scramble_input")]
    pub scramble_alg: Option<String>, // TODO: Make `Alg` implement `Send` (e.g. by using `Arc`, possibly through an optional feature or a separate thread-safe `Alg` struct)
    /// Solve a list of scrambles passed to standard in (separated by newlines).
    #[clap(long, help_heading = "Scramble input", group = "scramble_input"/* , visible_short_alias = 's' */)]
    pub stdin_scrambles: bool,
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

fn completions_for_shell(cmd: &mut clap::Command, generator: impl Generator) {
    generate(generator, cmd, "twsearch-rs", &mut stdout());
}

pub fn get_options() -> TwsearchArgs {
    let mut command = TwsearchArgs::command();

    let args = TwsearchArgs::parse();
    if let Command::Completions(completions_args) = args.command {
        completions_for_shell(&mut command, completions_args.shell);
        exit(0);
    };

    args
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

////////

pub struct ServeArgsForIndividualSearch<'a> {
    pub commandline_args: &'a ServeCommandArgs,
    pub client_args: &'a Option<ServeClientArgs>,
}

impl SetCppArgs for ServeArgsForIndividualSearch<'_> {
    fn set_cpp_args(&self) {
        self.commandline_args.set_cpp_args();
        if let Some(client_args) = self.client_args {
            client_args.set_cpp_args();
        }
        // Unconditional args
        set_arg("--writeprunetables", &"never");
    }
}

#[derive(Args, Debug)]
pub struct ServeCommandArgs {
    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

impl SetCppArgs for ServeCommandArgs {
    fn set_cpp_args(&self) {
        self.performance_args.set_cpp_args();
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServeClientArgs {
    // TODO: moves, min num solutions
    // TODO: allow the client to set performance args (with bounds checks) and prune table (if enabled by server).
    pub check_before_solve: Option<bool>,
    pub random_start: Option<bool>,
    pub min_depth: Option<u32>,
    pub max_depth: Option<u32>,
    pub start_prune_depth: Option<u32>,
    pub quantum_metric: Option<bool>, // TODO: enum
    pub move_subset: Option<Vec<Move>>,
}

impl SetCppArgs for ServeClientArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg(
            "--checkbeforesolve",
            self.check_before_solve.unwrap_or(true),
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
