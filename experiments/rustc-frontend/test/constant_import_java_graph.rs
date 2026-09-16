//! Whole-bundle import closure cannot replace original producer certificates.
use super::{CheckedGraph, JavaDependencyApi};
use portable_codegen::TargetSymbolRef;
use portable_java_bundle::{Owner, PreparedBundle};
use std::collections::BTreeSet;

pub(super) fn check(graph: &CheckedGraph) {
    let owners: Vec<_> = graph
        .crates
        .values()
        .map(|member| Owner {
            key: &member.key,
            api: &member.api,
        })
        .collect();
    let prepared = PreparedBundle::new(graph.root, &owners).unwrap();
    let output = prepared.render().unwrap();
    assert!(
        output
            .files()
            .iter()
            .map(|(_, text)| text.len() as u64)
            .sum::<u64>()
            <= prepared.reserved_bytes()
    );
    let mut used = BTreeSet::new();
    for member in graph.crates.values() {
        for function in member.api.functions() {
            let path = function.source().declaration;
            let exports = &function.source().crate_exports;
            let called_bridge = exports.modules.values().any(|bindings| {
                bindings.iter().any(|(name, target)|
                matches!(target, portable_codegen::RustExportTarget::Declaration(id) if *id == path)
                && (name.name == "read_left" || name.name == "read_right"))
            });
            assert_eq!(
                function.call_height(),
                if called_bridge { 2 } else { 1 },
                "constant reads must not introduce call frames"
            );
        }
        for symbol in member
            .api
            .package()
            .ast()
            .files()
            .iter()
            .flat_map(|file| file.items())
            .flat_map(|item| item.names.keys())
        {
            if let TargetSymbolRef::DependencyValue(value) = symbol {
                used.insert(value.constant().package_identity().root());
            }
        }
    }
    for root in &used {
        let remaining: Vec<_> = owners
            .iter()
            .copied()
            .filter(|owner| owner.api.root() != *root)
            .collect();
        assert!(PreparedBundle::new(graph.root, &remaining).is_err());
        let original = &graph.crates[root];
        let replacement =
            JavaDependencyApi::from_certificate(original.api.package().clone()).unwrap();
        let replaced: Vec<_> = owners
            .iter()
            .map(|owner| {
                if owner.api.root() == *root {
                    Owner {
                        key: owner.key,
                        api: &replacement,
                    }
                } else {
                    *owner
                }
            })
            .collect();
        assert!(PreparedBundle::new(graph.root, &replaced).is_err());
    }
    println!("CONSTANT_IMPORT_GRAPH\tjava\t{}", used.len());
}
