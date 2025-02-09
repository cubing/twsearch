use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use cubing::alg::{Alg, Move};
use cubing::kpuzzle::KPuzzle;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::stdout;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use crate::_internal::puzzle_traits::puzzle_traits::GroupActionPuzzle;
use crate::_internal::search::prune_table_trait::Depth;

/// twsearch-cpp-wrapper — a native Rust wrapper for `twsearch` functionality.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "twsearch-cpp-wrapper")]
pub struct TwsearchCppWrapperArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}

/// twsearch — solve every puzzle.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "twsearch")]
pub struct TwsearchArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Run a single search.
    Search(SearchCommandArgs),
    // The URL is not for Rust docs, it is printed to the comandline by `clap` (which does not remove brackets around URLs).
    #[allow(rustdoc::bare_urls)]
    /// Run a search server.
    /// Use with: https://experiments.cubing.net/cubing.js/twsearch/text-ui.html
    Serve(ServeCommandArgs),

    // TOOD: Detect identical pieces and warn/error (and give advice on how to run the "same" search with fully-distinguishable pieces).
    /// Run the Schreier-Sims algorithm to calculate the number of reachable patterns.
    ///
    /// Warning: Does NOT account for identical pieces.
    SchreierSims(SchreierSimsArgs),
    /// Enumerate the entire pattern graph and print antipodes.
    GodsAlgorithm(GodsAlgorithmArgs),
    /// Run a timing test for given definition.
    TimingTest(TimingTestArgs),
    // Enumerate canonical algs (move sequences) at iterative depths.
    CanonicalAlgs(CanonicalAlgsArgs),
    // Generate a scramble
    Scramble(ScrambleArgs),

    /// Run an internal benchmark suite.
    Benchmark(BenchmarkArgs),

    /// Print completions for the given shell.
    Completions(CompletionsArgs),
}

#[derive(Args, Debug, Default)]
pub struct CommonSearchArgs {
    /// Check that a position is valid before attempting to solve it. This may take extra time or memory for large puzzles.
    #[clap(long/*, visible_alias = "checkbeforesolve" */)]
    pub check_before_solve: Option<EnableAutoAlwaysNeverValueEnum>,

    /// Randomize the search order. This can produce different solutions the
    /// same run of each input, which is desirable for some use cases.
    #[clap(long/*, visible_alias = "randomstart"`*/)]
    pub random_start: bool,

    /// Print all optimal solutions.
    #[clap(long)]
    pub all_optimal: bool,

    /// Depth to start the pruning table. This can avoid multiple pruning table
    /// expansions that can already be anticipated by starting with a sufficient
    /// depth.
    #[clap(long/*, visible_alias = "startprunedepth" */, id = "DEPTH")]
    pub start_prune_depth: Option<Depth>,

    /// Start solution search at this depth.
    #[clap(long/* , visible_alias = "mindepth" */)]
    pub min_depth: Option<Depth>,

    /// Stop solution search at this depth.
    #[clap(long/* , visible_alias = "maxdepth" */)]
    pub max_depth: Option<Depth>,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

#[derive(Args, Debug)]
pub struct SearchCommandArgs {
    #[command(flatten)]
    pub def_args: RequiredDefArgs,

    #[command(flatten)]
    pub optional: SearchCommandOptionalArgs,
}

#[derive(Args, Debug, Default)]
pub struct SearchCommandOptionalArgs {
    #[clap(long/* , visible_short_alias = 't' */)]
    pub min_num_solutions: Option<usize>,

    #[command(flatten)]
    pub generator_args: GeneratorArgs,
    #[command(flatten)]
    pub search_args: CommonSearchArgs,
    #[command(flatten)]
    pub search_persistence_args: SearchPersistenceArgs,
    #[command(flatten)]
    pub metric_args: MetricArgs,
    #[command(flatten)]
    pub verbosity_args: VerbosityArgs,

    // We place this last show it shows at the end of `--help` (and therefore just above the next shell prompt).
    #[command(flatten)]
    pub scramble_and_target_pattern_optional_args: ScrambleAndTargetPatternOptionalArgs,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum VerbosityLevel {
    Silent,
    Error,
    Warning,
    Info,
}

impl Default for VerbosityLevel {
    fn default() -> Self {
        Self::Warning
    }
}

#[derive(Args, Debug, Default)]
pub struct VerbosityArgs {
    #[clap(long)]
    pub verbosity: Option<VerbosityLevel>,
}

#[derive(Args, Debug, Default)]
pub struct GeneratorArgs {
    /// A comma-separated list of moves to use. All multiples of these
    /// moves are considered. For example, `--moves U,F,R2` only permits
    /// half-turns on R, and all possible turns on U and F.
    #[clap(long = "generator-moves")]
    pub generator_moves_string: Option<String>,

