//! Test-only normal-path collection, never a checked input for production.
use rustc_middle::mir::{self, Rvalue, StatementKind, TerminatorKind};
use std::collections::HashSet;

pub(super) struct Trace<'a, 'tcx> {
    pub assignments: Vec<(mir::Location, mir::Place<'tcx>, &'a Rvalue<'tcx>)>,
    pub calls: Vec<(mir::Location, &'a TerminatorKind<'tcx>)>,
    pub drops: Vec<(mir::Location, mir::Place<'tcx>)>,
    pub returning: mir::Location,
    order: Vec<mir::Location>,
}
impl<'a, 'tcx> Trace<'a, 'tcx> {
    pub fn read(body: &'a mir::Body<'tcx>) -> Self {
        let mut result = Self {
            assignments: vec![],
            calls: vec![],
            drops: vec![],
            returning: mir::Location {
                block: mir::START_BLOCK,
                statement_index: 0,
            },
            order: vec![],
        };
        let mut block = mir::START_BLOCK;
        let mut seen = HashSet::new();
        loop {
            assert!(seen.insert(block));
            let data = &body.basic_blocks[block];
            assert!(!data.is_cleanup);
            for (index, statement) in data.statements.iter().enumerate() {
                let at = mir::Location {
                    block,
                    statement_index: index,
                };
                result.order.push(at);
                match &statement.kind {
                    StatementKind::Assign(pair) => result.assignments.push((at, pair.0, &pair.1)),
                    StatementKind::StorageLive(_)
                    | StatementKind::StorageDead(_)
                    | StatementKind::Nop => (),
                    other => panic!("unexpected statement {other:?}"),
                }
            }
            let at = mir::Location {
                block,
                statement_index: data.statements.len(),
            };
            result.order.push(at);
            match &data.terminator().kind {
                call @ TerminatorKind::Call {
                    target: Some(target),
                    unwind: mir::UnwindAction::Unreachable,
                    ..
                } => {
                    result.calls.push((at, call));
                    block = *target;
                }
                TerminatorKind::Drop {
                    place,
                    target,
                    unwind: mir::UnwindAction::Unreachable,
                    drop: None,
                    ..
                } => {
                    result.drops.push((at, *place));
                    block = *target;
                }
                TerminatorKind::Goto { target } => block = *target,
                TerminatorKind::Return => {
                    result.returning = at;
                    break;
                }
                other => panic!("unexpected terminator {other:?}"),
            }
        }
        assert_eq!(seen.len(), body.basic_blocks.len());
        result
    }
    pub fn definition(&self, place: mir::Place<'tcx>) -> (mir::Location, &'a Rvalue<'tcx>) {
        let matching: Vec<_> = self
            .assignments
            .iter()
            .filter(|(_, destination, _)| *destination == place)
            .collect();
        assert_eq!(matching.len(), 1, "unique definition of {place:?}");
        (matching[0].0, matching[0].2)
    }
    pub fn before(&self, first: mir::Location, second: mir::Location) {
        assert!(
            self.order.iter().position(|at| *at == first).unwrap()
                < self.order.iter().position(|at| *at == second).unwrap()
        );
    }
    pub fn moved(&self, source: mir::Place<'tcx>) -> (mir::Location, mir::Place<'tcx>) {
        let matching: Vec<_> = self.assignments.iter().filter(|(_, _, value)| matches!(value, Rvalue::Use(mir::Operand::Move(actual), _) if *actual == source)).collect();
        assert_eq!(matching.len(), 1, "unique move from {source:?}");
        (matching[0].0, matching[0].1)
    }
}
