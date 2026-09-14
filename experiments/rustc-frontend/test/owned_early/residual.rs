//! Isolate dead-temporary checks from the rest of the ownership verifier.
use crate::owned_linear::{flow, multiple::residual};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind},
    ty::TyCtxt,
};
use std::collections::HashSet;

pub(super) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId, body: &mir::Body<'tcx>) {
    let [no, yes] = flow::split(body).unwrap();
    let unit = no
        .assignments
        .iter()
        .find(|a| body.local_decls[a.destination.local].ty == tcx.types.unit)
        .unwrap();
    let boolean = no
        .assignments
        .iter()
        .find(|a| {
            matches!(a.value,
        Rvalue::Use(Operand::Constant(c), _) if c.const_.ty() == tcx.types.bool)
        })
        .unwrap();
    let other_write = yes
        .assignments
        .iter()
        .find(|a| {
            a.destination.local == boolean.destination.local
                && !no.assignments.iter().any(|n| n.location == a.location)
        })
        .unwrap()
        .location;
    let unit_location = unit.location;
    let unit_local = unit.destination.local;
    let bool_local = boolean.destination.local;
    let isolate = |b: &mir::Body<'tcx>, units: bool| {
        let [trace, _] = flow::split(b).unwrap();
        let candidate = if units { unit_local } else { bool_local };
        let mut used: HashSet<_> = trace
            .assignments
            .iter()
            .filter(|a| a.destination.local != candidate)
            .map(|a| a.location)
            .collect();
        let result = if units {
            residual::account_units(tcx, b, &trace, &mut used)
        } else {
            residual::account(tcx, b, &trace, &mut used)
        };
        if result.is_ok() {
            assert_eq!(used.len(), trace.assignments.len());
            assert!(
                !used.contains(&other_write),
                "other arm must not be charged to this path"
            );
        }
        result
    };
    assert!(isolate(body, true).is_ok());
    assert!(isolate(body, false).is_ok());
    let mut observed_unit = body.clone();
    let temporary = observed_unit
        .local_decls
        .push(observed_unit.local_decls[unit_local].clone());
    let mut read =
        body.basic_blocks[unit_location.block].statements[unit_location.statement_index].clone();
    let StatementKind::Assign(pair) = &mut read.kind else {
        unreachable!()
    };
    pair.0 = mir::Place::from(temporary);
    let Rvalue::Use(operand, _) = &mut pair.1 else {
        unreachable!()
    };
    *operand = Operand::Copy(mir::Place::from(unit_local));
    // Place the read only on the *other* arm: a path-local scan would miss it.
    observed_unit.basic_blocks.as_mut()[other_write.block]
        .statements
        .push(read);
    assert!(isolate(&observed_unit, true).is_err());

    let mut nonconstant_unit = body.clone();
    let StatementKind::Assign(pair) = &mut nonconstant_unit.basic_blocks.as_mut()
        [unit_location.block]
        .statements[unit_location.statement_index]
        .kind
    else {
        unreachable!()
    };
    let Rvalue::Use(operand, _) = &mut pair.1 else {
        unreachable!()
    };
    *operand = Operand::Copy(mir::Place::from(unit_local));
    assert!(isolate(&nonconstant_unit, true).is_err());

    let mut wrong_type = body.clone();
    let StatementKind::Assign(pair) = &mut wrong_type.basic_blocks.as_mut()[unit_location.block]
        .statements[unit_location.statement_index]
        .kind
    else {
        unreachable!()
    };
    pair.1 = boolean.value.clone();
    assert!(isolate(&wrong_type, true).is_err());

    let mut other_definition = body.clone();
    let parameter = super::EarlyOwnedBody::from_body(tcx, owner, body)
        .unwrap()
        .guard()
        .parameter()
        .1;
    let StatementKind::Assign(pair) = &mut other_definition.basic_blocks.as_mut()
        [other_write.block]
        .statements[other_write.statement_index]
        .kind
    else {
        unreachable!()
    };
    let Rvalue::Use(operand, _) = &mut pair.1 else {
        unreachable!()
    };
    *operand = Operand::Copy(mir::Place::from(parameter));
    // This does not read the bookkeeping local. Only whole-body definition
    // validation, not the no-reader scan alone, can reject it on the false path.
    assert!(isolate(&other_definition, false).is_err());
    for changed in [
        &observed_unit,
        &nonconstant_unit,
        &wrong_type,
        &other_definition,
    ] {
        assert!(super::EarlyOwnedBody::from_body(tcx, owner, changed).is_err());
    }
}
