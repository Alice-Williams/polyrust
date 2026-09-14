//! Walk actual MIR independently, retaining latest constant flag definitions.
use crate::owned_linear::multiple::selection::SelectedOwnedBody;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::{HashMap, HashSet};

pub(super) fn check<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    proof: &SelectedOwnedBody<'tcx>,
    index: usize,
) {
    let path = &proof.paths()[index];
    let mut block = mir::START_BLOCK;
    let mut visited = HashSet::new();
    let mut flags = HashMap::new();
    let mut drops = Vec::new();
    let mut switches = 0;
    loop {
        assert!(visited.insert(block));
        for (statement_index, statement) in body.basic_blocks[block].statements.iter().enumerate() {
            if let StatementKind::Assign(pair) = &statement.kind
                && let Rvalue::Use(Operand::Constant(value), _) = &pair.1
                && value.const_.ty() == tcx.types.bool
            {
                flags.insert(
                    pair.0.local,
                    (
                        value
                            .const_
                            .try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized())
                            .unwrap(),
                        mir::Location {
                            block,
                            statement_index,
                        },
                    ),
                );
            }
        }
        match &body.basic_blocks[block].terminator().kind {
            TerminatorKind::Goto { target }
            | TerminatorKind::Call {
                target: Some(target),
                ..
            } => block = *target,
            TerminatorKind::SwitchInt { discr, targets } => {
                let value = if switches == 0 {
                    assert_eq!(block, proof.guard().block);
                    index as u128
                } else {
                    let Operand::Copy(place) = discr else {
                        panic!("cleanup operand")
                    };
                    let &(value, definition) = flags.get(&place.local).unwrap();
                    let evidence = &path.decisions()[switches - 1];
                    assert_eq!(evidence.local(), place.local);
                    assert_eq!(evidence.definition(), definition);
                    assert_eq!(
                        evidence.location(),
                        mir::Location {
                            block,
                            statement_index: body.basic_blocks[block].statements.len()
                        }
                    );
                    assert_eq!(evidence.value(), value);
                    assert_eq!(
                        evidence.target(),
                        targets.target_for_value(u128::from(value))
                    );
                    u128::from(value)
                };
                switches += 1;
                block = targets.target_for_value(value);
            }
            TerminatorKind::Drop { place, target, .. } => {
                drops.push(place.local);
                block = *target;
            }
            TerminatorKind::Return => {
                assert_eq!(block, path.returning().block);
                break;
            }
            _ => panic!("unexpected terminator"),
        }
    }
    assert_eq!(switches, 3);
    assert_eq!(path.decisions().len(), 2);
    assert_eq!(drops.len(), path.chains().len());
    assert_eq!(
        drops,
        path.drop_order()
            .iter()
            .map(|id| path
                .chains()
                .iter()
                .find(|c| c.bindings().last().unwrap().0 == *id)
                .unwrap()
                .bindings()
                .last()
                .unwrap()
                .1)
            .collect::<Vec<_>>()
    );
    for (a, b) in proof.paths()[0]
        .decisions()
        .iter()
        .zip(proof.paths()[1].decisions())
    {
        assert_eq!(a.local(), b.local());
        assert_eq!(a.location(), b.location());
        assert_ne!(a.value(), b.value());
    }
}
