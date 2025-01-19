use std::process;

fn main() {
    let res = cargo_run_bin::cli::run();

    // Only reached if run-bin code fails, otherwise process exits early from within
    // binary::run.
    if let Err(res) = res {
        eprintln!("run-bin failed: {res}");
        process::exit(1);
    }
}
