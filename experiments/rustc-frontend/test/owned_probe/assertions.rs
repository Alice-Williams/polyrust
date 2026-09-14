//! Fixture-only oracle: names select expectations, never production capabilities.
use super::observe::Observation;
use rustc_middle::{
    mir::{self, Operand, Place, ProjectionElem, Rvalue, StatementKind, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::BTreeSet;

pub(super) fn names() -> BTreeSet<String> {
    [
        "straight",
        "conditional",
        "partial",
        "early",
        "shadow",
        "imitation::fake",
        "counterfeit",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub(super) fn check<'tcx>(
    tcx: TyCtxt<'tcx>,
    name: &str,
    body: &mir::Body<'tcx>,
    actual: &Observation,
) {
    let expected = match name {
        "straight" => [2, 1, 1, 0, 0, 1, 1],
        "conditional" => [4, 3, 2, 0, 2, 1, 2],
        "partial" => [4, 3, 2, 1, 0, 1, 2],
        "early" => [1, 0, 2, 0, 1, 1, 1],
        "shadow" => [2, 0, 2, 0, 0, 1, 2],
        "imitation::fake" => [0, 0, 0, 0, 0, 1, 0],
        "counterfeit" => [0, 0, 0, 0, 0, 1, 1],
        _ => panic!("unasserted fixture: {name}"),
    };
    assert_eq!(
        [
            actual.box_locals,
            actual.box_moves,
            actual.box_drops,
            actual.projected_drops,
            actual.switches,
            actual.returns,
            actual.calls,
        ],
        expected,
        "{name}"
    );
    let drops: Vec<_> = body
        .basic_blocks
        .iter_enumerated()
        .filter_map(|(block, data)| {
            if let TerminatorKind::Drop { place, target, .. } = data.terminator().kind {
                Some((block, place, target))
            } else {
                None
            }
        })
        .collect();
    let calls: Vec<_> = body
        .basic_blocks
        .iter()
        .filter_map(|data| {
            if let TerminatorKind::Call {
                func, destination, ..
            } = &data.terminator().kind
            {
                let ty::FnDef(def, _) = func.ty(&body.local_decls, tcx).kind() else {
                    panic!("indirect call")
                };
                Some((*def, *destination))
            } else {
                None
            }
        })
        .collect();
    if name == "counterfeit" {
        assert!(calls[0].0.is_local());
    } else {
        for (callee, destination) in &calls {
            assert!(!callee.is_local());
            let ty::Adt(def, _) = destination.ty(&body.local_decls, tcx).ty.kind() else {
                panic!("not an ADT")
            };
            assert_eq!(Some(def.did()), tcx.lang_items().owned_box());
            assert_eq!(*callee, calls[0].0);
        }
    }
    match name {
        "straight" => assert!(moves(body, calls[0].1, drops[0].1)),
        "conditional" => conditional(tcx, body, &drops, calls[0].1, calls[1].1),
        "partial" => {
            let [ProjectionElem::Field(field, _)] = drops[1].1.projection.as_slice() else {
                panic!("missing remaining field drop")
            };
            assert_eq!(field.as_usize(), 1);
            assert_eq!(drops[0].2, drops[1].0, "moved value drops before remainder");
            assert!(body.basic_blocks.iter().flat_map(|b| &b.statements).any(|s| {
                let StatementKind::Assign(pair) = &s.kind else { return false };
                let (destination, Rvalue::Use(Operand::Move(source), _)) = &**pair else { return false };
                *destination == drops[0].1 && source.local == drops[1].1.local
                    && matches!(source.projection.as_slice(), [ProjectionElem::Field(field, _)] if field.as_usize() == 0)
            }), "first field moves into separately dropped owner");
        }
        "early" => {
            assert_eq!(drops[0].1, calls[0].1);
            assert_eq!(drops[1].1, calls[0].1);
            let branch = body
                .basic_blocks
                .iter()
                .find_map(|b| {
                    if let TerminatorKind::SwitchInt { targets, .. } = &b.terminator().kind {
                        Some(targets)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(
                branch
                    .all_targets()
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([drops[0].0, drops[1].0])
            );
        }
        "shadow" => {
            assert_ne!(calls[0].1, calls[1].1, "shadowed owners remain distinct");
            assert_eq!(drops[0].1, calls[1].1);
            assert_eq!(drops[1].1, calls[0].1);
            assert_eq!(drops[0].2, drops[1].0, "inner owner drops first");
        }
        "imitation::fake" => {
            let local_adts: std::collections::HashSet<_> = body
                .local_decls
                .iter()
                .filter_map(|local| match local.ty.kind() {
                    ty::Adt(def, _) if def.did().is_local() => Some(def.did()),
                    _ => None,
                })
                .collect();
            assert_eq!(local_adts.len(), 1);
            let fake = *local_adts.iter().next().unwrap();
            // Fixture identity/spelling assertion, not capability recognition.
            assert_eq!(tcx.def_path_str(fake), "imitation::Box");
            let standard = tcx.lang_items().owned_box().unwrap();
            assert_eq!(tcx.item_name(fake), tcx.item_name(standard));
            assert_ne!(fake, standard);
        }
        "counterfeit" => {}
        _ => unreachable!(),
    }
}

fn moves<'tcx>(body: &mir::Body<'tcx>, from: Place<'tcx>, to: Place<'tcx>) -> bool {
    body.basic_blocks.iter().flat_map(|b| &b.statements).any(|s| {
        matches!(&s.kind, StatementKind::Assign(pair)
            if matches!(&**pair, (destination, Rvalue::Use(Operand::Move(source), _)) if *destination == to && *source == from))
    })
}

fn conditional<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    drops: &[(mir::BasicBlock, Place<'tcx>, mir::BasicBlock)],
    original: Place<'tcx>,
    alternate: Place<'tcx>,
) {
    assert_eq!(drops[1].1, original);
    assert!(moves(body, alternate, drops[0].1));
    let TerminatorKind::SwitchInt {
        discr: Operand::Copy(flag),
        targets,
    } = &body.basic_blocks[drops[0].2].terminator().kind
    else {
        panic!("missing conditional drop flag")
    };
    assert!(flag.projection.is_empty());
    assert_eq!(body.local_decls[flag.local].ty, tcx.types.bool);
    assert_eq!(
        targets.target_for_value(0),
        drops[1].2,
        "false skips original's drop"
    );
    assert_eq!(targets.otherwise(), drops[1].0, "true drops original");
    let assignments: Vec<_> = body
        .basic_blocks
        .iter()
        .flat_map(|b| &b.statements)
        .filter_map(|s| {
            let StatementKind::Assign(pair) = &s.kind else {
                return None;
            };
            let (destination, Rvalue::Use(Operand::Constant(value), _)) = &**pair else {
                return None;
            };
            if destination != flag {
                return None;
            }
            Some(
                value
                    .const_
                    .try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized())
                    .unwrap(),
            )
        })
        .collect();
    assert_eq!(assignments, [false, true, false, false]);
    // The flag becomes false in the same block that moves the original owner.
    assert!(body.basic_blocks.iter().any(|b| {
        b.statements.iter().any(|s| matches!(&s.kind, StatementKind::Assign(p)
            if matches!(&**p, (_, Rvalue::Use(Operand::Move(source), _)) if *source == original)))
        && b.statements.iter().any(|s| matches!(&s.kind, StatementKind::Assign(p)
            if matches!(&**p, (destination, Rvalue::Use(Operand::Constant(v), _))
                if destination == flag && v.const_.try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized()) == Some(false))))
    }));
}
