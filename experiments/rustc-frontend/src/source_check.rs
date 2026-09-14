//! Dependency-first compiler checking. No target packages are published here.
mod agreement;
mod dependencies;
mod inventory;
pub(crate) use dependencies::CheckedDependencies;

use crate::{
    compiler_dependencies, inputs::DeclaredInputs, metadata_dependencies::MetadataDependencies,
    metadata_stage::MetadataStage,
};
use portable_rustc_configuration::graph::{CrateDescription, CrateGraph};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use std::collections::BTreeMap;

/// Only owned callback results survive compiler invocations. A failed graph
/// drops every result rather than returning a successfully checked prefix.
pub(crate) fn check<T: Send + Sync>(
    graph: &CrateGraph,
    sysroot: &str,
    mut lower: impl for<'tcx> FnMut(
        &CrateDescription,
        TyCtxt<'tcx>,
        &CheckedDependencies<'_, 'tcx, T>,
    ) -> Result<T, String>
    + Send,
) -> Result<BTreeMap<String, T>, String> {
    let sources = inventory::resolve(graph)?;
    let sysroot = std::path::Path::new(sysroot)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let toolchain_artifacts = inventory::toolchain_artifacts(&sysroot)?;
    let mut evidence = BTreeMap::new();
    let mut results = BTreeMap::new();
    for description in graph.crates() {
        let subgraph = graph.rooted_at(description.key())?;
        let dependency_results = subgraph
            .crates()
            .iter()
            .filter(|item| item.key() != description.key())
            .map(|item| {
                let result = results
                    .get(item.key())
                    .ok_or("checked dependency result missing")?;
                Ok((item.key().to_owned(), result))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        let resolved = sources
            .get(description.key())
            .ok_or("resolved crate missing")?;
        let inputs = DeclaredInputs::new(resolved.root(), &resolved.declared_input_arguments())?;
        if inputs.root() != resolved.root() {
            return Err("source root changed after resolution".into());
        }
        // Reserve a private scratch stage, never a declared metadata destination.
        // This type has no publication method; compilation stops after analysis.
        let output = MetadataStage::prepare(&std::env::temp_dir())?;
        let dependencies = MetadataDependencies::prepare(
            &subgraph,
            resolved,
            output
                .output()
                .parent()
                .ok_or("scratch stage has no parent")?,
        )?;
        let mut arguments = resolved.metadata_arguments(
            sysroot.to_str().ok_or("sysroot is not UTF-8")?,
            output
                .output()
                .to_str()
                .ok_or("scratch path is not UTF-8")?,
        )?;
        arguments.extend(dependencies.compiler_arguments(&subgraph)?);
        let mut callback = |tcx: TyCtxt<'_>| {
            inputs.verify(tcx)?;
            let loaded = agreement::verify(
                tcx,
                &subgraph,
                &dependencies,
                &evidence,
                &toolchain_artifacts,
            )?;
            let dependencies = CheckedDependencies::new(tcx, loaded, &dependency_results)?;
            let own = agreement::SourceEvidence::local(tcx);
            if evidence
                .values()
                .any(|old: &agreement::SourceEvidence| old.same_owner(&own))
            {
                return Err("distinct defining keys have the same compiler crate identity".into());
            }
            let result = lower(description, tcx, &dependencies)?;
            Ok((own, result))
        };
        let mut analysis = Analysis {
            callback: &mut callback,
            result: None,
        };
        let status = rustc_driver::catch_with_exit_code(|| {
            rustc_driver::run_compiler(&arguments, &mut analysis);
        });
        if status != std::process::ExitCode::SUCCESS {
            return Err(format!(
                "compiler rejected source crate {}",
                description.key()
            ));
        }
        let (own, result) = analysis
            .result
            .ok_or("source compiler analysis did not run")??;
        evidence.insert(description.key().to_owned(), own);
        results.insert(description.key().to_owned(), result);
    }
    Ok(results)
}

struct Analysis<'a, T> {
    callback: &'a mut (dyn for<'tcx> FnMut(TyCtxt<'tcx>) -> Result<T, String> + Send),
    result: Option<Result<T, String>>,
}

impl<T> Callbacks for Analysis<'_, T> {
    fn config(&mut self, config: &mut interface::Config) {
        compiler_dependencies::configure(config);
    }

    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some((self.callback)(tcx));
        Compilation::Stop
    }
}
