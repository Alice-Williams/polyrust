//! Coherent same-type borrow and staging counterexamples.
use super::{Reject, flow, relations, source, term, value};
use crate::owned_linear::cloning::{Loan, Owner};
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::TyCtxt,
};

pub(super) fn cases<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &source::Plan<'tcx>,
    body: &mir::Body<'tcx>,
    matched: &relations::Matched<'tcx>,
    reject: &mut Reject<'_, 'tcx>,
) {
    let trace = flow::trace(body).unwrap();
    let at = trace.calls[1].0;
    let cloned = matched
        .bindings
        .iter()
        .find(|(tag, _, _, _)| *tag == Owner::Cloned)
        .unwrap()
        .2;
    let steps = match matched.loan {
        Loan::Direct(step) => vec![step],
        Loan::Reborrow { initial, argument } => vec![initial, argument],
    };
    let initial = steps[0];
    let argument = *steps.last().unwrap();
    let other_read = matched
        .bindings
        .iter()
        .rev()
        .find(|(owner, _, _, _)| *owner != plan.read)
        .unwrap()
        .2;
    let cast = trace
        .assignments
        .iter()
        .find(|assignment| matches!(assignment.value, Rvalue::Cast(_, Operand::Copy(_), _)))
        .unwrap()
        .location;
    reject("same-type wrong read owner", &|b| {
        let Rvalue::Cast(_, Operand::Copy(place), _) = value(b, cast) else {
            unreachable!()
        };
        place.local = other_read;
    });
    reject("borrow cloned owner instead of original", &|b| {
        let Rvalue::Ref(_, _, place) = value(b, initial.location()) else {
            unreachable!()
        };
        *place = mir::Place::from(cloned);
    });
    reject("borrow argument has wrong reference type", &|b| {
        b.local_decls[argument.reference()].ty = tcx.types.i32
    });
    reject("copy replaces moved reference argument", &|b| {
        let TerminatorKind::Call { args, .. } = term(b, at) else {
            unreachable!()
        };
        args[0].node = Operand::Copy(mir::Place::from(argument.reference()));
    });
    reject("extra shared-reference stage", &|b| {
        let fresh = b
            .local_decls
            .push(b.local_decls[argument.reference()].clone());
        let mut statement = b.basic_blocks[argument.location().block].statements
            [argument.location().statement_index]
            .clone();
        let mir::StatementKind::Assign(pair) = &mut statement.kind else {
            unreachable!()
        };
        pair.0 = mir::Place::from(fresh);
        b.basic_blocks.as_mut()[at.block].statements.push(statement);
        let TerminatorKind::Call { args, .. } = term(b, at) else {
            unreachable!()
        };
        args[0].node = Operand::Move(mir::Place::from(fresh));
    });
    for step in &steps {
        reject("fake instead of shared borrow", &|b| {
            let Rvalue::Ref(_, kind, _) = value(b, step.location()) else {
                unreachable!()
            };
            *kind = mir::BorrowKind::Fake(mir::FakeBorrowKind::Deep);
        });
        reject("stage destination identity", &|b| {
            let fresh = b.local_decls.push(b.local_decls[step.reference()].clone());
            let mir::StatementKind::Assign(pair) = &mut b.basic_blocks.as_mut()
                [step.location().block]
                .statements[step.location().statement_index]
                .kind
            else {
                unreachable!()
            };
            pair.0 = mir::Place::from(fresh);
        });
    }
    if steps.len() == 2 {
        reject("reborrow replaced by direct borrow", &|b| {
            *value(b, argument.location()) = value(b, initial.location()).clone();
        });
        reject("reversed reference stages", &|b| {
            let first = b.basic_blocks[initial.location().block].statements
                [initial.location().statement_index]
                .clone();
            let last = b.basic_blocks[argument.location().block].statements
                [argument.location().statement_index]
                .clone();
            b.basic_blocks.as_mut()[initial.location().block].statements
                [initial.location().statement_index] = last;
            b.basic_blocks.as_mut()[argument.location().block].statements
                [argument.location().statement_index] = first;
        });
    }
    let constructor = trace.calls[0].0;
    let scalar = trace.assignments.iter().find(|a| matches!(a.value, Rvalue::Use(Operand::Copy(p), _) if *p == mir::Place::from(matched.parameter))).unwrap();
    reject("constructor scalar moved not copied", &|b| {
        let Rvalue::Use(op, _) = value(b, scalar.location) else {
            unreachable!()
        };
        *op = Operand::Move(mir::Place::from(matched.parameter));
    });
    reject("constructor scalar stage omitted", &|b| {
        b.basic_blocks.as_mut()[scalar.location.block].statements
            [scalar.location.statement_index]
            .kind = mir::StatementKind::Nop;
        let TerminatorKind::Call { args, .. } = term(b, constructor) else {
            unreachable!()
        };
        args[0].node = Operand::Copy(mir::Place::from(matched.parameter));
    });
    assert_eq!(plan.cloning.box_ty(), plan.construction.result());
}
