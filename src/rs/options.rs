use clap::{CommandFactory, Parser};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use std::io::stdout;
use std::process::exit;

/// Twsearch
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "folderify")]
pub struct TwsearchArgs {
    #[clap(long)]
    pub check_before_solve: bool,

    /// Defaults to the number of logical CPU cores available.
    #[clap(visible_short_alias = 't', long)]
    pub num_threads: Option<usize>,

    #[clap(long, alias = "startprunedepth", id = "DEPTH")]
    pub start_prune_depth: Option<usize>,

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
