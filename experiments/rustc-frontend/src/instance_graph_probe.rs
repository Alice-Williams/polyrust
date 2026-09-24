#![feature(rustc_private)]
#![forbid(unsafe_code)]
//! Observation-only fan-in graph. Never lowers or publishes target packages.
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

#[path = "../test/instance_graph/audit.rs"]
mod audit;
mod compiler_dependencies;
mod inputs;
#[path = "../test/instance_graph/interner.rs"]
mod interner;
mod metadata_cli;
mod metadata_dependencies;
mod metadata_stage;
#[path = "../test/scalar_result_identity/shape.rs"]
mod shape;
mod source_check;

use interner::{Interner, Observation};
use portable_codegen::{RustCanonicalInstanceKey, RustDeclarationId};
use rustc_hir::def::DefKind;
use rustc_middle::ty::{self, TyCtxt};
use std::{collections::BTreeMap, sync::Arc};

struct CheckedCrate {
    root: RustDeclarationId,
    owners: BTreeMap<RustCanonicalInstanceKey, Arc<Observation>>,
}

fn observe(tcx: TyCtxt<'_>, interner: &mut Interner) -> Result<CheckedCrate, String> {
    let mut owners = BTreeMap::new();
    for item in tcx.hir_crate_items(()).free_items() {
        let definition = item.owner_id.def_id;
        if tcx.def_kind(definition) != DefKind::Fn {
            continue;
        }
        if tcx.generics_of(definition).count() != 0 {
            return Err("generic result signatures are not observed".into());
        }
        let signature = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.fn_sig(definition).instantiate_identity(),
            )
            .map_err(|_| "signature normalization failed")?;
        if !signature.bound_vars().is_empty() {
            return Err("bound result signatures are not observed".into());
        }
        let signature = signature.skip_binder();
        if signature.inputs().len() != 1 {
            return Err("identity observation requires one result parameter".into());
        }
        let input = shape::ResultShape::observe(tcx, signature.inputs()[0])?;
        let output = shape::ResultShape::observe(tcx, signature.output())?;
        audit::verify(tcx, signature.inputs()[0], input.facts())?;
        audit::verify(tcx, signature.output(), output.facts())?;
        if !input.same_instance(&output) {
            return Err("result instance changed across signature".into());
        }
        let input = interner.intern(&input)?;
        let output = interner.intern(&output)?;
        if !Arc::ptr_eq(&input, &output) {
            return Err("signature did not reuse original instance authority".into());
        }
        owners.insert(input.facts().key(), input);
    }
    Ok(CheckedCrate {
        root: shape::identity(tcx, rustc_hir::def_id::CRATE_DEF_ID.to_def_id()),
        owners,
    })
}

fn run(arguments: &[String]) -> Result<Vec<String>, String> {
    let sysroot = arguments.get(1).ok_or("missing pinned sysroot")?;
    let graph = metadata_cli::parse(&arguments[2..])?;
    let mut instances = Interner::new(graph.crates().len())?;
    let results = source_check::check::<CheckedCrate>(&graph, sysroot, |_, tcx, dependencies| {
        let current = observe(tcx, &mut instances)?;
        for (krate, dependency) in dependencies.iter() {
            if tcx.stable_crate_id(*krate).as_u64() != dependency.root.crate_id {
                return Err("instance dependency source owner differs".into());
            }
            for (key, handle) in &current.owners {
                if let Some(original) = dependency.owners.get(key)
                    && !Arc::ptr_eq(original, handle)
                {
                    return Err("fan-in did not reuse original instance authority".into());
                }
            }
        }
        Ok(current)
    })?;
    let frozen = instances.freeze()?;
    if frozen.owners().next().is_none() {
        return Err("no result signature observed".into());
    }
    for observed in results.values().flat_map(|item| item.owners.values()) {
        if !frozen.contains_original(observed) {
            return Err("frozen graph omitted original authority".into());
        }
    }
    let mut lines: Vec<_> = frozen
        .owners()
        .map(|(_, observation)| format!("INSTANCE {:?}", observation.facts()))
        .collect();
    lines.push(format!(
        "CHECKED {} CRATES {} USES; no target output published",
        results.len(),
        frozen.uses()
    ));
    Ok(lines)
}

#[cfg(instance_graph_forge)]
#[allow(dead_code)]
fn forge(interner: &mut Interner, facts: portable_codegen::RustCanonicalInstanceFacts) {
    let _ = interner.intern_facts(facts);
}

#[cfg(instance_graph_frozen_forge)]
#[allow(dead_code)]
fn mutate_frozen(frozen: &mut interner::Frozen) {
    frozen.owners.clear();
}

fn main() -> std::process::ExitCode {
    if std::env::var_os("RUSTC_BOOTSTRAP").is_some() {
        eprintln!("input compiler exemption is not permitted");
        return std::process::ExitCode::from(2);
    }
    #[cfg(instance_graph_controls)]
    interner::run_controls();
    match run(&std::env::args().collect::<Vec<_>>()) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("instance graph: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
