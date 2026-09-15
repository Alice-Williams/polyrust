//! Private complete-body corruption oracle; malformed MIR is never a public input.
use super::{relations, source};
use crate::owned_linear::flow;
use rustc_hir::{def::DefKind, def_id::LocalDefId};
use rustc_middle::{
    mir::{self, StatementKind, TerminatorKind},
    ty::TyCtxt,
};

#[path = "mutation_cleanup.rs"]
mod cleanup;
#[path = "mutation_events.rs"]
mod events;
#[path = "mutation_provenance.rs"]
mod provenance;

pub(super) type Reject<'a, 'tcx> = dyn FnMut(&str, &dyn Fn(&mut mir::Body<'tcx>)) + 'a;

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let plan = source::read(tcx, owner).unwrap();
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let matched = relations::validate(tcx, owner, &plan, &original).unwrap();
    let trace = flow::trace(&original).unwrap();
    let other = tcx
        .hir_body_owners()
        .find(|id| *id != owner && tcx.def_kind(*id) == DefKind::Fn)
        .unwrap();
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert_ne!(
            fingerprint(&changed),
            fingerprint(&original),
            "vacuous: {name}"
        );
        assert!(
            relations::validate(tcx, owner, &plan, &changed).is_err(),
            "admitted nested corruption: {name}"
        );
        cases += 1;
    };
    reject("owner", &|b| {
        b.source = tcx
            .mir_drops_elaborated_and_const_checked(other)
            .borrow()
            .source
    });
    reject("phase", &|b| b.phase = mir::MirPhase::Built);
    reject("parameter count", &|b| b.arg_count -= 1);
    reject("extra Box storage", &|b| {
        b.local_decls
            .push(b.local_decls[matched.constructions[0].binding().1].clone());
    });
    events::check(tcx, &original, &matched, &mut reject);
    cleanup::check(tcx, &original, &matched, &mut reject);
    provenance::check(tcx, owner, &original, &matched, &mut reject);
    for assignment in &trace.assignments {
        let location = assignment.location;
        // Unread bool/unit bookkeeping has its own narrow residual contract.
        if matches!(
            original.local_decls[assignment.destination.local].ty.kind(),
            rustc_middle::ty::Bool | rustc_middle::ty::Tuple(_)
        ) {
            continue;
        }
        reject("missing assignment", &|b| {
            b.basic_blocks.as_mut()[location.block].statements[location.statement_index].kind =
                StatementKind::Nop
        });
    }
    for &(location, _) in &trace.drops {
        reject("missing drop", &|b| {
            let TerminatorKind::Drop { target, .. } =
                b.basic_blocks[location.block].terminator().kind
            else {
                panic!("drop");
            };
            b.basic_blocks.as_mut()[location.block]
                .terminator_mut()
                .kind = TerminatorKind::Goto { target };
        });
    }
    reject("premature return", &|b| {
        b.basic_blocks.as_mut()[mir::START_BLOCK]
            .terminator_mut()
            .kind = TerminatorKind::Return
    });
    assert!(cases >= 100, "incomplete corruption inventory: {cases}");
    println!("nested body corruptions rejected: {cases}");
    provenance::renaming(tcx, owner, &original, &matched);
}

pub(super) fn statement<'a, 'tcx>(
    body: &'a mut mir::Body<'tcx>,
    location: mir::Location,
) -> &'a mut mir::Statement<'tcx> {
    &mut body.basic_blocks.as_mut()[location.block].statements[location.statement_index]
}
pub(super) fn value<'a, 'tcx>(
    body: &'a mut mir::Body<'tcx>,
    location: mir::Location,
) -> &'a mut mir::Rvalue<'tcx> {
    let StatementKind::Assign(pair) = &mut statement(body, location).kind else {
        panic!("assignment");
    };
    &mut pair.1
}

fn fingerprint(body: &mir::Body<'_>) -> String {
    // MIR's pretty printer hides nongeneric ADT arguments and indexes variants.
    // Record the complete raw aggregate tuple separately, before printing the
    // remainder. This also safely fingerprints intentionally invalid variants.
    let mut body = body.clone();
    let mut aggregates = Vec::new();
    for block in body.basic_blocks.as_mut() {
        for statement in &mut block.statements {
            if let StatementKind::Assign(pair) = &statement.kind
                && let mir::Rvalue::Aggregate(kind, operands) = &pair.1
                && let mir::AggregateKind::Adt(def, variant, args, annotation, active) = &**kind
            {
                aggregates.push(format!(
                    "{:?}/{def:?}/{variant:?}/{args:?}/{annotation:?}/{active:?}/{operands:?}",
                    pair.0
                ));
                statement.kind = StatementKind::Nop;
            }
        }
    }
    format!("{body:?}/{aggregates:?}")
}
