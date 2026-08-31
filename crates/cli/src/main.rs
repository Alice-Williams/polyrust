//! Command-line composition root.

#![forbid(unsafe_code)]

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    std::process::exit(portable_cli::run(&arguments, &mut stdout, &mut stderr));
}
