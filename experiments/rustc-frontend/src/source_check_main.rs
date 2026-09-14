#![feature(rustc_private)]
#![forbid(unsafe_code)]
//! Isolated source agreement driver: no backend or publication dependency.
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod compiler_dependencies;
mod inputs;
mod metadata_cli;
mod metadata_dependencies;
mod metadata_stage;
#[cfg(source_callback_contract)]
#[path = "../test/source_callback.rs"]
mod source_callback;
mod source_check;

fn run(arguments: &[String]) -> Result<usize, String> {
    let sysroot = arguments
        .get(1)
        .ok_or("source checker requires its pinned sysroot")?;
    let graph = metadata_cli::parse(&arguments[2..])?;
    #[cfg(not(source_callback_contract))]
    return Ok(source_check::check(&graph, sysroot, |_, _, _| Ok(()))?.len());
    #[cfg(source_callback_contract)]
    source_callback::check(&graph, sysroot)
}

fn main() -> std::process::ExitCode {
    if std::env::var_os("RUSTC_BOOTSTRAP").is_some() {
        eprintln!("input compiler exemption is not permitted");
        return std::process::ExitCode::from(2);
    }
    match run(&std::env::args().collect::<Vec<_>>()) {
        Ok(count) => {
            println!("checked {count} source crates; no target output published");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("source checking: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
