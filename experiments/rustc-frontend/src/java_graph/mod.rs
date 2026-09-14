//! Dependency-first compiler analysis joined to exact Java owner certificates.
use crate::{metadata_cli, source_check};
mod inventory;
mod publication;
pub(crate) use inventory::{CheckedCrate, CheckedGraph};
#[cfg(any(
    java_graph_wrong_owner,
    java_graph_wrong_declaration,
    java_graph_wrong_signature
))]
#[path = "../../test/java_graph_mutations.rs"]
pub(crate) mod mutations;

use portable_backend_java::dialect::{JavaDependencyApi, JavaDialect};
use portable_codegen::{TargetLinker, certify_resolved_package, verify_unresolved_package};
use rustc_hir::def_id::{DefId, LOCAL_CRATE};

#[cfg(java_graph_probe)]
#[path = "../../test/java_graph_probe.rs"]
mod probe;

pub(crate) fn check(sysroot: &str, arguments: &[String]) -> Result<usize, String> {
    let graph = lower(sysroot, arguments)?;
    if !graph.crates.contains_key(&graph.root) {
        return Err("checked Java graph has no root owner".into());
    }
    let owners: Vec<_> = graph
        .crates
        .values()
        .map(|member| portable_java_bundle::Owner {
            key: &member.key,
            api: &member.api,
        })
        .collect();
    portable_java_bundle::PreparedBundle::new(graph.root, &owners)?;
    #[cfg(java_graph_probe)]
    probe::write(&graph)?;
    Ok(graph.crates.len())
}

fn lower(sysroot: &str, arguments: &[String]) -> Result<CheckedGraph, String> {
    let graph = metadata_cli::parse(arguments)?;
    let checked =
        source_check::check::<CheckedCrate>(&graph, sysroot, |description, tcx, dependencies| {
            let lookup = |definition: DefId| {
                let owner = dependencies
                    .get(&definition.krate)
                    .map(|item| &item.api)
                    .ok_or("foreign call has no source-authenticated owning Java package")?;
                #[cfg(java_graph_wrong_owner)]
                let owner = mutations::owner(owner, dependencies);
                if owner.root().crate_id != tcx.stable_crate_id(definition.krate).as_u64() {
                    return Err(
                        "foreign Java certificate belongs to the wrong compiler crate".into(),
                    );
                }
                let function = owner
                    .function(crate::source_origin::identity(tcx, definition))
                    .cloned()
                    .ok_or("foreign call is not in the certified public Java API")?;
                #[cfg(java_graph_wrong_declaration)]
                let function = mutations::declaration(owner, function);
                Ok(function)
            };
            let package = crate::java_lower::lower(
                tcx,
                crate::java_lower::Selection::PublicApi,
                Some(&lookup),
            )?;
            let verified = verify_unresolved_package(&JavaDialect, package)
                .map_err(|error| format!("Java graph verification: {error:?}"))?;
            let linked = TargetLinker::new(JavaDialect)
                .link_ast(&verified)
                .map_err(|error| format!("Java graph linking: {error:?}"))?;
            let certificate = certify_resolved_package(&JavaDialect, linked)
                .map_err(|error| format!("Java graph certification: {error:?}"))?;
            let api = JavaDependencyApi::from_certificate(certificate)?;
            if api.root().crate_id != tcx.stable_crate_id(LOCAL_CRATE).as_u64() {
                return Err("checked Java package differs from its compiler source owner".into());
            }
            #[cfg(java_graph_probe)]
            probe::owner(tcx, &api);
            Ok(CheckedCrate {
                api,
                key: description.key().to_owned(),
            })
        })?;
    CheckedGraph::from_checked(graph.root_key(), checked)
}

pub(crate) fn publish(
    sysroot: &str,
    destination: &std::path::Path,
    arguments: &[String],
) -> Result<usize, String> {
    let graph = lower(sysroot, arguments)?;
    publication::publish(destination, &graph)?;
    Ok(graph.crates.len())
}
