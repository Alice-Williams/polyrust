//! Synthetic source provenance used only by independently certified target tests.
use portable_backend_c::ast::*;
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

pub fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}

pub fn origins(crate_id: u64, names: &[String]) -> Vec<RustSourceOrigin> {
    let id = |definition_path_hash| RustDeclarationId {
        crate_id,
        definition_path_hash,
    };
    let location = RustSourceLocation {
        file: format!("crate_{crate_id}/lib.rs"),
        line: 1,
        column: 0,
    };
    let ancestry = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: location.clone(),
        documentation: vec![],
    });
    let exports = Arc::new(RustCrateExports {
        root: id(1),
        modules: BTreeMap::from([(
            id(1),
            names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    (
                        RustExportName {
                            namespace: RustExportNamespace::Value,
                            name: name.clone(),
                        },
                        RustExportTarget::Declaration(id(10 + i as u64)),
                    )
                })
                .collect(),
        )]),
        module_ancestries: BTreeMap::from([(id(1), vec![ancestry.clone()].into())]),
    });
    names
        .iter()
        .enumerate()
        .map(|(i, _)| RustSourceOrigin {
            declaration: id(10 + i as u64),
            node: RustSourceNode::Declaration,
            module: id(1),
            location: location.clone(),
            visibility: RustVisibility::Public,
            externally_reachable: true,
            documentation: vec![],
            module_ancestors: vec![ancestry.clone()].into(),
            crate_exports: exports.clone(),
        })
        .collect()
}
