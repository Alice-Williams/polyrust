//! Closed root-block ownership paths, retaining nominal field identities.
use super::super::super::{
    LinearError as Error, Result, exits, scopes,
    source::{binding, local},
};
use crate::owned_source::{BoxConstructionInput, record::RecordConstructionInput};
use rustc_abi::{ExternAbi, FieldIdx};
use rustc_hir::{
    self as hir, HirId,
    def::DefKind,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::TyCtxt;
use std::collections::HashSet;

/// Presentation names never identify an ownership step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SourcePlace {
    Local(HirId),
    Field {
        record: HirId,
        declaration: DefId,
        index: FieldIdx,
    },
}
pub(super) struct Chain<'tcx> {
    pub constructor: BoxConstructionInput<'tcx>,
    pub parameter: HirId,
    pub places: Vec<SourcePlace>,
}
pub(super) struct Plan<'tcx> {
    pub parameters: Vec<HirId>,
    pub chains: Vec<Chain<'tcx>>,
    pub record: RecordConstructionInput<'tcx>,
    pub binding: HirId,
    pub scopes: scopes::ScopeFacts<'tcx>,
    pub read: HirId,
}

pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    if tcx.def_kind(owner) != DefKind::Fn || tcx.generics_of(owner).count() != 0 {
        return Err(Error::Signature);
    }
    let signature = tcx.fn_sig(owner).instantiate_identity().skip_binder();
    if signature.abi() != ExternAbi::Rust
        || !signature.safety().is_safe()
        || signature.c_variadic()
        || signature.inputs().is_empty()
        || signature.inputs().len() > 128
        || signature.inputs().iter().any(|ty| *ty != tcx.types.i32)
        || signature.output() != tcx.types.i32
    {
        return Err(Error::Signature);
    }
    let body = tcx.hir_body_owned_by(owner);
    let parameters: Vec<_> = body
        .params
        .iter()
        .map(|p| binding(p.pat))
        .collect::<Result<_>>()?;
    if parameters.len() != signature.inputs().len() {
        return Err(Error::Signature);
    }
    let hir::ExprKind::Block(block, None) = body.value.kind else {
        return Err(Error::BodyShape);
    };
    let mode = match block.expr {
        Some(expression) if !matches!(expression.kind, hir::ExprKind::Ret(_)) => exits::Mode::Tail,
        _ => exits::Mode::Return,
    };
    let (statements, exits::End::Exit(exit)) = exits::parts(block, mode)? else {
        return Err(Error::BodyShape);
    };
    if statements.len() > 128 {
        return Err(Error::Budget);
    }
    let checked = tcx.typeck(owner);
    let mut seen: HashSet<_> = parameters.iter().copied().collect();
    let mut anchors = HashSet::new();
    let mut chains: Vec<Chain<'_>> = Vec::new();
    let mut record: Option<(HirId, RecordConstructionInput<'_>)> = None;
    let mut bindings = Vec::new();
    for statement in statements {
        let hir::StmtKind::Let(declaration) = statement.kind else {
            return Err(Error::BodyShape);
        };
        if declaration.els.is_some() {
            return Err(Error::BodyShape);
        }
        let id = binding(declaration.pat)?;
        if id.owner.def_id != owner || !seen.insert(id) {
            return Err(Error::SourceIdentity);
        }
        let init = declaration.init.ok_or(Error::BodyShape)?;
        if !checked.expr_adjustments(init).is_empty() {
            return Err(Error::BodyShape);
        }
        match init.kind {
            hir::ExprKind::Call(..) if record.is_none() => {
                let constructor =
                    BoxConstructionInput::read(tcx, owner, init).map_err(Error::Constructor)?;
                let parameter = local(checked, constructor.argument())?;
                if !parameters.contains(&parameter) {
                    return Err(Error::Argument);
                }
                if !anchors.insert(parameter) {
                    return Err(Error::Ambiguous);
                }
                if checked.node_type(id) != constructor.result() {
                    return Err(Error::SourceIdentity);
                }
                chains.push(Chain {
                    constructor,
                    parameter,
                    places: vec![SourcePlace::Local(id)],
                });
            }
            hir::ExprKind::Struct(..) if record.is_none() => {
                let input =
                    RecordConstructionInput::read(tcx, owner, init).map_err(Error::Record)?;
                if checked.node_type(id) != input.result() {
                    return Err(Error::SourceIdentity);
                }
                for field in input.fields() {
                    let previous = SourcePlace::Local(local(checked, field.initializer())?);
                    advance(
                        &mut chains,
                        previous,
                        SourcePlace::Field {
                            record: id,
                            declaration: field.declaration(),
                            index: field.index(),
                        },
                        field.ty(),
                    )?;
                }
                record = Some((id, input));
            }
            hir::ExprKind::Path(_) => {
                advance(
                    &mut chains,
                    SourcePlace::Local(local(checked, init)?),
                    SourcePlace::Local(id),
                    checked.node_type(id),
                )?;
            }
            hir::ExprKind::Field(base, _) => {
                let (record_id, input) = record.as_ref().ok_or(Error::BodyShape)?;
                if local(checked, base)? != *record_id || checked.expr_ty(base) != input.result() {
                    return Err(Error::SourceIdentity);
                }
                let index = checked.field_index(init.hir_id);
                let field = input
                    .fields()
                    .iter()
                    .find(|f| f.index() == index)
                    .ok_or(Error::SourceIdentity)?;
                if checked.expr_ty(init) != field.ty() {
                    return Err(Error::SourceIdentity);
                }
                advance(
                    &mut chains,
                    SourcePlace::Field {
                        record: *record_id,
                        declaration: field.declaration(),
                        index,
                    },
                    SourcePlace::Local(id),
                    checked.node_type(id),
                )?;
            }
            _ => return Err(Error::BodyShape),
        }
        bindings.push((id, block.hir_id));
    }
    let (binding, record) = record.ok_or(Error::BodyShape)?;
    let tail = exit.value();
    let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = tail.kind else {
        return Err(Error::BodyShape);
    };
    let read = local(checked, operand)?;
    let selected = chains
        .iter()
        .find(|c| c.places.last() == Some(&SourcePlace::Local(read)))
        .ok_or(Error::Read)?;
    if !selected
        .places
        .iter()
        .any(|p| matches!(p, SourcePlace::Field { .. }))
        || checked.expr_ty(tail) != tcx.types.i32
        || !checked.expr_adjustments(tail).is_empty()
    {
        return Err(Error::Read);
    }
    let scopes = scopes::certify_exit(
        tcx,
        owner,
        scopes::ContainmentClaims {
            blocks: vec![(block.hir_id, None)],
            bindings,
            read: block.hir_id,
        },
        exit,
    )?;
    Ok(Plan {
        parameters,
        chains,
        record,
        binding,
        scopes,
        read,
    })
}

fn advance<'tcx>(
    chains: &mut [Chain<'tcx>],
    previous: SourcePlace,
    next: SourcePlace,
    ty: rustc_middle::ty::Ty<'tcx>,
) -> Result<()> {
    let mut matching = chains
        .iter_mut()
        .filter(|c| c.places.last() == Some(&previous));
    let chain = matching.next().ok_or(Error::SourceIdentity)?;
    if matching.next().is_some() || chain.constructor.result() != ty {
        return Err(Error::SourceIdentity);
    }
    chain.places.push(next);
    Ok(())
}
