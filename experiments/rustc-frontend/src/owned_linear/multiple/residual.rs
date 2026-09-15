//! Admit only completely unobserved constant Boolean/unit compiler temporaries.
use super::super::{LinearError as Error, Result, flow::Trace};
use rustc_middle::{
    mir::{
        self, Operand, Rvalue,
        visit::{MutatingUseContext, NonUseContext, PlaceContext, Visitor},
    },
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

#[derive(Clone, Copy)]
enum Kind {
    Boolean,
    Unit,
}

impl Kind {
    fn accepts<'tcx>(self, tcx: TyCtxt<'tcx>, value: &Rvalue<'tcx>) -> bool {
        let Rvalue::Use(Operand::Constant(value), _) = value else {
            return false;
        };
        match self {
            Self::Boolean => {
                value.const_.ty() == tcx.types.bool
                    && value
                        .const_
                        .try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized())
                        .is_some()
            }
            Self::Unit => {
                matches!(value.const_, mir::Const::Val(mir::ConstValue::ZeroSized, ty) if ty == tcx.types.unit)
            }
        }
    }
}

struct Uses<'a> {
    locals: &'a HashSet<mir::Local>,
    writes: &'a HashSet<mir::Location>,
    invalid: bool,
}
impl<'tcx> Visitor<'tcx> for Uses<'_> {
    fn visit_local(&mut self, local: mir::Local, context: PlaceContext, location: mir::Location) {
        if !self.locals.contains(&local) {
            return;
        }
        match context {
            PlaceContext::MutatingUse(MutatingUseContext::Store)
                if self.writes.contains(&location) => {}
            PlaceContext::NonUse(NonUseContext::StorageLive | NonUseContext::StorageDead) => {}
            _ => self.invalid = true,
        }
    }
}

pub(in crate::owned_linear) fn account<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    used: &mut HashSet<mir::Location>,
) -> Result<()> {
    let mut locals = HashSet::new();
    let mut writes = HashSet::new();
    for assignment in &trace.assignments {
        if used.contains(&assignment.location) {
            continue;
        }
        let Rvalue::Use(Operand::Constant(value), _) = assignment.value else {
            return Err(Error::Assignment);
        };
        if body.local_decls[assignment.destination.local].ty != tcx.types.bool
            || !assignment.destination.projection.is_empty()
            || value.const_.ty() != tcx.types.bool
            || value
                .const_
                .try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized())
                .is_none()
        {
            return Err(Error::Assignment);
        }
        locals.insert(assignment.destination.local);
        writes.insert(assignment.location);
    }
    account_unread(tcx, body, Kind::Boolean, locals, writes, used)
}

pub(in crate::owned_linear) fn account_units<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    used: &mut HashSet<mir::Location>,
) -> Result<()> {
    let mut locals = HashSet::new();
    let mut writes = HashSet::new();
    for assignment in &trace.assignments {
        if used.contains(&assignment.location)
            || body.local_decls[assignment.destination.local].ty != tcx.types.unit
        {
            continue;
        }
        let Rvalue::Use(Operand::Constant(value), _) = assignment.value else {
            return Err(Error::Assignment);
        };
        if !assignment.destination.projection.is_empty()
            || !matches!(value.const_, mir::Const::Val(mir::ConstValue::ZeroSized, ty) if ty == tcx.types.unit)
        {
            return Err(Error::Assignment);
        }
        locals.insert(assignment.destination.local);
        writes.insert(assignment.location);
    }
    account_unread(tcx, body, Kind::Unit, locals, writes, used)
}

fn account_unread<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    kind: Kind,
    locals: HashSet<mir::Local>,
    writes: HashSet<mir::Location>,
    used: &mut HashSet<mir::Location>,
) -> Result<()> {
    // A temporary can be written in both cleanup arms. Check every definition
    // before allowing those stores in the whole-body no-reader proof, while
    // accounting only the writes that actually occur on the selected path.
    let mut definitions = HashSet::new();
    for (block, data) in body.basic_blocks.iter_enumerated() {
        for (statement_index, statement) in data.statements.iter().enumerate() {
            let mir::StatementKind::Assign(pair) = &statement.kind else {
                continue;
            };
            if locals.contains(&pair.0.local) {
                if !pair.0.projection.is_empty() || !kind.accepts(tcx, &pair.1) {
                    return Err(Error::Assignment);
                }
                definitions.insert(mir::Location {
                    block,
                    statement_index,
                });
            }
        }
    }
    let mut uses = Uses {
        locals: &locals,
        writes: &definitions,
        invalid: false,
    };
    uses.visit_body(body);
    if uses.invalid {
        return Err(Error::Assignment);
    }
    used.extend(writes);
    Ok(())
}
