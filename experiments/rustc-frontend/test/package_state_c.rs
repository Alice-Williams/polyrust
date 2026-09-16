//! Probe the production lifecycle without fabricating a source function.
use super::{State, lower_functions};
use crate::c_lower::{capabilities, origin};
use portable_backend_c::ast::*;
use portable_codegen::RelativeOutputPath;
use rustc_hir::def_id::{CRATE_DEF_ID, LocalDefId};
use rustc_middle::ty::TyCtxt;
use std::{collections::HashMap, sync::Arc};

fn empty() -> State {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("empty.c").unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    State {
        registry,
        file,
        mappings: capabilities::c_bindings(),
        functions: HashMap::new(),
        foreign_functions: HashMap::new(),
        header: None,
        constants: HashMap::new(),
        foreign_constants: HashMap::new(),
        records: HashMap::new(),
        declarations: vec![],
        origins: origin::Cache::default(),
    }
}

pub(crate) fn check(tcx: TyCtxt<'_>, mut state: State, roots: &[LocalDefId]) -> State {
    let empty = lower_functions(tcx, empty(), &[], None).unwrap();
    assert!(empty.prototypes.is_empty() && empty.public_prototypes.is_empty());
    assert!(empty.definitions.is_empty() && empty.state.functions.is_empty());
    assert!(empty.state.declarations.is_empty() && empty.state.records.is_empty());
    assert!(empty.state.into_reader(tcx, CRATE_DEF_ID).is_err());

    let file = state.file.clone();
    let functions = state.functions.clone();
    let constant_imports = state.foreign_constants.clone();
    let imports = state.foreign_functions.clone();
    let exports = state.origins.exports(tcx).unwrap();
    let result = lower_functions(tcx, state, &[], None).unwrap();
    assert!(result.prototypes.is_empty() && result.public_prototypes.is_empty());
    assert!(result.definitions.is_empty());
    state = result.state;
    assert_eq!(state.file, file);
    state.registry.check_file(&file).unwrap();
    assert_eq!(state.functions, functions);
    assert_eq!(state.foreign_functions, imports);
    assert!(Arc::ptr_eq(&exports, &state.origins.exports(tcx).unwrap()));
    for &root in roots {
        let mut reader = state.into_reader(tcx, root).unwrap();
        assert!(std::ptr::eq(reader.checked, tcx.typeck(root)));
        assert_eq!(reader.foreign_constants, constant_imports);
        assert_eq!(reader.root, root);
        assert_eq!(reader.function, functions[&root]);
        assert!(reader.bindings.is_empty() && reader.control_scopes.is_empty());
        assert!(reader.parameters.is_empty() && reader.prelude.is_empty());
        assert!(reader.active_scope.is_none());
        assert_eq!(
            (
                reader.next_binding,
                reader.next_scope,
                reader.next_temporary
            ),
            (0, 0, 0)
        );
        reader.next_binding = 71;
        reader.next_scope = 72;
        reader.next_temporary = 73;
        state = State::from_reader(reader);
    }
    assert_eq!(state.functions, functions);
    assert_eq!(state.foreign_constants, constant_imports);
    println!("C_PACKAGE_STATE_CHECKED\t{}", roots.len());
    state
}
