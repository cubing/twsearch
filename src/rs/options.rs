use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use std::fmt::Display;
use std::io::stdout;
use std::path::PathBuf;
use std::process::exit;

use crate::rust_api;

/// Twsearch
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "twsearch-rs")]
pub struct TwsearchArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a single search (not fully implemented).
    Search {
        #[command(flatten)]
        search_args: CommonSearchArgs,
        #[command(flatten)]
        input_args: InputDefAndOptionalScrambleFileArgs,
    },
    /// Run a search server.
    /// Use with: https://experiments.cubing.net/cubing.js/twsearch/text-ui.html
    Serve {
        #[command(flatten)]
        search_args: CommonSearchArgs,
    },
    SchreierSims {},
    GodsAlgorithm(GodsAlgorithmArgs),

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

    // TODO: Implement this for the server, or remove it as an arg for the `serve` command.
    /// Don't write to disk
    #[clap(long/* , visible_alias = "nowrite" */)]
    pub no_write: bool,

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

impl SetCppArgs for CommonSearchArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("--randomstart", self.check_before_solve);
        set_boolean_arg("--checkbeforesolve", self.random_start);
        set_optional_arg("--mindepth", self.min_depth);
        set_optional_arg("--maxdepth", self.max_depth);
        set_optional_arg("--startprunedepth", self.start_prune_depth);
        set_boolean_arg("--nowrite", self.no_write);
        self.performance_args.set_cpp_args();
    }
}

#[derive(Args, Debug)]
pub struct PerformanceArgs {
    /// Defaults to the number of logical CPU cores available.
    #[clap(long/* , visible_short_alias = 't' */)]
    pub num_threads: Option<usize>,

    /// Memory to use in MiB. See `README.md` for advice on how to tune memory usage.
    #[clap(long/* , visible_short_alias = 'm' */, id = "MEGABYTES")]
    pub memory_mb: Option<usize>,
}

impl SetCppArgs for PerformanceArgs {
    fn set_cpp_args(&self) {
        let num_threads = match self.num_threads {
            Some(num_threads) => num_threads,
            None => num_cpus::get(),
        };
        println!("Setting twsearch to use {} threads.", num_threads);
        rust_api::rust_arg(&format!("-t {}", num_threads));

        set_optional_arg("-m", self.memory_mb);
    }
}

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Print completions for the given shell.
    /// These can be loaded/stored permanently (e.g. when using Homebrew), but they can also be sourced directly, e.g.:
    ///
    ///  twsearch completions fish | source # fish
    ///  source <(twsearch completions zsh) # zsh
    #[clap(id = "SHELL")]
    shell: Shell,
}

#[derive(Args, Debug)]
pub struct GodsAlgorithmArgs {
    #[command(flatten)]
    pub input_args: InputDefFileOnlyArgs,

    #[clap(long/* , visible_short_alias = 'a' */, default_value_t = 20)]
    pub num_antipodes: u32, // TODO: Change this to `Option<u32>` while still displaying a semantic default value?

    #[command(flatten)]
    pub performance_args: PerformanceArgs,
}

impl SetCppArgs for GodsAlgorithmArgs {
    fn set_cpp_args(&self) {
        set_boolean_arg("-g", true);
        set_arg("-a", self.num_antipodes);
        self.performance_args.set_cpp_args();
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
    #[clap()]
    pub scramble_file: Option<PathBuf>,
}

fn completions_for_shell(cmd: &mut clap::Command, generator: impl Generator) {
    generate(generator, cmd, "twsearch", &mut stdout());
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

pub fn set_arg<T: Display>(arg_flag: &str, arg: T) {
    rust_api::rust_arg(&format!("{} {}", arg_flag, arg));
}

pub fn set_boolean_arg(arg_flag: &str, arg: bool) {
    if arg {
        rust_api::rust_arg(arg_flag);
    }
}

pub fn set_optional_arg<T: Display>(arg_flag: &str, arg: Option<T>) {
    if let Some(v) = arg {
        rust_api::rust_arg(&format!("{} {}", arg_flag, v));
    }
}

pub fn reset_args_from(arg_structs: Vec<&dyn SetCppArgs>) {
    rust_api::rust_reset();
    for arg_struct in arg_structs {
        arg_struct.set_cpp_args();
    }
}
