use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use cubing::alg::{Alg, Move};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::stdout;
use std::path::PathBuf;
use std::process::exit;
use twips::_internal::canonical_fsm::search_generators::Generators;
use twips::_internal::notation::metric::TurnMetric;
use twips::_internal::search::search_logger::VerbosityLevel;
use twips::experimental_lib_api::{KPuzzleSource, PatternSource};

use twips::_internal::errors::{ArgumentError, TwipsError};
use twips::_internal::search::iterative_deepening::continuation_condition::ContinuationCondition;
use twips::_internal::search::iterative_deepening::solution_moves::alg_to_moves;
use twips::_internal::search::prune_table_trait::Depth;
use twips::scramble::{DerivationSalt, DerivationSeed, Puzzle};

/// twips — solve every puzzle.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "twips")]
pub struct TwipsArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}

// Clippy warns on only 200 bytes of difference between variants, but this enum is not memory usage sensitive. (TODO: can we have it warn us on a much larger difference?)
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Run a single search.
    Search(SearchCommandArgs),
    // The URL is not for Rust docs, it is printed to the comandline by `clap` (which does not remove brackets around URLs).
    #[allow(rustdoc::bare_urls)]
    /// Run a search server.
    /// Use with: https://experiments.cubing.net/cubing.js/twsearch/text-ui.html
    Serve(ServeCommandArgs),

    /// Solve a known puzzle.
    SolveKnownPuzzle(SolveKnownPuzzleCommandArgs),

    // TOOD: Detect identical pieces and warn/error (and give advice on how to run the "same" search with fully-distinguishable pieces).
    /// Run the Schreier-Sims algorithm to calculate the number of reachable patterns.
    ///
    /// Warning: Does NOT account for identical pieces.
    SchreierSims(SchreierSimsArgs),
    /// Enumerate the entire pattern graph and print antipodes.
    GodsAlgorithm(GodsAlgorithmArgs),
    /// Run a timing test for given definition.
    TimingTest(TimingTestArgs),
    /// Enumerate canonical algs (move sequences) at iterative depths.
    CanonicalAlgs(CanonicalAlgsArgs),
    /// Generate a scramble
    Scramble(ScrambleArgs),
    /// Test the scramble finder implementations directly.
    ScrambleFinder(ScrambleFinderArgs),
    /// Derive scrambles
    Derive(DeriveArgs),

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

    /// Continue (or start) search after this alg.
    /// If the alg is a valid solution, it will be skipped.
    #[clap(long, group = "continue_search")]
    continue_after: Option<Alg>,

    /// Continue (or start) search at this alg.
    /// If the alg is a valid solution, it will be the first one returned.
    #[clap(long, group = "continue_search")]
    continue_at: Option<Alg>,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

impl CommonSearchArgs {
    pub fn continuation_condition(&self) -> Result<ContinuationCondition, TwipsError> {
        Ok(match (&self.continue_after, &self.continue_at) {
            (None, None) => ContinuationCondition::None,
            (Some(after), None) => {
                ContinuationCondition::After(Self::process_continuation_alg_arg(after)?)
            }
            (None, Some(at)) => ContinuationCondition::At(Self::process_continuation_alg_arg(at)?),
            (Some(_), Some(_)) => {
                // TODO: figure out how to make this unrepresentable using idiomatic `clap` config.
                panic!("Specifying `--continue-after` and `--continue-at` simultaneously is supposed to be impossible.");
            }
        })
    }

    fn process_continuation_alg_arg(alg: &Alg) -> Result<Vec<Move>, TwipsError> {
        let Some(moves) = alg_to_moves(alg) else {
            return Err((ArgumentError {
                description: "Non-moves used in the continuation alg.".to_owned(),
            })
            .into());
        };
        Ok(moves)
    }
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

#[derive(Args, Debug)]
pub struct SolveKnownPuzzleCommandArgs {
    #[clap(value_parser = puzzle_from_id)]
    pub puzzle: Puzzle,

    /// Scramble setup alg
    // TODO: support pattern input via file.
    pub scramble_setup_alg: Alg,

    /// By default, the command prints a URL for the solution to `stderr`. Pass this to disable the URL printing functionality.
    #[clap(long, default_value = "true")]
    pub print_link: Option<bool>,
}

fn puzzle_from_id(s: &str) -> Result<Puzzle, String> {
    Puzzle::try_from_id(s).map_err(|e| e.description)
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
    #[clap(long, value_delimiter = ',')]
    pub generator_moves: Option<Vec<Move>>,

    /// A comma-separated list of algs to use. All multiples of these
    /// algs are considered. For example, `--algs U,F,R2` only permits
    /// half-turns on R, and all possible turns on U and F.
    #[clap(long, value_delimiter = ',')]
    pub generator_algs: Option<Vec<Alg>>,
}

impl GeneratorArgs {
    pub fn generators(&self) -> Generators {
        match (&self.generator_moves, &self.generator_algs) {
            (None, None) => Generators::Default,
            (moves, algs) => Generators::Custom {
                moves: moves.clone().unwrap_or_default(),
                algs: algs.clone().unwrap_or_default(),
            },
        }
    }
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

// TODO
// impl EnableAutoAlwaysNeverValueEnum {
//     pub fn enabled(&self, auto_case: fn() -> bool) -> bool {
//         match self {
//             EnableAutoAlwaysNeverValueEnum::Auto => auto_case(),
//             EnableAutoAlwaysNeverValueEnum::Never => false,
//             EnableAutoAlwaysNeverValueEnum::Always => true,
//         }
//     }
// }

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
    ///  twips completions fish | source # fish
    ///  source <(twips completions zsh) # zsh
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

