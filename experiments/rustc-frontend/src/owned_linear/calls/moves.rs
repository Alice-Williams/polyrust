//! Unique whole-owner edges and a closed census of all Box locals.
use super::super::{LinearError as Error, Result, flow::Trace};
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::{self, Ty, TyCtxt},
};
use std::collections::HashSet;

pub(super) struct Owners {
    pub current: Option<(mir::Local, Option<mir::Location>)>,
    pub locals: HashSet<mir::Local>,
}
impl Owners {
    pub fn new(parameter: Option<mir::Local>) -> Self {
        Self {
            current: parameter.map(|p| (p, None)),
            locals: parameter.into_iter().collect(),
        }
    }
    pub fn receive(&mut self, local: mir::Local, at: mir::Location) -> Result<()> {
        if self.current.is_some() || !self.locals.insert(local) {
            return Err(Error::MoveGraph);
        }
        self.current = Some((local, Some(at)));
        Ok(())
    }
    pub fn advance<'tcx>(
        &mut self,
        body: &mir::Body<'tcx>,
        trace: &Trace<'_, 'tcx>,
        box_ty: Ty<'tcx>,
        used: &mut HashSet<mir::Location>,
    ) -> Result<(mir::Local, mir::Location)> {
        let (current, previous) = self.current.ok_or(Error::MoveGraph)?;
        let mut edges = trace.assignments.iter().filter(|a|
            matches!(a.value, Rvalue::Use(Operand::Move(p), _) if *p == mir::Place::from(current)));
        let edge = edges.next().ok_or(Error::MoveGraph)?;
        if edges.next().is_some()
            || trace.definition(edge.destination.local)?.location != edge.location
            || body.local_decls[edge.destination.local].ty != box_ty
            || previous.is_some_and(|p| !trace.before(p, edge.location))
            || !used.insert(edge.location)
            || !self.locals.insert(edge.destination.local)
        {
            return Err(Error::MoveGraph);
        }
        self.current = Some((edge.destination.local, Some(edge.location)));
        Ok((edge.destination.local, edge.location))
    }
    pub fn finish<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &mir::Body<'tcx>,
        box_ty: Ty<'tcx>,
    ) -> Result<()> {
        let mut all = HashSet::new();
        for (local, decl) in body.local_decls.iter_enumerated() {
            if matches!(decl.ty.kind(), ty::Adt(def, _) if Some(def.did()) == tcx.lang_items().owned_box())
            {
                if decl.ty != box_ty {
                    return Err(Error::MoveGraph);
                }
                all.insert(local);
            }
        }
        if all != self.locals {
            return Err(Error::MoveGraph);
        }
        Ok(())
    }
}
