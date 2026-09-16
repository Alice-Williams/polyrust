//! Read-only observation around the actual declaration mapping.
use super::{LiteralValue, LocalConstantInput};
use crate::c_lower::Reader;
use rustc_middle::ty;

pub(super) fn snapshot(reader: &Reader<'_>) -> String {
    // Exhaustive destructuring makes future Reader state an explicit test decision.
    let Reader {
        tcx,
        checked,
        mappings: _,
        registry,
        header,
        constants,
        foreign_constants,
        file,
        function,
        functions,
        foreign_functions,
        parameters,
        root,
        records,
        declarations,
        bindings,
        control_scopes,
        origins,
        next_binding,
        next_scope,
        active_scope,
        prelude,
        next_temporary,
    } = reader;
    // Mapping bindings are stateless unit types; compiler handles are immutable.
    let session = (std::ptr::from_ref(tcx.sess), std::ptr::from_ref(*checked));
    format!(
        "{:?}",
        (
            session,
            (header, constants, foreign_constants),
            (
                registry,
                file,
                function,
                functions,
                foreign_functions,
                parameters,
                root,
                records,
                declarations
            ),
            (
                bindings,
                control_scopes,
                origins,
                next_binding,
                next_scope,
                active_scope,
                prelude,
                next_temporary
            ),
        )
    )
}

pub(super) fn check(reader: &Reader<'_>, input: LocalConstantInput<'_>, before: &str) {
    assert_eq!(
        snapshot(reader),
        before,
        "local constant changed runtime state"
    );
    let (definition, statement) = input.origin();
    let rustc_hir::StmtKind::Item(item) = statement.kind else {
        panic!("constant item");
    };
    assert_eq!(
        reader.tcx.hir_item(item).owner_id.def_id.to_def_id(),
        definition
    );
    let declared = reader
        .tcx
        .try_normalize_erasing_regions(
            ty::TypingEnv::fully_monomorphized(),
            reader.tcx.type_of(definition).instantiate_identity(),
        )
        .expect("checked scalar constant type");
    match input.value() {
        LiteralValue::Bool(_) => assert!(matches!(declared.kind(), ty::Bool)),
        LiteralValue::I32(_) => assert!(matches!(declared.kind(), ty::Int(ty::IntTy::I32))),
        LiteralValue::I64(_) => assert!(matches!(declared.kind(), ty::Int(ty::IntTy::I64))),
    }
    eprintln!(
        "LOCAL_CONSTANT_AST\tc\t{:?}\t{:?}",
        definition,
        input.value()
    );
}
