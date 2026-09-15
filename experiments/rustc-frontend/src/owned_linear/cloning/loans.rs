//! Authenticate the exact live owner and each unique shared-reference stage.
use super::super::{LinearError as Error, Result, flow::Trace};
use crate::owned_source::cloning::CloneForm;
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(crate) struct BorrowStep<'tcx> {
    reference: mir::Local,
    source: mir::Place<'tcx>,
    location: mir::Location,
}
impl<'tcx> BorrowStep<'tcx> {
    pub(crate) fn reference(self) -> mir::Local {
        self.reference
    }
    pub(crate) fn source(self) -> mir::Place<'tcx> {
        self.source
    }
    pub(crate) fn location(self) -> mir::Location {
        self.location
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Loan<'tcx> {
    Direct(BorrowStep<'tcx>),
    Reborrow {
        initial: BorrowStep<'tcx>,
        argument: BorrowStep<'tcx>,
    },
}
pub(super) struct Site<'a, 'tcx> {
    pub form: CloneForm<'tcx>,
    pub owner: mir::Local,
    pub previous: mir::Location,
    pub call: mir::Location,
    pub argument: &'a Operand<'tcx>,
}
pub(super) fn read<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    site: Site<'_, 'tcx>,
    used: &mut HashSet<mir::Location>,
) -> Result<Loan<'tcx>> {
    let Site {
        form,
        owner,
        previous,
        call,
        argument,
    } = site;
    let Operand::Move(reference) = *argument else {
        return Err(Error::Argument);
    };
    if !reference.projection.is_empty() {
        return Err(Error::Argument);
    }
    let box_ty = body.local_decls[owner].ty;
    let mut step = |reference: mir::Local| {
        if !matches!(body.local_decls[reference].ty.kind(), ty::Ref(_, target, rustc_hir::Mutability::Not) if *target == box_ty)
        {
            return Err(Error::Argument);
        }
        let definition = trace.definition(reference)?;
        let Rvalue::Ref(_, mir::BorrowKind::Shared, source) = definition.value else {
            return Err(Error::Argument);
        };
        if source.ty(&body.local_decls, tcx).ty != box_ty
            || !trace.before(previous, definition.location)
            || !trace.before(definition.location, call)
            || !used.insert(definition.location)
        {
            return Err(Error::Argument);
        }
        Ok(BorrowStep {
            reference,
            source: *source,
            location: definition.location,
        })
    };
    let last = step(reference.local)?;
    match form {
        CloneForm::Method => {
            if last.source != mir::Place::from(owner) {
                return Err(Error::Argument);
            }
            Ok(Loan::Direct(last))
        }
        CloneForm::ExplicitBorrow(_) => {
            if last.source.projection.len() != 1
                || last.source.projection[0] != mir::ProjectionElem::Deref
            {
                return Err(Error::Argument);
            }
            let first = step(last.source.local)?;
            if first.reference == last.reference
                || first.source != mir::Place::from(owner)
                || !trace.before(first.location, last.location)
            {
                return Err(Error::Argument);
            }
            Ok(Loan::Reborrow {
                initial: first,
                argument: last,
            })
        }
    }
}
