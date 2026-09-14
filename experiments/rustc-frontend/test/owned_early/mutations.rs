//! Private evidence substitutions cannot bypass canonical source/cleanup checks.
use super::super::super::{exits, flow, scopes};
use super::EarlyOwnedBody;
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};

#[path = "residual.rs"]
mod residual;

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let proof = EarlyOwnedBody::from_body(tcx, owner, &original).unwrap();
    let [no, yes] = flow::split(&original).unwrap();
    assert_eq!(no.drops.len(), 2);
    assert_eq!(yes.drops.len(), 2);
    // Unlike the if/else fixture, early-return cleanup is duplicated, but the
    // terminal Return block is shared. Require the actual compiler shape.
    assert_ne!(no.drops[1].0, yes.drops[1].0);
    assert_eq!(no.returning, yes.returning);
    let switch = no.branch.as_ref().unwrap().location.block;
    let mut count = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            EarlyOwnedBody::from_body(tcx, owner, &changed).is_err(),
            "admitted early-return mutation: {name}"
        );
        count += 1;
    };
    reject("phase", &|b| b.phase = mir::MirPhase::Built);
    reject("signature", &|b| b.arg_count -= 1);
    reject("parameter type", &|b| {
        b.local_decls[proof.guard().parameter().1].ty = tcx.types.i32;
    });
    reject("wrong source owner", &|b| {
        let other = tcx
            .hir_body_owners()
            .find(|id| tcx.def_path_str(*id) == "swapped")
            .unwrap();
        b.source = tcx
            .mir_drops_elaborated_and_const_checked(other)
            .borrow()
            .source;
    });
    reject("negated guard", &|b| {
        let local = match &b.basic_blocks[switch].terminator().kind {
            TerminatorKind::SwitchInt {
                discr: Operand::Copy(place) | Operand::Move(place),
                ..
            } => place.local,
            _ => panic!("copied guard"),
        };
        let statement = b.basic_blocks.as_mut()[switch]
            .statements
            .iter_mut()
            .find(|s| matches!(&s.kind, StatementKind::Assign(p) if p.0.local == local))
            .unwrap();
        let StatementKind::Assign(pair) = &mut statement.kind else {
            unreachable!()
        };
        let Rvalue::Use(operand, _) = &pair.1 else {
            panic!("guard producer")
        };
        pair.1 = Rvalue::UnaryOp(mir::UnOp::Not, operand.clone());
    });
    for (name, left, right) in [
        (
            "swapped",
            yes.branch.as_ref().unwrap().target,
            no.branch.as_ref().unwrap().target,
        ),
        (
            "omitted",
            no.branch.as_ref().unwrap().target,
            no.branch.as_ref().unwrap().target,
        ),
        ("cycle", switch, yes.branch.as_ref().unwrap().target),
    ] {
        reject(name, &|b| {
            let TerminatorKind::SwitchInt { targets, .. } =
                &mut b.basic_blocks.as_mut()[switch].terminator_mut().kind
            else {
                unreachable!()
            };
            *targets = mir::SwitchTargets::new([(0, left)].into_iter(), right);
        });
    }
    for (index, trace) in [&no, &yes].into_iter().enumerate() {
        let cast = trace
            .assignments
            .iter()
            .find(|a| matches!(a.value, Rvalue::Cast(..)))
            .unwrap()
            .location;
        let other = proof.paths()[1 - index].scalar_read().0;
        let replacement = proof.paths()[1 - index]
            .chains()
            .iter()
            .find(|c| c.bindings().last().unwrap().0 == other)
            .unwrap()
            .bindings()
            .last()
            .unwrap()
            .1;
        reject("wrong selected owner", &|b| {
            let StatementKind::Assign(pair) =
                &mut b.basic_blocks.as_mut()[cast.block].statements[cast.statement_index].kind
            else {
                unreachable!()
            };
            let Rvalue::Cast(_, Operand::Copy(place), _) = &mut pair.1 else {
                panic!("cast")
            };
            assert_ne!(place.local, replacement);
            place.local = replacement;
        });
        reject("missing cleanup", &|b| {
            let term = b.basic_blocks.as_mut()[trace.drops[0].0.block].terminator_mut();
            let TerminatorKind::Drop { target, .. } = term.kind else {
                unreachable!()
            };
            term.kind = TerminatorKind::Goto { target };
        });
        reject("duplicate owner", &|b| {
            let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
                [trace.drops[0].0.block]
                .terminator_mut()
                .kind
            else {
                unreachable!()
            };
            *place = trace.drops[1].1;
        });
        reject("cleanup order", &|b| {
            for (i, (location, _)) in trace.drops.iter().enumerate() {
                let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
                    [location.block]
                    .terminator_mut()
                    .kind
                else {
                    unreachable!()
                };
                *place = trace.drops[1 - i].1;
            }
        });
        reject("async cleanup", &|b| {
            let TerminatorKind::Drop { drop, target, .. } = &mut b.basic_blocks.as_mut()
                [trace.drops[0].0.block]
                .terminator_mut()
                .kind
            else {
                unreachable!()
            };
            *drop = Some(*target);
        });
    }
    reject("missing shared return", &|b| {
        b.basic_blocks.as_mut()[no.returning.block]
            .terminator_mut()
            .kind = TerminatorKind::Goto {
            target: mir::START_BLOCK,
        };
    });
    assert_eq!(count, 19);
    source_claims(tcx, owner);
    residual::check(tcx, owner, &original);
    println!("19 early MIR and 8 lexical-exit corruptions rejected; residual controls passed");
}

fn source_claims(tcx: TyCtxt<'_>, owner: LocalDefId) {
    let plans = [exits::Outcome::False, exits::Outcome::True]
        .map(|outcome| super::super::source::read_early(tcx, owner, outcome).unwrap());
    let mut count = 0;
    for (index, outcome) in [exits::Outcome::False, exits::Outcome::True]
        .into_iter()
        .enumerate()
    {
        let plan = &plans[index];
        let other = &plans[1 - index];
        let fresh = || scopes::ContainmentClaims {
            blocks: plan.scopes.blocks().collect(),
            bindings: plan.scopes.bindings().to_vec(),
            read: plan.scopes.read_scope(),
        };
        assert!(
            scopes::certify_route(
                tcx,
                owner,
                fresh(),
                plan.scopes.exit(),
                exits::Mode::Early(outcome)
            )
            .is_ok()
        );
        let mut wrong_scope = fresh();
        wrong_scope.read = other.scopes.read_scope();
        let mut wrong_blocks = fresh();
        wrong_blocks.blocks = other.scopes.blocks().collect();
        let mut wrong_parent = fresh();
        wrong_parent.blocks[0].1 = Some(plan.scopes.read_scope());
        for claims in [wrong_scope, wrong_blocks, wrong_parent] {
            assert!(
                scopes::certify_route(
                    tcx,
                    owner,
                    claims,
                    plan.scopes.exit(),
                    exits::Mode::Early(outcome)
                )
                .is_err()
            );
            count += 1;
        }
        assert!(
            scopes::certify_route(
                tcx,
                owner,
                fresh(),
                other.scopes.exit(),
                exits::Mode::Early(outcome)
            )
            .is_err()
        );
        count += 1;
    }
    assert_eq!(count, 8);
}
