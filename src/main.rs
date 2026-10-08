use std::io::{stderr, stdout};

fn main() {
    if let Err(err) = gsbt::cli::run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
    // Keep stdout/stderr handles alive for the compiler's benefit in tests.
    let _ = (stdout(), stderr());
}