    #[command(flatten)]
    pub metric_args: MetricArgs,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
    // TODO
    // #[clap(long/* , visible_short_alias = 'a' */, default_value_t = 20)]
    // pub num_antipodes: u32, // TODO: Change this to `Option<u32>` while still displaying a semantic default value?
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

#[derive(Clone, Args, Debug, Default)]
pub struct MetricArgs {
    #[clap(long)]
    pub metric: Option<TurnMetric>,
}

#[derive(Args, Debug)]
pub struct ScrambleArgs {
    /// Event ID (WCA or unofficial)
    pub event_id: String,

    /// Amount of scrambles
    #[clap(long, default_value_t = 1)]
    pub amount: usize,
}

#[derive(Args, Debug)]
pub struct ScrambleFinderArgs {
    #[command(subcommand)]
    pub command: ScrambleFinderCommand,
}

#[derive(Subcommand, Debug)]
pub enum ScrambleFinderCommand {
    /// Search for a solution to the given setup alg.
    Search(ScrambleFinderSearchArgs),
    /// Run the filter on the given setup alg.
    Filter(ScrambleFinderFilterArgs),
}

#[derive(Args, Debug)]
// TODO: combine with `ScrambleFinderFilterArgs`?
pub struct ScrambleFinderSearchArgs {
    #[clap(long, default_value = "true")]
    pub print_link: Option<bool>,

    #[clap(long, default_value_t = false)]
    pub apply_filtering: bool,

    #[command(flatten)]
    pub filter_args: ScrambleFinderFilterArgs,
}

#[derive(Args, Debug)]
// TODO: combine with `ScrambleFinderSolveArgs`?
pub struct ScrambleFinderFilterArgs {
    /// Event ID (WCA or unofficial)
    pub event_id: String,

    /// Scramble setup alg
    // TODO: support pattern input via file.
    pub scramble_setup_alg: Alg,
}

#[derive(Args, Debug)]
pub struct DefOnlyArgs {
    #[clap()]
    pub def_file: PathBuf,
    // TODO: remove this
    // #[clap(long)]
    // pub debug_print_serialized_json: bool,
}

impl From<DefOnlyArgs> for KPuzzleSource {
    fn from(def_only_args: DefOnlyArgs) -> Self {
        Self::FilePath(def_only_args.def_file.clone())
    }
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
    pub scramble_alg: Option<Alg>,
    // TODO: also allow an alg for this?
    /// Use the target pattern from the specified file instead of the default start pattern from the defintion.
    #[clap(long, help_heading = "Scramble input")]
    pub experimental_target_pattern: Option<PathBuf>,
}

impl ScrambleAndTargetPatternOptionalArgs {
    pub fn search_pattern(&self) -> PatternSource {
        match (&self.scramble_alg, &self.scramble_file) {
            (None, None) => {
                // TODO
                println!("No scramble specified, exiting.");
                exit(0);
            }
            (None, Some(scramble_file)) => PatternSource::FilePath(scramble_file.clone()),
            (Some(scramble_alg), None) => {
                PatternSource::AlgAppliedToDefaultPattern(scramble_alg.clone())
            }
            (Some(_), Some(_)) => {
                eprintln!("Error: specified both a scramble alg and a scramble file, exiting.");
                exit(1);
            }
        }
    }

    pub fn target_pattern(&self) -> PatternSource {
        match &self.experimental_target_pattern {
            Some(experimental_target_pattern) => {
                PatternSource::FilePath(experimental_target_pattern.clone())
            }
            None => PatternSource::DefaultFromDefinition,
        }
    }
}

#[derive(Args, Debug)]
pub struct DeriveArgs {
    /// Derivation seed. This is a 64-char hex value (representing a 32-byte
    /// value), where:
    ///
    /// - the first byte is a fixed protocol sentinel value (0x67), and
    /// - the second byte encodes the derivation level (which must be 0 for this
    ///   subcommand).
    pub root_derivation_seed: DerivationSeed,

    /// Example value:
    ///
    /// EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF/scrambles/333/r1/g1/a1/333/sub1/candidate1
    ///
    /// If present:
    ///
    /// - Level 3 must be a valid event ID.
    /// - Level 7 must be a valid monoscramble event ID.
    #[clap(required = true, value_delimiter = '/')]
    pub derivation_salts: Vec<DerivationSalt>,
}

#[derive(Args, Debug, Default)]
pub struct StartPatternArgs {
    #[clap(long)]
    pub start_pattern: Option<PathBuf>,
}

impl StartPatternArgs {
    pub fn start_pattern_source(self) -> PatternSource {
        match self.start_pattern {
            Some(start_pattern) => PatternSource::FilePath(start_pattern),
            None => PatternSource::DefaultFromDefinition,
        }
    }
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
    generate(generator, cmd, "twips", &mut stdout());
}

pub fn get_args() -> TwipsArgs {
    let mut command = TwipsArgs::command();

    let args = TwipsArgs::parse();
    if let CliCommand::Completions(completions_args) = args.command {
        completions_for_shell(&mut command, completions_args.shell);
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

#[cfg(test)]
mod tests {
    use crate::args::TwipsArgs;

    // https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#testing
    #[test]
    fn test_clap_args() {
        use clap::CommandFactory;

        TwipsArgs::command().debug_assert();
    }
}
