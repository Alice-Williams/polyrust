#![feature(rustc_private)]
#![forbid(unsafe_code)]
//! Compiler-only experiment. This binary cannot issue target certificates.
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use portable_rustc_configuration as configuration;
#[path = "../../src/compiler_dependencies.rs"]
mod compiler_dependencies;
#[path = "../../src/inputs.rs"]
mod inputs;
mod mapped;
mod preload;
mod snapshot;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

struct Probe {
    inputs: inputs::DeclaredInputs,
    emit: bool,
    preload: bool,
    result: Option<Result<Vec<String>, String>>,
}

impl Callbacks for Probe {
    fn config(&mut self, config: &mut interface::Config) {
        if self.preload {
            compiler_dependencies::configure(config);
        }
    }

    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some(self.inputs.verify(tcx).map(|()| {
            let mut records = snapshot::read(tcx);
            if self.preload {
                records.extend(snapshot::loaded(tcx));
            }
            records
        }));
        if self.emit && self.result.as_ref().is_some_and(Result::is_ok) {
            Compilation::Continue
        } else {
            Compilation::Stop
        }
    }
}

fn run(args: &[String], emit: bool) -> Result<Vec<String>, String> {
    let config = configuration::Configuration::parse(&args[6..])?;
    if config.mode() != configuration::Mode::PublicPackage {
        return Err("metadata probe requires explicit package configuration".into());
    }
    let inputs = inputs::DeclaredInputs::new(&args[3], config.declared_inputs())?;
    let mut compiler_args = if args[2] == "unmapped" {
        let mut arguments = config.compiler_arguments(inputs.root(), &args[1]);
        arguments.extend(["--emit=metadata".into(), "-o".into(), args[4].clone()]);
        arguments
    } else {
        mapped::arguments(&config, &inputs, &args[1], &args[4])?
    };
    if args[5] != "-" {
        compiler_args.extend(["--extern".into(), format!("renamed={}", args[5])]);
    }
    let mut probe = Probe {
        inputs,
        emit,
        preload: args[2] == "preload" || args[2] == "preload-emit",
        result: None,
    };
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_args, &mut probe);
    });
    if status != std::process::ExitCode::SUCCESS {
        return Err("compiler rejected metadata probe input".into());
    }
    probe.result.ok_or("compiler analysis did not run")?
}

fn main() -> std::process::ExitCode {
    if std::env::var_os("RUSTC_BOOTSTRAP").is_some() {
        eprintln!("input compiler exemption is not permitted");
        return std::process::ExitCode::from(2);
    }
    let args: Vec<String> = std::env::args().collect();
    let result: Result<Vec<String>, String> = (|| {
        if args.len() < 7
            || ![
                "analyze",
                "emit",
                "repeat",
                "unmapped",
                "sequence",
                "preload",
                "preload-emit",
            ]
            .contains(&args[2].as_str())
        {
            return Err(
                "usage: probe SYSROOT MODE ROOT METADATA EXTERN_OR_DASH --package [configuration]"
                    .into(),
            );
        }
        if args[2] == "sequence" {
            return sequence(&args);
        }
        let result = run(&args, args[2] == "emit" || args[2] == "preload-emit")?;
        if args[2] == "repeat" && run(&args, false)? != result {
            return Err("sequential compiler analyses disagree".into());
        }
        Ok(result)
    })();
    match result {
        Ok(records) => {
            for record in records {
                println!("{record}");
            }
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("metadata probe: {error}");
            std::process::ExitCode::from(2)
        }
    }
}

// Two distinct crates are analyzed in one process, retaining only owned records.
fn sequence(args: &[String]) -> Result<Vec<String>, String> {
    let mut dependency = args.to_vec();
    dependency[4] = "-".into();
    dependency[5] = "-".into();
    let mut records = run(&dependency, false)?;
    let mut consumer = args.to_vec();
    consumer[3] = args[4].clone();
    consumer[4] = "-".into();
    let name = consumer
        .iter()
        .position(|value| value == "--crate-name")
        .ok_or("sequence requires an explicit crate name")?;
    *consumer.get_mut(name + 1).ok_or("missing crate name")? = "metadata_consumer".into();
    records.extend(
        run(&consumer, false)?
            .into_iter()
            .filter(|record| record.contains("\tforeign\t")),
    );
    Ok(records)
}
