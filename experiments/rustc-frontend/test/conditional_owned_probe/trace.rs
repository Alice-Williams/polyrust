//! Independent test-only path traversal, including constant-defined cleanup flags.
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::{HashMap, HashSet};

pub(super) struct Decision {
    pub local: mir::Local,
    pub definition: mir::Location,
    pub location: mir::Location,
    pub value: bool,
}
pub(super) struct Trace<'a, 'tcx> {
    pub assignments: Vec<(mir::Location, mir::Place<'tcx>, &'a Rvalue<'tcx>)>,
    pub calls: Vec<(mir::Location, &'a TerminatorKind<'tcx>)>,
    pub drops: Vec<(mir::Location, mir::Place<'tcx>)>,
    pub decisions: Vec<Decision>,
    pub guard: Option<(mir::Location, mir::Location, mir::Local)>,
    pub returning: mir::Location,
    pub visited: HashSet<mir::BasicBlock>,
    order: Vec<mir::Location>,
}
impl<'a, 'tcx> Trace<'a, 'tcx> {
    pub fn read(
        tcx: TyCtxt<'tcx>,
        body: &'a mir::Body<'tcx>,
        guard_parameter: mir::Local,
        choice: bool,
    ) -> Self {
        assert!(body.basic_blocks.len() <= 128 && body.local_decls.len() <= 256);
        let mut trace = Self {
            assignments: vec![],
            calls: vec![],
            drops: vec![],
            decisions: vec![],
            guard: None,
            returning: mir::Location {
                block: mir::START_BLOCK,
                statement_index: 0,
            },
            visited: HashSet::new(),
            order: vec![],
        };
        let mut flags = HashMap::new();
        let mut block = mir::START_BLOCK;
        loop {
            assert!(trace.visited.insert(block), "acyclic normal path");
            let data = &body.basic_blocks[block];
            assert!(!data.is_cleanup);
            for (index, statement) in data.statements.iter().enumerate() {
                let at = mir::Location {
                    block,
                    statement_index: index,
                };
                trace.order.push(at);
                match &statement.kind {
                    StatementKind::Assign(pair) => {
                        assert!(pair.0.projection.is_empty());
                        trace.assignments.push((at, pair.0, &pair.1));
                        flags.remove(&pair.0.local);
                        if let Rvalue::Use(Operand::Constant(value), _) = &pair.1
                            && value.const_.ty() == tcx.types.bool
                        {
                            let flag = value
                                .const_
                                .try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized())
                                .unwrap();
                            assert_eq!(body.local_decls[pair.0.local].ty, tcx.types.bool);
                            flags.insert(pair.0.local, (at, flag));
                        }
                    }
                    StatementKind::StorageLive(_)
                    | StatementKind::StorageDead(_)
                    | StatementKind::Nop => {}
                    other => panic!("unexpected statement {other:?}"),
                }
            }
            let at = mir::Location {
                block,
                statement_index: data.statements.len(),
            };
            trace.order.push(at);
            match &data.terminator().kind {
                call @ TerminatorKind::Call {
                    target: Some(target),
                    unwind: mir::UnwindAction::Unreachable,
                    ..
                } => {
                    trace.calls.push((at, call));
                    block = *target;
                }
                TerminatorKind::Drop {
                    place,
                    target,
                    unwind: mir::UnwindAction::Unreachable,
                    drop: None,
                    ..
                } => {
                    trace.drops.push((at, *place));
                    block = *target;
                }
                TerminatorKind::Goto { target } => block = *target,
                TerminatorKind::SwitchInt { discr, targets } => {
                    let (Operand::Copy(place) | Operand::Move(place)) = discr else {
                        panic!("decision operand")
                    };
                    assert!(place.projection.is_empty());
                    assert_eq!(body.local_decls[place.local].ty, tcx.types.bool);
                    assert_eq!(targets.all_targets().len(), 2);
                    assert_ne!(targets.target_for_value(0), targets.target_for_value(1));
                    let value = if let Some(&(definition, value)) = flags.get(&place.local) {
                        assert!(matches!(discr, Operand::Copy(_)));
                        assert!(trace.guard.is_some(), "cleanup follows source decision");
                        trace.before(definition, at);
                        trace.decisions.push(Decision {
                            local: place.local,
                            definition,
                            location: at,
                            value,
                        });
                        value
                    } else {
                        assert!(trace.guard.is_none(), "one source decision");
                        assert!(matches!(discr, Operand::Move(_)));
                        let (definition, value) = trace.definition(*place);
                        let Rvalue::Use(Operand::Copy(parameter), _) = value else {
                            panic!("source guard copy")
                        };
                        assert_eq!(*parameter, mir::Place::from(guard_parameter));
                        trace.before(definition, at);
                        trace.guard = Some((at, definition, place.local));
                        choice
                    };
                    block = targets.target_for_value(u128::from(value));
                }
                TerminatorKind::Return => {
                    trace.returning = at;
                    break;
                }
                other => panic!("unexpected terminator {other:?}"),
            }
        }
        assert!(trace.guard.is_some());
        trace
    }
    pub fn definition(&self, place: mir::Place<'tcx>) -> (mir::Location, &'a Rvalue<'tcx>) {
        let found: Vec<_> = self
            .assignments
            .iter()
            .filter(|(_, destination, _)| *destination == place)
            .collect();
        assert_eq!(found.len(), 1, "unique definition of {place:?}");
        (found[0].0, found[0].2)
    }
    pub fn moved(&self, source: mir::Place<'tcx>) -> (mir::Location, mir::Place<'tcx>) {
        let found: Vec<_> = self.assignments.iter().filter(|(_, _, value)| matches!(value, Rvalue::Use(Operand::Move(actual), _) if *actual == source)).collect();
        assert_eq!(found.len(), 1, "unique move from {source:?}");
        (found[0].0, found[0].1)
    }
    pub fn before(&self, first: mir::Location, second: mir::Location) {
        assert!(
            self.order.iter().position(|at| *at == first).unwrap()
                < self.order.iter().position(|at| *at == second).unwrap()
        );
    }
}
