//! Actual compiler callback scope/order contract, not a mocked graph traversal.
use portable_rustc_configuration::graph::CrateGraph;
use std::collections::BTreeSet;

pub(crate) fn check(graph: &CrateGraph, sysroot: &str) -> Result<usize, String> {
    let mut calls = Vec::new();
    let results = crate::source_check::check::<(String, rustc_span::def_id::StableCrateId)>(
        graph,
        sysroot,
        |description, tcx, dependencies| {
            let subgraph = graph.rooted_at(description.key())?;
            let expected = subgraph
                .crates()
                .iter()
                .filter(|item| item.key() != description.key())
                .map(|item| item.key().to_owned())
                .collect::<BTreeSet<_>>();
            if dependencies
                .values()
                .map(|result| result.0.clone())
                .collect::<BTreeSet<_>>()
                != expected
            {
                return Err("callback dependency view contains missing or unrelated owners".into());
            }
            for (krate, result) in dependencies.iter() {
                if tcx.stable_crate_id(*krate) != result.1 || !calls.contains(&result.0) {
                    return Err(
                        "callback result is not the exact earlier checked dependency".into(),
                    );
                }
            }
            calls.push(description.key().to_owned());
            Ok((
                description.key().to_owned(),
                tcx.stable_crate_id(rustc_hir::def_id::LOCAL_CRATE),
            ))
        },
    )?;
    if calls
        != graph
            .crates()
            .iter()
            .map(|item| item.key().to_owned())
            .collect::<Vec<_>>()
    {
        return Err("callbacks were not dependency-first and exactly once".into());
    }
    Ok(results.len())
}
