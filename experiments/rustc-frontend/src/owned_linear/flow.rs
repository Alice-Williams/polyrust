//! Complete bounded normal-flow inventory; no source-name or span matching.
use super::{LinearError as Error, Result};
use rustc_middle::mir::{self, StatementKind, TerminatorKind, UnwindAction};
use std::collections::HashSet;

pub(super) struct Assignment<'a, 'tcx> {
    pub location: mir::Location,
    pub destination: mir::Place<'tcx>,
    pub value: &'a mir::Rvalue<'tcx>,
}

pub(super) struct Flow<'a, 'tcx> {
    pub assignments: Vec<Assignment<'a, 'tcx>>,
    pub call: (mir::Location, &'a TerminatorKind<'tcx>),
    pub drop: (mir::Location, mir::Place<'tcx>),
    pub returning: mir::Location,
    events: Vec<mir::Location>,
}

impl<'a, 'tcx> Flow<'a, 'tcx> {
    pub fn before(&self, first: mir::Location, second: mir::Location) -> bool {
        let first = self.events.iter().position(|event| *event == first);
        let second = self.events.iter().position(|event| *event == second);
        matches!((first, second), (Some(a), Some(b)) if a < b)
    }
    pub fn definition(&self, local: mir::Local) -> Result<&Assignment<'a, 'tcx>> {
        let mut matches = self
            .assignments
            .iter()
            .filter(|a| a.destination.local == local);
        let value = matches.next().ok_or(Error::Assignment)?;
        if matches.next().is_some() {
            return Err(Error::Assignment);
        }
        Ok(value)
    }
}

pub(super) fn read<'a, 'tcx>(body: &'a mir::Body<'tcx>) -> Result<Flow<'a, 'tcx>> {
    if body.basic_blocks.len() > 512 || body.local_decls.len() > 1024 {
        return Err(Error::Budget);
    }
    let mut visited = HashSet::new();
    let mut assignments = Vec::new();
    let mut events = Vec::new();
    let mut call = None;
    let mut drop = None;
    let mut block = mir::START_BLOCK;
    let returning = loop {
        if !visited.insert(block) {
            return Err(Error::ControlFlow);
        }
        let data = body.basic_blocks.get(block).ok_or(Error::ControlFlow)?;
        if data.is_cleanup {
            return Err(Error::ControlFlow);
        }
        for (statement_index, statement) in data.statements.iter().enumerate() {
            let location = mir::Location {
                block,
                statement_index,
            };
            match &statement.kind {
                StatementKind::Assign(pair) => {
                    if !pair.0.projection.is_empty() {
                        return Err(Error::Assignment);
                    }
                    assignments.push(Assignment {
                        location,
                        destination: pair.0,
                        value: &pair.1,
                    });
                    events.push(location);
                }
                StatementKind::StorageLive(_)
                | StatementKind::StorageDead(_)
                | StatementKind::Nop => {}
                _ => return Err(Error::Assignment),
            }
        }
        let location = mir::Location {
            block,
            statement_index: data.statements.len(),
        };
        events.push(location);
        match &data.terminator().kind {
            TerminatorKind::Goto { target } => block = *target,
            kind @ TerminatorKind::Call {
                target: Some(target),
                unwind: UnwindAction::Unreachable,
                ..
            } => {
                if call.replace((location, kind)).is_some() {
                    return Err(Error::Call);
                }
                block = *target;
            }
            TerminatorKind::Drop {
                place,
                target,
                unwind: UnwindAction::Unreachable,
                ..
            } => {
                if drop.replace((location, *place)).is_some() {
                    return Err(Error::Drop);
                }
                block = *target;
            }
            TerminatorKind::Return => break location,
            _ => return Err(Error::ControlFlow),
        }
    };
    if visited.len() != body.basic_blocks.len() {
        return Err(Error::ControlFlow);
    }
    Ok(Flow {
        assignments,
        call: call.ok_or(Error::Call)?,
        drop: drop.ok_or(Error::Drop)?,
        returning,
        events,
    })
}
