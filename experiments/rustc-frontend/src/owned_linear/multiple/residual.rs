//! Admit only completely unobserved constant Boolean compiler temporaries.
use super::super::{LinearError as Error, Result, flow::Trace};
use rustc_middle::{
    mir::{
        self, Operand, Rvalue,
        visit::{MutatingUseContext, NonUseContext, PlaceContext, Visitor},
    },
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

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

pub(super) fn account<'tcx>(
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
    let mut uses = Uses {
        locals: &locals,
        writes: &writes,
        invalid: false,
    };
    uses.visit_body(body);
    if uses.invalid {
        return Err(Error::Assignment);
    }
    used.extend(writes);
    Ok(())
}
