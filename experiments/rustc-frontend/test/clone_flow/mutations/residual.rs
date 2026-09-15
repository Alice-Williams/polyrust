//! Boolean bookkeeping must stay unobserved, constant and correctly typed.
use super::{Reject, flow, relations, source, value};
use crate::owned_linear::multiple::residual;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind},
    ty::{Ty, TyCtxt},
};
use std::collections::HashSet;

pub(super) fn cases<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &source::Plan<'tcx>,
    original: &mir::Body<'tcx>,
    reject: &mut Reject<'_, 'tcx>,
) -> usize {
    let trace = flow::trace(original).unwrap();
    let Some(flag) = trace
        .assignments
        .iter()
        .find(|a| original.local_decls[a.destination.local].ty == tcx.types.bool)
    else {
        return 0;
    };
    reject("flag type", &|b| {
        b.local_decls[flag.destination.local].ty = tcx.types.u32
    });
    reject("nonconstant flag", &|b| {
        let Rvalue::Use(operand, _) = value(b, flag.location) else {
            unreachable!()
        };
        *operand = Operand::Copy(flag.destination);
    });
    // Isolate the no-reader check: the caller accounts for this synthetic,
    // correctly typed Boolean read; the original flag must still reject.
    let mut changed = original.clone();
    let destination = changed
        .local_decls
        .push(changed.local_decls[flag.destination.local].clone());
    let mut statement =
        changed.basic_blocks[flag.location.block].statements[flag.location.statement_index].clone();
    let StatementKind::Assign(pair) = &mut statement.kind else {
        unreachable!()
    };
    pair.0 = mir::Place::from(destination);
    let Rvalue::Use(operand, _) = &mut pair.1 else {
        unreachable!()
    };
    *operand = Operand::Copy(flag.destination);
    let index = changed.basic_blocks[flag.location.block].statements.len();
    changed.basic_blocks.as_mut()[flag.location.block]
        .statements
        .push(statement);
    let changed_trace = flow::trace(&changed).unwrap();
    let mut used: HashSet<_> = changed_trace
        .assignments
        .iter()
        .filter(|a| changed.local_decls[a.destination.local].ty != tcx.types.bool)
        .map(|a| a.location)
        .collect();
    used.insert(mir::Location {
        block: flag.location.block,
        statement_index: index,
    });
    assert!(
        residual::account(tcx, &changed, &changed_trace, &mut used).is_err(),
        "observed flag escaped no-reader proof"
    );
    let mut borrowed = original.clone();
    let template = trace
        .assignments
        .iter()
        .find(|a| matches!(a.value, Rvalue::Ref(..)))
        .unwrap();
    let mut declaration = borrowed.local_decls[template.destination.local].clone();
    declaration.ty = Ty::new_imm_ref(tcx, tcx.lifetimes.re_erased, tcx.types.bool);
    let local = borrowed.local_decls.push(declaration);
    let mut statement = borrowed.basic_blocks[template.location.block].statements
        [template.location.statement_index]
        .clone();
    let StatementKind::Assign(pair) = &mut statement.kind else {
        unreachable!()
    };
    pair.0 = mir::Place::from(local);
    pair.1 = Rvalue::Ref(
        tcx.lifetimes.re_erased,
        mir::BorrowKind::Shared,
        flag.destination,
    );
    borrowed.basic_blocks.as_mut()[template.location.block]
        .statements
        .push(statement);
    let borrowed_trace = flow::trace(&borrowed).unwrap();
    let mut used: HashSet<_> = borrowed_trace
        .assignments
        .iter()
        .filter(|a| borrowed.local_decls[a.destination.local].ty != tcx.types.bool)
        .map(|a| a.location)
        .collect();
    assert!(
        residual::account(tcx, &borrowed, &borrowed_trace, &mut used).is_err(),
        "borrowed flag escaped no-reader proof"
    );
    let writes: Vec<_> = trace
        .assignments
        .iter()
        .filter(|a| a.destination == flag.destination)
        .collect();
    assert!(writes.len() >= 2);
    let mut unobserved = original.clone();
    *value(&mut unobserved, writes[0].location) = writes[1].value.clone();
    assert!(
        relations::validate(tcx, plan, &unobserved).is_ok(),
        "unused constants became owner identities"
    );
    2
}
