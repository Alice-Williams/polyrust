//! Trace actual typed producers, not matching scalar types or debug variables.
use super::{LinearError as Error, Result, flow::Flow};
use rustc_middle::{
    mir::{self, Operand, ProjectionElem, Rvalue},
    ty::{self, TyCtxt},
};
use rustc_type_ir::inherent::IntoKind;
use std::collections::HashSet;

pub(super) fn argument<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    flow: &Flow<'_, 'tcx>,
    operand: &Operand<'tcx>,
    parameter: mir::Local,
    used: &mut HashSet<mir::Location>,
) -> Result<()> {
    let mut operand = operand;
    let mut use_location = flow.call.0;
    let mut seen = HashSet::new();
    loop {
        let (Operand::Copy(place) | Operand::Move(place)) = operand else {
            return Err(Error::Argument);
        };
        if !place.projection.is_empty() || place.ty(&body.local_decls, tcx).ty != tcx.types.i32 {
            return Err(Error::Argument);
        }
        if place.local == parameter {
            if flow
                .assignments
                .iter()
                .any(|a| a.destination.local == parameter)
            {
                return Err(Error::Argument);
            }
            return Ok(());
        }
        if !seen.insert(place.local) {
            return Err(Error::Argument);
        }
        let assignment = flow.definition(place.local)?;
        if !flow.before(assignment.location, use_location) || !used.insert(assignment.location) {
            return Err(Error::Argument);
        }
        let Rvalue::Use(source, _) = assignment.value else {
            return Err(Error::Argument);
        };
        operand = source;
        use_location = assignment.location;
    }
}

pub(super) fn scalar_read<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    flow: &Flow<'_, 'tcx>,
    owner: mir::Local,
    used: &mut HashSet<mir::Location>,
) -> Result<(mir::Location, mir::Location)> {
    let read = flow.definition(mir::RETURN_PLACE)?;
    let Rvalue::Use(Operand::Copy(pointer), _) = read.value else {
        return Err(Error::Read);
    };
    if !matches!(pointer.projection.as_slice(), [ProjectionElem::Deref]) {
        return Err(Error::Read);
    }
    let pointer_ty = body.local_decls[pointer.local].ty;
    if !matches!(pointer_ty.kind(), ty::RawPtr(inner, rustc_hir::Mutability::Not) if *inner == tcx.types.i32)
    {
        return Err(Error::Read);
    }
    let cast = flow.definition(pointer.local)?;
    let Rvalue::Cast(mir::CastKind::Transmute, Operand::Copy(source), target_ty) = cast.value
    else {
        return Err(Error::Read);
    };
    if source.local != owner || *target_ty != pointer_ty || source.projection.len() != 2 {
        return Err(Error::Read);
    }
    let mut projected = body.local_decls[owner].ty;
    for projection in source.projection {
        let ProjectionElem::Field(field, recorded_ty) = projection else {
            return Err(Error::Read);
        };
        if field.as_usize() != 0 {
            return Err(Error::Read);
        }
        let ty::Adt(def, arguments) = projected.kind() else {
            return Err(Error::Read);
        };
        if !def.is_struct() {
            return Err(Error::Read);
        }
        let field = def
            .non_enum_variant()
            .fields
            .get(field)
            .ok_or(Error::Read)?;
        projected = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                field.ty(tcx, arguments),
            )
            .map_err(|_| Error::Read)?;
        if projected != recorded_ty {
            return Err(Error::Read);
        }
    }
    // Pin the compiler's Box -> Unique -> NonNull pointer producer, without
    // using any of these Rust representation details to generate C layout.
    let ty::Adt(def, arguments) = projected.kind() else {
        return Err(Error::Read);
    };
    if !def.is_struct() || def.non_enum_variant().fields.len() != 1 {
        return Err(Error::Read);
    }
    let field_ty = tcx
        .try_normalize_erasing_regions(
            ty::TypingEnv::fully_monomorphized(),
            def.non_enum_variant()
                .fields
                .iter()
                .next()
                .unwrap()
                .ty(tcx, arguments),
        )
        .map_err(|_| Error::Read)?;
    if !matches!(field_ty.kind(), ty::Pat(base, pattern) if *base == pointer_ty && matches!(pattern.kind(), ty::PatternKind::NotNull))
    {
        return Err(Error::Read);
    }
    if !flow.before(cast.location, read.location)
        || !used.insert(cast.location)
        || !used.insert(read.location)
    {
        return Err(Error::Read);
    }
    Ok((cast.location, read.location))
}
