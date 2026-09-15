//! Closed canonical source grammar for one original and one cloned scalar Box.
use super::{
    super::{
        LinearError as Error, Result,
        frame::{self, Frame},
        source::local,
    },
    Owner,
};
use crate::owned_source::{BoxConstructionInput, cloning::BoxCloneInput};
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;

#[derive(Clone, Copy)]
pub(super) enum Step {
    Construct(HirId),
    Clone(HirId),
    Move(Owner, HirId),
}
impl Step {
    pub fn owner(self) -> Owner {
        match self {
            Self::Construct(_) => Owner::Original,
            Self::Clone(_) => Owner::Cloned,
            Self::Move(owner, _) => owner,
        }
    }
    pub fn binding(self) -> HirId {
        match self {
            Self::Construct(binding) | Self::Clone(binding) | Self::Move(_, binding) => binding,
        }
    }
}
pub(super) struct Plan<'tcx> {
    pub frame: Frame<'tcx>,
    pub construction: BoxConstructionInput<'tcx>,
    pub cloning: BoxCloneInput<'tcx>,
    pub steps: Vec<Step>,
    pub read: Owner,
}
pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    let frame = frame::read(tcx, owner)?;
    if frame.signature.inputs() != [tcx.types.i32]
        || frame.signature.output() != tcx.types.i32
        || frame.declarations.len() < 2
    {
        return Err(Error::Signature);
    }
    let checked = tcx.typeck(owner);
    let &(first, initializer) = frame.declarations.first().ok_or(Error::BodyShape)?;
    let construction =
        BoxConstructionInput::read(tcx, owner, initializer).map_err(Error::Constructor)?;
    if local(checked, construction.argument())? != frame.parameter {
        return Err(Error::Argument);
    }
    let box_ty = construction.result();
    let mut current = [Some(first), None];
    let mut cloning = None;
    let mut steps = vec![Step::Construct(first)];
    for &(binding, init) in frame.declarations.iter().skip(1) {
        if matches!(init.kind, hir::ExprKind::Path(_)) {
            let previous = local(checked, init)?;
            let tag = match (current[0], current[1]) {
                (Some(original), _) if previous == original => Owner::Original,
                (_, Some(cloned)) if previous == cloned => Owner::Cloned,
                _ => return Err(Error::SourceIdentity),
            };
            current[tag.index()] = Some(binding);
            steps.push(Step::Move(tag, binding));
        } else {
            if cloning.is_some() {
                return Err(Error::BodyShape);
            }
            let input = BoxCloneInput::read(tcx, owner, init).map_err(Error::Clone)?;
            if Some(input.binding()) != current[0] || input.box_ty() != box_ty {
                return Err(Error::SourceIdentity);
            }
            current[1] = Some(binding);
            cloning = Some(input);
            steps.push(Step::Clone(binding));
        }
    }
    if frame
        .declarations
        .iter()
        .any(|(binding, _)| checked.node_type(*binding) != box_ty)
    {
        return Err(Error::SourceIdentity);
    }
    let cloning = cloning.ok_or(Error::BodyShape)?;
    let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = frame.scopes.exit().value().kind else {
        return Err(Error::Read);
    };
    let selected = local(checked, operand)?;
    let read = match (current[0], current[1]) {
        (Some(original), _) if selected == original => Owner::Original,
        (_, Some(cloned)) if selected == cloned => Owner::Cloned,
        _ => return Err(Error::Read),
    };
    Ok(Plan {
        frame,
        construction,
        cloning,
        steps,
        read,
    })
}
