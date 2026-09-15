//! Check empty assembly, exact authority and package-wide budget carry-over.
use super::{State, lower_functions};
use crate::java_lower::{Place, TypePlan, Value, capabilities, name};
use portable_backend_java::{ast::*, dialect::JavaDialect};
use portable_codegen::TargetAstBuilder;
use rustc_hir::def_id::{CRATE_DEF_ID, LocalDefId};
use rustc_middle::ty::TyCtxt;
use std::{collections::HashMap, sync::Arc};

fn empty() -> State {
    State {
        builder: TargetAstBuilder::new(JavaDialect),
        mappings: capabilities::java_bindings(),
        functions: HashMap::new(),
        imported: HashMap::new(),
        records: HashMap::new(),
        origins: crate::source_origin::Cache::default(),
        remaining: 100_000,
    }
}

pub(crate) fn check(tcx: TyCtxt<'_>, mut state: State, roots: &[LocalDefId]) -> State {
    let (empty, members) = lower_functions(tcx, empty(), &[]).unwrap();
    assert!(members.is_empty() && empty.functions.is_empty() && empty.records.is_empty());
    assert_eq!(empty.remaining, 100_000);
    assert!(empty.into_reader(tcx, CRATE_DEF_ID).is_err());

    let exports = state.origins.exports(tcx).unwrap();
    let functions: HashMap<_, _> = state
        .functions
        .iter()
        .map(|(key, value)| (*key, value.id))
        .collect();
    let imports = state.imported.clone();
    let original_budget = state.remaining;
    let (mut state, members) = lower_functions(tcx, state, &[]).unwrap();
    assert!(members.is_empty());
    assert_eq!(state.remaining, original_budget);
    assert_eq!(state.imported, imports);
    assert!(Arc::ptr_eq(&exports, &state.origins.exports(tcx).unwrap()));
    let mut budget = 97;
    state.remaining = budget;
    for &root in roots {
        let mut reader = state.into_reader(tcx, root).unwrap();
        assert!(std::ptr::eq(reader.checked, tcx.typeck(root)));
        assert_eq!(reader.functions[&root].id, functions[&root]);
        assert_eq!(reader.remaining, budget);
        assert!(reader.bindings.is_empty() && reader.scopes.is_empty());
        assert!(reader.prelude.is_empty() && reader.active_scope.is_none());
        assert!(reader.expression_observations.is_empty());
        assert_eq!((reader.next_local, reader.depth), (0, 0));
        let id = tcx.local_def_id_to_hir_id(root);
        reader.bindings.insert(
            id,
            Place::resolved(
                Value::new(
                    TypePlan::I32,
                    JavaExpr::local(
                        JavaType::primitive(JavaPrimitive::Int),
                        name("poison").unwrap(),
                    ),
                )
                .unwrap(),
            ),
        );
        reader.scopes.insert(id, None);
        reader.active_scope = Some(id);
        reader.prelude.push(JavaStmt::Return(None));
        reader.next_local = 71;
        reader.depth = 72;
        budget = budget.saturating_sub(3);
        reader.remaining = budget;
        state = State::from_reader(reader);
        assert_eq!(state.remaining, budget);
    }
    if let Some(&root) = roots.first() {
        state.remaining = 0;
        let mut reader = state.into_reader(tcx, root).unwrap();
        assert!(
            reader
                .branch(tcx.hir_body_owned_by(root).value, None)
                .is_err()
        );
        state = State::from_reader(reader);
        assert_eq!(state.remaining, 0);
    }
    state.remaining = original_budget;
    println!("JAVA_PACKAGE_STATE_CHECKED\t{}", roots.len());
    state
}
