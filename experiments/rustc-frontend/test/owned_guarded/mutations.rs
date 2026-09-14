//! Whole private MIR substitutions, plus behavior-preserving sharing controls.
use super::{GuardedOwnedBody, flow};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Place, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let proof = GuardedOwnedBody::from_body(tcx, owner, &original).unwrap();
    let [no, yes] = flow::split(&original).unwrap();
    assert_eq!(no.drops.len(), 2);
    assert_eq!(yes.drops.len(), 2);
    let branch = no.branch.as_ref().unwrap();
    let switch = branch.location.block;
    let parameter = proof.guard().parameter().1;
    let constant = no
        .assignments
        .iter()
        .find_map(|a| match a.value {
            Rvalue::Use(Operand::Constant(value), _) if value.const_.ty() == tcx.types.bool => {
                Some(value.clone())
            }
            _ => None,
        })
        .expect("real Boolean compiler bookkeeping");
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            GuardedOwnedBody::from_body(tcx, owner, &changed).is_err(),
            "admitted guarded mutation: {name}"
        );
        cases += 1;
    };
    reject("wrong phase", &|b| b.phase = mir::MirPhase::Built);
    reject("wrong owner", &|b| {
        let other = tcx
            .hir_body_owners()
            .find(|id| tcx.def_path_str(*id) == "choose")
            .unwrap();
        b.source = tcx
            .mir_drops_elaborated_and_const_checked(other)
            .borrow()
            .source;
    });
    reject("wrong signature", &|b| b.arg_count -= 1);
    reject("wrong Boolean parameter type", &|b| {
        b.local_decls[parameter].ty = tcx.types.i32
    });
    reject("constant discriminator", &|b| {
        let TerminatorKind::SwitchInt { discr, .. } =
            &mut b.basic_blocks.as_mut()[switch].terminator_mut().kind
        else {
            panic!("switch")
        };
        *discr = Operand::Constant(constant.clone());
    });
    reject("wrong parameter discriminator", &|b| {
        let other = b.args_iter().find(|local| *local != parameter).unwrap();
        let TerminatorKind::SwitchInt { discr, .. } =
            &mut b.basic_blocks.as_mut()[switch].terminator_mut().kind
        else {
            panic!("switch")
        };
        *discr = Operand::Copy(Place::from(other));
    });
    reject("negated producer", &|b| {
        add_guard_copy(b, switch, parameter, true)
    });
    reject("swapped successors", &|b| {
        let TerminatorKind::SwitchInt { targets, .. } =
            &mut b.basic_blocks.as_mut()[switch].terminator_mut().kind
        else {
            panic!("switch")
        };
        *targets = mir::SwitchTargets::new(
            [(0, yes.branch.as_ref().unwrap().target)].into_iter(),
            branch.target,
        );
    });
    reject("missing branch", &|b| {
        let TerminatorKind::SwitchInt { targets, .. } =
            &mut b.basic_blocks.as_mut()[switch].terminator_mut().kind
        else {
            panic!("switch")
        };
        *targets = mir::SwitchTargets::new([(0, branch.target)].into_iter(), branch.target);
    });
    reject("extra successor", &|b| {
        let TerminatorKind::SwitchInt { targets, .. } =
            &mut b.basic_blocks.as_mut()[switch].terminator_mut().kind
        else {
            panic!("switch")
        };
        *targets = mir::SwitchTargets::new(
            [(0, branch.target), (1, yes.branch.as_ref().unwrap().target)].into_iter(),
            branch.target,
        );
    });
    reject("cycle", &|b| {
        let TerminatorKind::SwitchInt { targets, .. } =
            &mut b.basic_blocks.as_mut()[switch].terminator_mut().kind
        else {
            panic!("switch")
        };
        *targets = mir::SwitchTargets::new(
            [(0, switch)].into_iter(),
            yes.branch.as_ref().unwrap().target,
        );
    });
    let cast = no
        .assignments
        .iter()
        .find(|a| matches!(a.value, Rvalue::Cast(..)))
        .unwrap()
        .location;
    reject("wrong false owner", &|b| {
        let StatementKind::Assign(pair) =
            &mut b.basic_blocks.as_mut()[cast.block].statements[cast.statement_index].kind
        else {
            panic!("cast")
        };
        let Rvalue::Cast(_, Operand::Copy(place), _) = &mut pair.1 else {
            panic!("cast place")
        };
        let replacement = proof.paths()[1]
            .chains()
            .iter()
            .find(|c| c.bindings().last().unwrap().0 == proof.paths()[1].scalar_read().0)
            .unwrap()
            .bindings()
            .last()
            .unwrap()
            .1;
        assert_ne!(place.local, replacement);
        place.local = replacement;
    });
    reject("missing cleanup", &|b| {
        let term = b.basic_blocks.as_mut()[no.drops[0].0.block].terminator_mut();
        let TerminatorKind::Drop { target, .. } = term.kind else {
            panic!("drop")
        };
        term.kind = TerminatorKind::Goto { target };
    });
    reject("duplicate cleanup owner", &|b| {
        let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()[no.drops[0].0.block]
            .terminator_mut()
            .kind
        else {
            panic!("drop")
        };
        *place = no.drops[1].1;
    });
    reject("reordered cleanup", &|b| {
        for (index, (location, _)) in no.drops.iter().enumerate() {
            let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()[location.block]
                .terminator_mut()
                .kind
            else {
                panic!("drop")
            };
            *place = no.drops[1 - index].1;
        }
    });
    reject("async cleanup", &|b| {
        let TerminatorKind::Drop { drop, target, .. } = &mut b.basic_blocks.as_mut()
            [no.drops[0].0.block]
            .terminator_mut()
            .kind
        else {
            panic!("drop")
        };
        assert!(drop.is_none());
        *drop = Some(*target);
    });
    reject("unwind cleanup", &|b| {
        let TerminatorKind::Drop { unwind, .. } = &mut b.basic_blocks.as_mut()[no.drops[0].0.block]
            .terminator_mut()
            .kind
        else {
            panic!("drop")
        };
        *unwind = mir::UnwindAction::Continue;
    });
    reject("cleanup block", &|b| {
        b.basic_blocks.as_mut()[no.drops[0].0.block].is_cleanup = true
    });
    reject("extra switch", &|b| {
        let extra = b.basic_blocks[switch].terminator().kind.clone();
        b.basic_blocks.as_mut()[no.drops[0].0.block]
            .terminator_mut()
            .kind = extra;
    });
    reject("unreachable block", &|b| {
        let extra = b.basic_blocks[no.returning.block].clone();
        b.basic_blocks.as_mut().push(extra);
    });
    reject("missing return", &|b| {
        b.basic_blocks.as_mut()[no.returning.block]
            .terminator_mut()
            .kind = TerminatorKind::Goto {
            target: mir::START_BLOCK,
        }
    });
    assert_eq!(cases, 21);
    let mut copied = original.clone();
    add_guard_copy(&mut copied, switch, parameter, false);
    assert!(GuardedOwnedBody::from_body(tcx, owner, &copied).is_ok());
    shared_cleanup(tcx, owner, &original, &no, &yes);
    println!(
        "22 guarded corruptions rejected; guard copies and shared cleanup preserve correspondence"
    );
}

