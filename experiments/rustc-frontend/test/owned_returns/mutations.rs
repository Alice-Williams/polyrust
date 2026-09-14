//! Corrupt source exit claims separately from the MIR ownership relation.
use super::super::super::{exits, flow, scopes};
use super::{Claims, certify, relations, source};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let plan = source::read_return(tcx, owner).unwrap();
    let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let matched = relations::validate(tcx, owner, &plan, &body).unwrap();
    let trace = flow::trace(&body).unwrap();
    assert_eq!(matched.chains.len(), 2);
    assert_eq!(plan.scopes.blocks().count(), 3);
    let exits::Exit::Return { expression, value } = plan.scopes.exit() else {
        panic!("return")
    };
    let fresh = || Claims {
        expression,
        value,
        scope: plan.scopes.read_scope(),
        returning: matched.returning,
    };
    assert!(certify(&plan, &body, fresh()).is_ok());
    let mut count = 0;
    let mut reject = |edit: &dyn Fn(&mut Claims<'tcx>)| {
        let mut claims = fresh();
        edit(&mut claims);
        assert!(certify(&plan, &body, claims).is_err());
        count += 1;
    };
    reject(&|c| c.expression = value);
    reject(&|c| c.value = expression);
    reject(&|c| c.scope = plan.scopes.blocks().next().unwrap().0);
    reject(&|c| c.returning = matched.read);
    reject(&|c| c.returning = matched.chains[0].drop);
    let containment = || scopes::ContainmentClaims {
        blocks: plan.scopes.blocks().collect(),
        bindings: plan.scopes.bindings().to_vec(),
        read: plan.scopes.read_scope(),
    };
    for changed in [
        exits::Exit::Return {
            expression: value,
            value,
        },
        exits::Exit::Return {
            expression,
            value: expression,
        },
    ] {
        assert!(scopes::certify_exit(tcx, owner, containment(), changed).is_err());
        count += 1;
    }
    let mut changed_scope = containment();
    changed_scope.bindings[0].1 = changed_scope.read;
    assert!(scopes::certify_exit(tcx, owner, changed_scope, plan.scopes.exit()).is_err());
    count += 1;
    let mut reject_body = |edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = body.clone();
        edit(&mut changed);
        assert!(relations::validate(tcx, owner, &plan, &changed).is_err());
        count += 1;
    };
    reject_body(&|b| {
        b.basic_blocks.as_mut()[matched.returning.block]
            .terminator_mut()
            .kind = TerminatorKind::Goto {
            target: mir::START_BLOCK,
        }
    });
    reject_body(&|b| {
        let term = b.basic_blocks.as_mut()[matched.chains[0].drop.block].terminator_mut();
        let TerminatorKind::Drop { target, .. } = term.kind else {
            panic!("drop")
        };
        term.kind = TerminatorKind::Goto { target };
    });
    reject_body(&|b| {
        let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
            [matched.chains[0].drop.block]
            .terminator_mut()
            .kind
        else {
            panic!("drop")
        };
        *place = mir::Place::from(*matched.chains[1].owners.last().unwrap());
    });
    let cast = trace
        .assignments
        .iter()
        .find(|a| matches!(a.value, Rvalue::Cast(..)))
        .unwrap()
        .location;
    reject_body(&|b| {
        let StatementKind::Assign(pair) =
            &mut b.basic_blocks.as_mut()[cast.block].statements[cast.statement_index].kind
        else {
            panic!("cast")
        };
        let Rvalue::Cast(_, Operand::Copy(place), _) = &mut pair.1 else {
            panic!("place")
        };
        assert_eq!(place.local, *matched.chains[1].owners.last().unwrap());
        place.local = *matched.chains[0].owners.last().unwrap();
    });
    reject_body(&|b| {
        let TerminatorKind::Drop { target, drop, .. } = &mut b.basic_blocks.as_mut()
            [matched.chains[0].drop.block]
            .terminator_mut()
            .kind
        else {
            panic!("drop")
        };
        assert!(drop.is_none());
        *drop = Some(*target);
    });
    reject_body(&|b| {
        let extra = b.basic_blocks[matched.chains[0].drop.block]
            .terminator()
            .kind
            .clone();
        b.basic_blocks.as_mut()[matched.returning.block]
            .terminator_mut()
            .kind = extra;
    });
    assert_eq!(count, 14);
    println!("14 explicit return identity, scope and cleanup corruptions rejected");
}
