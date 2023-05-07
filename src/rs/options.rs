use clap::{CommandFactory, Parser};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use std::fmt::Display;
use std::io::stdout;
use std::process::exit;

use crate::ffi;

/// Twsearch
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "folderify")]
pub struct TwsearchArgs {
    #[clap(long, visible_alias = "checkbeforesolve")]
    pub check_before_solve: bool,

    #[clap(long, visible_alias = "randomstart")]
    pub random_start: bool,

    /// Defaults to the number of logical CPU cores available.
    #[clap(visible_short_alias = 't', long)]
    pub num_threads: Option<usize>,

    #[clap(long, visible_alias = "startprunedepth", id = "DEPTH")]
    pub start_prune_depth: Option<usize>,

    #[clap(long, visible_alias = "mindepth")]
    pub min_depth: Option<usize>,

    #[clap(long, visible_alias = "maxdepth")]
    pub max_depth: Option<usize>,

    #[clap(long, visible_short_alias = 'm', id = "MEGABYTES")]
    pub memory_mb: Option<usize>,

    /// Print completions for the given shell (instead of running any commands).
    /// These can be loaded/stored permanently (e.g. when using Homebrew), but they can also be sourced directly, e.g.:
    ///
    ///  twsearch --completions fish | source # fish
    ///  source <(twsearch --completions zsh) # zsh
    #[clap(long, id = "SHELL")]
    pub completions: Option<Shell>,
}

fn completions_for_shell(cmd: &mut clap::Command, generator: impl Generator) {
    generate(generator, cmd, "twsearch", &mut stdout());
}

pub fn get_options() -> TwsearchArgs {
    let mut command = TwsearchArgs::command();

    let args = TwsearchArgs::parse();
    if let Some(shell) = args.completions {
        completions_for_shell(&mut command, shell);
        exit(0);
    }

    args
}

fn set_boolean_arg(arg_flag: &str, arg: bool) {
    if arg {
        ffi::rust_arg(arg_flag);
    }
}

fn set_optional_arg<T: Display>(arg_flag: &str, arg: Option<T>) {
    if let Some(v) = arg {
        ffi::rust_arg(&format!("{} {}", arg_flag, v));
    }
}

pub fn reset_args(args: &TwsearchArgs) {
    ffi::rust_reset();

    let num_threads = match args.num_threads {
        Some(num_threads) => num_threads,
        None => num_cpus::get(),
    };
    println!("Setting search to use {} threads.", num_threads);
    ffi::rust_arg(&format!("-t {}", num_threads));

    set_boolean_arg("--randomstart", args.check_before_solve);
    set_boolean_arg("--checkbeforesolve", args.random_start);

    set_optional_arg("--mindepth", args.min_depth);
    set_optional_arg("--maxdepth", args.max_depth);
    set_optional_arg("--startprunedepth", args.start_prune_depth);

    set_optional_arg("-m", args.memory_mb);
}
