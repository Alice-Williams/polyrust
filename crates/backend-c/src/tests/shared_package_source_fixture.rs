//! Shared compiler provenance fixture for public packages and dependency APIs.
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

pub(crate) fn origins() -> (RustSourceOrigin, RustSourceOrigin) {
    let id = |hash| RustDeclarationId {
        crate_id: 7,
        definition_path_hash: hash,
    };
    let location = RustSourceLocation {
        file: "crate/src/lib.rs".into(),
        line: 1,
        column: 0,
    };
    let mut exports = Arc::new(RustCrateExports {
        module_ancestries: std::collections::BTreeMap::new(),
        root: id(1),
        modules: BTreeMap::from([(
            id(1),
            BTreeMap::from([(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: "exported".into(),
                },
                RustExportTarget::Declaration(id(3)),
            )]),
        )]),
    });
    let ancestors: RustModuleAncestry = vec![
        Arc::new(RustModuleDocumentation {
            declaration: id(1),
            parent: None,
            location: location.clone(),
            documentation: vec!["public root".into()],
        }),
        Arc::new(RustModuleDocumentation {
            declaration: id(2),
            parent: Some(id(1)),
            location: location.clone(),
            documentation: vec!["private ancestor".into()],
        }),
    ]
    .into();
    Arc::make_mut(&mut exports)
        .module_ancestries
        .insert(id(1), vec![ancestors[0].clone()].into());
    let public = RustSourceOrigin {
        declaration: id(3),
        node: RustSourceNode::Declaration,
        module: id(2),
        location,
        visibility: RustVisibility::Public,
        externally_reachable: true,
        documentation: vec!["public operation".into()],
        module_ancestors: ancestors,
        crate_exports: exports,
    };
    let helper = RustSourceOrigin {
        declaration: id(4),
        visibility: RustVisibility::RestrictedTo(id(2)),
        externally_reachable: false,
        documentation: vec!["private helper".into()],
        ..public.clone()
    };
    (public, helper)
}
