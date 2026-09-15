//! Constructor-rooted ownership positions for the closed two-level source grammar.
use super::super::{LinearError as E, Result, source::local};
use super::{
    frame::{self, Frame},
    path::{self, SourcePlace},
};
use crate::owned_source::{
    BoxConstructionInput,
    nested_record::{FieldKind, NestedRecordConstructionInput},
    record::RecordConstructionInput,
};
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::ty::{Ty, TyCtxt};
use std::collections::HashSet;

pub(super) struct Construction<'tcx> {
    pub input: BoxConstructionInput<'tcx>,
    pub parameter: HirId,
    pub binding: HirId,
}
pub(super) struct Movement<'tcx> {
    pub source: SourcePlace<'tcx>,
    pub destination: HirId,
    pub ty: Ty<'tcx>,
}
pub(super) struct Leaf<'tcx> {
    pub constructor: HirId,
    pub current: SourcePlace<'tcx>,
}
pub(super) struct Plan<'tcx> {
    pub frame: Frame<'tcx>,
    pub constructions: Vec<Construction<'tcx>>,
    pub inner: (HirId, RecordConstructionInput<'tcx>),
    pub outer: (HirId, NestedRecordConstructionInput<'tcx>),
    pub movements: Vec<Movement<'tcx>>,
    pub leaves: Vec<Leaf<'tcx>>,
    pub cleanup: Vec<(super::events::CleanupKind, SourcePlace<'tcx>)>,
    pub read: HirId,
}
pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    let frame = frame::read(tcx, owner)?;
    let checked = tcx.typeck(owner);
    let mut anchors = HashSet::new();
    let mut constructions = Vec::new();
    let mut leaves = Vec::new();
    for &(binding, initializer) in frame.declarations.iter().take(3) {
        let input = BoxConstructionInput::read(tcx, owner, initializer).map_err(E::Constructor)?;
        let parameter = local(checked, input.argument())?;
        if !frame.parameters.contains(&parameter) || !anchors.insert(parameter) {
            return Err(E::Argument);
        }
        leaves.push(Leaf {
            constructor: binding,
            current: SourcePlace::local(binding),
        });
        constructions.push(Construction {
            input,
            parameter,
            binding,
        });
    }
    let box_ty = constructions.first().ok_or(E::BodyShape)?.input.result();
    if constructions.len() != 3 || constructions.iter().any(|c| c.input.result() != box_ty) {
        return Err(E::SourceIdentity);
    }
    let &(inner_binding, inner_init) = frame.declarations.get(3).ok_or(E::BodyShape)?;
    let inner = RecordConstructionInput::read(tcx, owner, inner_init).map_err(E::Record)?;
    if inner.fields().len() != 2 {
        return Err(E::BodyShape);
    }
    for field in inner.fields() {
        let source = SourcePlace::local(local(checked, field.initializer())?);
        let destination = SourcePlace::local(inner_binding).child(path::field(
            tcx,
            inner.result(),
            field.index(),
        )?);
        transfer(&mut leaves, &source, &destination, 1)?;
    }
    let &(outer_binding, outer_init) = frame.declarations.get(4).ok_or(E::BodyShape)?;
    let outer =
        NestedRecordConstructionInput::read(tcx, owner, outer_init).map_err(E::NestedRecord)?;
    let [nested, spare] = outer.layout().fields() else {
        return Err(E::BodyShape);
    };
    if !matches!(nested.kind(), FieldKind::Record(layout) if layout.ty() == inner.result())
        || !matches!(spare.kind(), FieldKind::Box(ty) if *ty == box_ty)
    {
        return Err(E::BodyShape);
    }
    for initializer in outer.initializers() {
        let source = SourcePlace::local(local(checked, initializer.expression())?);
        let field = outer.field(initializer.index()).ok_or(E::SourceIdentity)?;
        let count = match field.kind() {
            FieldKind::Record(_) => {
                if source.binding() != inner_binding {
                    return Err(E::SourceIdentity);
                }
                2
            }
            FieldKind::Box(_) => 1,
        };
        let destination = SourcePlace::local(outer_binding).child(path::field(
            tcx,
            outer.result(),
            field.index(),
        )?);
        transfer(&mut leaves, &source, &destination, count)?;
    }
    if leaves
        .iter()
        .any(|leaf| leaf.current.binding() != outer_binding)
    {
        return Err(E::SourceIdentity);
    }
    let mut movements = Vec::new();
    for &(destination, initializer) in frame.declarations.iter().skip(5) {
        let source = path::read(tcx, checked, initializer)?;
        let ty = checked.expr_ty(initializer);
        let count = if ty == box_ty {
            1
        } else if ty == inner.result() && !source.fields().is_empty() {
            2
        } else {
            return Err(E::BodyShape);
        };
        transfer(
            &mut leaves,
            &source,
            &SourcePlace::local(destination),
            count,
        )?;
        movements.push(Movement {
            source,
            destination,
            ty,
        });
    }
    let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = frame.scopes.exit().value().kind else {
        return Err(E::Read);
    };
    let read = local(checked, operand)?;
    if !leaves
        .iter()
        .any(|leaf| leaf.current == SourcePlace::local(read))
        || checked.expr_ty(operand) != box_ty
    {
        return Err(E::Read);
    }
    // Lexical binding order is canonical HIR order, with declaration-indexed
    // field paths within each remaining record. No presentation names sort it.
    leaves.sort_by_key(|leaf| {
        let position = frame
            .declarations
            .iter()
            .position(|(id, _)| *id == leaf.current.binding())
            .unwrap();
        (
            std::cmp::Reverse(position),
            leaf.current
                .fields()
                .iter()
                .map(|f| f.index().as_usize())
                .collect::<Vec<_>>(),
        )
    });
    let cleanup = cleanup(&leaves, inner.result());
    Ok(Plan {
        frame,
        constructions,
        inner: (inner_binding, inner),
        outer: (outer_binding, outer),
        movements,
        leaves,
        cleanup,
        read,
    })
}

fn cleanup<'tcx>(
    leaves: &[Leaf<'tcx>],
    inner: Ty<'tcx>,
) -> Vec<(super::events::CleanupKind, SourcePlace<'tcx>)> {
    use super::events::CleanupKind;
    let mut result = Vec::new();
    let mut cursor = 0;
    while let Some(first) = leaves.get(cursor) {
        if let (Some(parent), Some(field), Some(second)) = (
            first.current.parent(),
            first.current.fields().last(),
            leaves.get(cursor + 1),
        ) && field.parent() == inner
            && field.index().as_usize() == 0
            && second.current.parent().as_ref() == Some(&parent)
            && second
                .current
                .fields()
                .last()
                .is_some_and(|f| f.parent() == inner && f.index().as_usize() == 1)
        {
            result.push((CleanupKind::InnerRecord, parent));
            cursor += 2;
        } else {
            result.push((CleanupKind::Leaf, first.current.clone()));
            cursor += 1;
        }
    }
    result
}

fn transfer<'tcx>(
    leaves: &mut [Leaf<'tcx>],
    source: &SourcePlace<'tcx>,
    destination: &SourcePlace<'tcx>,
    expected: usize,
) -> Result<()> {
    if leaves
        .iter()
        .filter(|leaf| source.contains(&leaf.current))
        .count()
        != expected
    {
        return Err(E::SourceIdentity);
    }
    for leaf in leaves
        .iter_mut()
        .filter(|leaf| source.contains(&leaf.current))
    {
        leaf.current = leaf.current.relocate(source, destination)?;
    }
    if leaves
        .iter()
        .map(|leaf| &leaf.current)
        .collect::<HashSet<_>>()
        .len()
        != leaves.len()
    {
        return Err(E::SourceIdentity);
    }
    Ok(())
}