    /// A comma-separated list of algs to use. All multiples of these
    /// algs are considered. For example, `--algs U,F,R2` only permits
    /// half-turns on R, and all possible turns on U and F.
    #[clap(long)]
    pub generator_algs: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Generators {
    Default,
    Custom(CustomGenerators),
}

impl Generators {
    pub fn enumerate_moves_for_kpuzzle(&self, kpuzzle: &KPuzzle) -> Vec<Move> {
        if let Generators::Custom(custom_generators) = self {
            if !custom_generators.algs.is_empty() {
                eprintln!("WARNING: Alg generators are not implemented yet. Ignoring.");
            }
        };

        match self {
            Generators::Default => kpuzzle.puzzle_definition_all_moves(),
            Generators::Custom(generators) => generators.moves.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CustomGenerators {
    pub moves: Vec<Move>,
    pub algs: Vec<Alg>,
}

impl GeneratorArgs {
    pub fn parse(&self) -> Generators {
        let moves = parse_comma_separated(&self.generator_moves_string);
        let algs = parse_comma_separated(&self.generator_algs);
        match (moves, algs) {
            (None, None) => Generators::Default,
            (moves, algs) => Generators::Custom(CustomGenerators {
                moves: moves.unwrap_or_default(),
                algs: algs.unwrap_or_default(),
            }),
        }
    }
}

fn parse_comma_separated<T: FromStr<Err = E>, E: Display>(
    input: &Option<String>,
) -> Option<Vec<T>> {
    input.as_ref().map(|moves| {
        moves
            .split(',')
            .by_ref()
            .map(|move_str| match move_str.parse::<T>() {
                Ok(r#move) => r#move,
                Err(err) => {
                    eprintln!("Invalid move ({}): {}", err, move_str);
                    panic!("Exiting due to invalid move.")
                }
            })
            .collect()
    })
}

#[derive(Args, Debug, Default)]
pub struct SearchPersistenceArgs {
    #[clap(long, help_heading = "Persistence"/* , visible_alias = "writeprunetables" */)]
    pub write_prune_tables: Option<EnableAutoAlwaysNeverValueEnum>,

    #[clap(long, help_heading = "Persistence"/* , visible_alias = "cachedir" */)]
    pub cache_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
pub enum EnableAutoAlwaysNeverValueEnum {
    Auto,
    Never,
    Always,
}

impl EnableAutoAlwaysNeverValueEnum {
    pub fn enabled(&self, auto_case: fn() -> bool) -> bool {
        match self {
            EnableAutoAlwaysNeverValueEnum::Auto => auto_case(),
            EnableAutoAlwaysNeverValueEnum::Never => false,
            EnableAutoAlwaysNeverValueEnum::Always => true,
        }
    }
}

impl Display for EnableAutoAlwaysNeverValueEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EnableAutoAlwaysNeverValueEnum::Auto => "auto",
            EnableAutoAlwaysNeverValueEnum::Never => "never",
            EnableAutoAlwaysNeverValueEnum::Always => "always",
        };
        write!(f, "{}", s)
    }
}

#[derive(Args, Debug, Default)]
pub struct PerformanceArgs {
    /// Defaults to the number of logical CPU cores available.
    #[clap(long, help_heading = "Performance"/* , visible_short_alias = 't' */)]
    pub num_threads: Option<usize>,

    #[command(flatten)]
    pub memory_args: MemoryArgs,
}

#[derive(Args, Debug, Default)]
pub struct MemoryArgs {
    /// Memory to use in MiB. See `README.md` for advice on how to tune memory usage.
    #[clap(long = "memory-MiB", help_heading = "Performance"/* , visible_short_alias = 'm' */, id = "MEBIBYTES")]
    pub memory_mebibytes: Option<usize>,
}

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Print completions for the given shell.
    /// These can be loaded/stored permanently (e.g. when using Homebrew), but they can also be sourced directly, e.g.:
    ///
    ///  twsearch completions fish | source # fish
    ///  source <(twsearch completions zsh) # zsh
    #[clap(verbatim_doc_comment, id = "SHELL")]
    shell: Shell,
}

// TODO: support moves arg?
#[derive(Args, Debug)]
pub struct SchreierSimsArgs {
    #[command(flatten)]
    pub def_args: DefOnlyArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

#[derive(Args, Debug)]
pub struct GodsAlgorithmArgs {
    #[command(flatten)]
    pub def_args: DefOnlyArgs,

    #[command(flatten)]
    pub optional: GodsAlgorithmOptionalArgs,
}

#[derive(Args, Debug, Default)]
pub struct GodsAlgorithmOptionalArgs {
    // TODO: move this to a shared place.
    #[command(flatten)]
    pub start_pattern_args: StartPatternArgs,

    #[command(flatten)]
    pub generator_args: GeneratorArgs,

    #[clap(long/* , visible_short_alias = 'a' */, default_value_t = 20)]
    pub num_antipodes: u32, // TODO: Change this to `Option<u32>` while still displaying a semantic default value?

    /// Force the use of arrays rather than bitmaps.
    #[clap(long/* , visible_short_alias = 'F' */)]
    pub force_arrays: bool,

    /// Use 128-bit hash to encode patterns rather than actual packed pattern representation.
    #[clap(long/* , visible_short_alias = 'H' */)]
    pub hash_patterns: bool,

    #[command(flatten)]
    pub metric_args: MetricArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

#[derive(Args, Debug)]
pub struct TimingTestArgs {
    #[command(flatten)]
    pub def_args: DefOnlyArgs,

