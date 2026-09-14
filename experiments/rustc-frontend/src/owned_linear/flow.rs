//! Complete bounded normal-flow inventory; no source-name or span matching.
use super::{LinearError as Error, Result, exits::Outcome};
use rustc_middle::mir::{self, StatementKind, TerminatorKind, UnwindAction};
use std::collections::HashSet;
#[path = "flow/flags.rs"]
pub(super) mod flags;

pub(super) struct Assignment<'a, 'tcx> {
    pub location: mir::Location,
    pub destination: mir::Place<'tcx>,
    pub value: &'a mir::Rvalue<'tcx>,
}

pub(super) struct Flow<'a, 'tcx> {
    pub trace: Trace<'a, 'tcx>,
    pub call: (mir::Location, &'a TerminatorKind<'tcx>),
    pub drop: (mir::Location, mir::Place<'tcx>),
}

pub(super) struct Trace<'a, 'tcx> {
    pub assignments: Vec<Assignment<'a, 'tcx>>,
    pub calls: Vec<(mir::Location, &'a TerminatorKind<'tcx>)>,
    pub drops: Vec<(mir::Location, mir::Place<'tcx>)>,
    pub returning: mir::Location,
    pub branch: Option<Branch<'a, 'tcx>>,
    pub cleanup: Vec<flags::Decision>,
    visited: HashSet<mir::BasicBlock>,
    events: Vec<mir::Location>,
}

pub(super) struct Branch<'a, 'tcx> {
    pub location: mir::Location,
    pub discriminator: &'a mir::Operand<'tcx>,
    pub target: mir::BasicBlock,
    pub outcome: Outcome,
}

impl<'a, 'tcx> std::ops::Deref for Flow<'a, 'tcx> {
    type Target = Trace<'a, 'tcx>;
    fn deref(&self) -> &Self::Target {
        &self.trace
    }
}

impl<'a, 'tcx> Trace<'a, 'tcx> {
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
    let trace = trace(body)?;
    if trace.calls.len() != 1 {
        return Err(Error::Call);
    }
    if trace.drops.len() != 1 {
        return Err(Error::Drop);
    }
    Ok(Flow {
        call: trace.calls[0],
        drop: trace.drops[0],
        trace,
    })
}

pub(super) fn trace<'a, 'tcx>(body: &'a mir::Body<'tcx>) -> Result<Trace<'a, 'tcx>> {
    let trace = inventory(body, None, None)?;
    if trace.visited.len() != body.basic_blocks.len() {
        return Err(Error::ControlFlow);
    }
    Ok(trace)
}

pub(super) fn split<'a, 'tcx>(body: &'a mir::Body<'tcx>) -> Result<[Trace<'a, 'tcx>; 2]> {
    split_with(body, None)
}

pub(super) fn selection<'a, 'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    body: &'a mir::Body<'tcx>,
) -> Result<[Trace<'a, 'tcx>; 2]> {
    let paths = split_with(body, Some(tcx))?;
    if paths.iter().any(|p| p.cleanup.len() != 2) {
        return Err(Error::ControlFlow);
    }
    for (no, yes) in paths[0].cleanup.iter().zip(&paths[1].cleanup) {
        if no.location() != yes.location() || no.local() != yes.local() || no.value() == yes.value()
        {
            return Err(Error::ControlFlow);
        }
    }
    flags::validate(tcx, body, &paths)?;
    Ok(paths)
}

fn split_with<'a, 'tcx>(
    body: &'a mir::Body<'tcx>,
    constants: Option<rustc_middle::ty::TyCtxt<'tcx>>,
) -> Result<[Trace<'a, 'tcx>; 2]> {
    let no = inventory(body, Some(Outcome::False), constants)?;
    let yes = inventory(body, Some(Outcome::True), constants)?;
    let a = no.branch.as_ref().ok_or(Error::ControlFlow)?;
    let b = yes.branch.as_ref().ok_or(Error::ControlFlow)?;
    if a.location != b.location
        || a.target == b.target
        || no.visited.union(&yes.visited).count() != body.basic_blocks.len()
    {
        return Err(Error::ControlFlow);
    }
    Ok([no, yes])
}

fn inventory<'a, 'tcx>(
    body: &'a mir::Body<'tcx>,
    choice: Option<Outcome>,
    constants: Option<rustc_middle::ty::TyCtxt<'tcx>>,
) -> Result<Trace<'a, 'tcx>> {
    if body.basic_blocks.len() > 512 || body.local_decls.len() > 1024 {
        return Err(Error::Budget);
    }
    let mut visited = HashSet::new();
    let mut assignments = Vec::new();
    let mut events = Vec::new();
    let mut calls = Vec::new();
    let mut drops = Vec::new();
    let mut branch = None;
    let mut cleanup = Vec::new();
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
            TerminatorKind::SwitchInt { discr, targets } => {
                let outcome = choice.ok_or(Error::ControlFlow)?;
                if targets.all_targets().len() != 2
                    || targets.target_for_value(0) == targets.target_for_value(1)
                {
                    return Err(Error::ControlFlow);
                }
                if branch.is_some() {
                    let tcx = constants.ok_or(Error::ControlFlow)?;
                    let decision =
                        flags::resolve(tcx, body, &assignments, discr, targets, location)?;
                    block = decision.target();
                    cleanup.push(decision);
                    continue;
                }
                let target = targets.target_for_value(outcome.value());
                branch = Some(Branch {
                    location,
                    discriminator: discr,
                    target,
                    outcome,
                });
                block = target;
            }
            TerminatorKind::Goto { target } => block = *target,
            kind @ TerminatorKind::Call {
                target: Some(target),
                unwind: UnwindAction::Unreachable,
                ..
            } => {
                calls.push((location, kind));
                block = *target;
            }
            TerminatorKind::Drop {
                place,
                target,
                unwind: UnwindAction::Unreachable,
                drop: None,
                ..
            } => {
                drops.push((location, *place));
                block = *target;
            }
            TerminatorKind::Return => break location,
            _ => return Err(Error::ControlFlow),
        }
    };
    Ok(Trace {
        assignments,
        calls,
        drops,
        returning,
        branch,
        cleanup,
        visited,
        events,
    })
}