fn add_guard_copy<'tcx>(
    body: &mut mir::Body<'tcx>,
    switch: mir::BasicBlock,
    parameter: mir::Local,
    negate: bool,
) {
    let temporary = body.local_decls.push(body.local_decls[parameter].clone());
    let original = match &body.basic_blocks[switch].terminator().kind {
        TerminatorKind::SwitchInt { discr, .. } => discr.clone(),
        _ => panic!("switch"),
    };
    let mut statement = body
        .basic_blocks
        .iter()
        .flat_map(|b| &b.statements)
        .find(|s| matches!(&s.kind, StatementKind::Assign(p) if matches!(p.1, Rvalue::Use(..))))
        .unwrap()
        .clone();
    let StatementKind::Assign(pair) = &mut statement.kind else {
        unreachable!()
    };
    pair.0 = Place::from(temporary);
    if negate {
        pair.1 = Rvalue::UnaryOp(mir::UnOp::Not, original);
    } else {
        let Rvalue::Use(operand, _) = &mut pair.1 else {
            unreachable!()
        };
        *operand = original;
    }
    body.basic_blocks.as_mut()[switch]
        .statements
        .push(statement);
    let TerminatorKind::SwitchInt { discr, .. } =
        &mut body.basic_blocks.as_mut()[switch].terminator_mut().kind
    else {
        unreachable!()
    };
    *discr = Operand::Copy(Place::from(temporary));
}

fn shared_cleanup<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    body: &mir::Body<'tcx>,
    no: &flow::Trace<'_, 'tcx>,
    yes: &flow::Trace<'_, 'tcx>,
) {
    let &(left, place) = no.drops.last().unwrap();
    let &(right, same_place) = yes.drops.last().unwrap();
    assert_eq!(place, same_place);
    // The pinned compiler already shares this suffix. Require that real shape
    // rather than manufacturing a sharing rewrite which might be a no-op.
    assert_eq!(left.block, right.block);
    assert_eq!(
        left.statement_index,
        body.basic_blocks[left.block].statements.len()
    );
    let mut shared = body.clone();
    let proof = GuardedOwnedBody::from_body(tcx, owner, &shared).unwrap();
    for path in proof.paths() {
        let last = path.drop_order().last().unwrap();
        let chain = path
            .chains()
            .iter()
            .find(|c| c.bindings().last().unwrap().0 == *last)
            .unwrap();
        assert_eq!(chain.drop_location(), left);
    }
    let term = shared.basic_blocks.as_mut()[left.block].terminator_mut();
    let TerminatorKind::Drop { target, .. } = term.kind else {
        unreachable!()
    };
    term.kind = TerminatorKind::Goto { target };
    assert!(
        GuardedOwnedBody::from_body(tcx, owner, &shared).is_err(),
        "missing shared cleanup admitted"
    );
}