    #[command(flatten)]
    pub metric_args: MetricArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

#[derive(Args, Debug)]
pub struct CanonicalAlgsArgs {
    #[command(flatten)]
    pub def_args: DefOnlyArgs,

    #[command(flatten)]
    pub generator_args: GeneratorArgs,

    #[command(flatten)]
    pub metric_args: MetricArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

#[derive(Clone, Args, Debug)]
pub struct MetricArgs {
    #[clap(long, default_value_t = MetricEnum::Hand)]
    pub metric: MetricEnum,
}

impl Default for MetricArgs {
    fn default() -> Self {
        Self {
            // TODO: deduplicate with `IDFSearchConstructionOptions`
            metric: MetricEnum::Hand,
        }
    }
}

#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
pub enum MetricEnum {
    Hand,
    Quantum,
}

impl Display for MetricEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MetricEnum::Hand => "hand",
            MetricEnum::Quantum => "quantum",
        };
        write!(f, "{}", s)
    }
}

#[derive(Args, Debug)]
pub struct ScrambleArgs {
    /// Event ID (WCA or unofficial)
    pub event_id: String,

    /// Amount of scrambles
    #[clap(long, default_value = "1")]
    pub amount: usize,
}

#[derive(Args, Debug)]
pub struct DefOnlyArgs {
    #[clap()]
    pub def_file: PathBuf,
    // TODO: remove this
    // #[clap(long)]
    // pub debug_print_serialized_json: bool,
}

#[derive(Args, Debug)]
pub struct RequiredDefArgs {
    #[command(flatten)]
    pub def_args: DefOnlyArgs,
}

#[derive(Args, Debug, Default)]
pub struct ScrambleAndTargetPatternOptionalArgs {
    /// Solve all the scrambles from the given file.
    #[clap(help_heading = "Scramble input", group = "scramble_input")]
    pub scramble_file: Option<PathBuf>,
    /// Solve a single scramble specified directly as an argument.
    #[clap(long/*, visible_alias = "scramblealg" */, help_heading = "Scramble input", group = "scramble_input")]
    pub scramble_alg: Option<String>, // TODO: Make `Alg` implement `Send` (e.g. by using `Arc`, possibly through an optional feature or a separate thread-safe `Alg` struct)
    /// Solve a list of scrambles passed to standard in (separated by newlines).
    #[clap(long, help_heading = "Scramble input", group = "scramble_input"/* , visible_short_alias = 's' */)]
    pub stdin_scrambles: bool,
    /// Use the target pattern from the specified file instead of the default start pattern from the defintion.
    #[clap(long, help_heading = "Scramble input")]
    pub experimental_target_pattern: Option<PathBuf>,
}

#[derive(Args, Debug, Default)]
pub struct StartPatternArgs {
    #[clap(long)]
    pub start_pattern: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct BenchmarkArgs {
    #[command(flatten)]
    pub def_args: DefOnlyArgs,

    #[command(flatten)]
    pub memory_args: MemoryArgs,

    #[command(flatten)]
    pub generator_args: GeneratorArgs,

    #[command(flatten)]
    pub metric_args: MetricArgs,
}

fn completions_for_shell(cmd: &mut clap::Command, generator: impl Generator) {
    generate(generator, cmd, "twsearch", &mut stdout());
}

pub fn get_options() -> TwsearchArgs {
    let mut command = TwsearchArgs::command();

    let args = TwsearchArgs::parse();
    if let CliCommand::Completions(completions_args) = args.command {
        completions_for_shell(&mut command, completions_args.shell);
        exit(0);
    };

    args
}

fn completions_for_shell_cpp_wrapper(cmd: &mut clap::Command, generator: impl Generator) {
    generate(generator, cmd, "twsearch-cpp-wrapper", &mut stdout());
}

pub fn get_options_cpp_wrapper() -> TwsearchCppWrapperArgs {
    let mut command = TwsearchCppWrapperArgs::command();

    let args = TwsearchCppWrapperArgs::parse();
    if let CliCommand::Completions(completions_args) = args.command {
        completions_for_shell_cpp_wrapper(&mut command, completions_args.shell);
        exit(0);
    };

    args
}

////////

pub struct ServeArgsForIndividualSearch<'a> {
    pub commandline_args: &'a ServeCommandArgs,
    pub client_args: &'a Option<ServeClientArgs>,
}

#[derive(Args, Debug)]
pub struct ServeCommandArgs {
    #[command(flatten)]
    pub performance_args: PerformanceArgs,
    #[command(flatten)]
    pub verbosity_args: VerbosityArgs,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServeClientArgs {
    // TODO: moves, min num solutions
    // TODO: allow the client to set performance args (with bounds checks) and prune table (if enabled by server).
    pub check_before_solve: Option<EnableAutoAlwaysNeverValueEnum>,
    pub random_start: Option<bool>,
    pub min_depth: Option<Depth>,
    pub max_depth: Option<Depth>,
    pub start_prune_depth: Option<Depth>,
    pub quantum_metric: Option<bool>, // TODO: enum
    pub generator_moves: Option<Vec<Move>>,
}
