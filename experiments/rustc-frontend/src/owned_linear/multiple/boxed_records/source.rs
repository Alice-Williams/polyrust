//! Canonical root-scope scalar producers and explicit Box field selection.
use super::super::super::{
    LinearError as Error, Result, exits, scopes,
    source::{binding, local},
};
use crate::owned_source::boxed_record::ScalarRecordBoxInput;
use rustc_abi::{ExternAbi, FieldIdx};
use rustc_hir::{self as hir, HirId, def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::{Ty, TyCtxt};
use std::collections::HashSet;

pub(super) struct Initializer<'tcx> {
    pub index: FieldIdx,
    pub parameter: HirId,
    pub expression: &'tcx hir::Expr<'tcx>,
}
pub(super) struct Plan<'tcx> {
    pub constructor: ScalarRecordBoxInput<'tcx>,
    pub parameters: Vec<(HirId, Ty<'tcx>)>,
    pub literal: &'tcx hir::Expr<'tcx>,
    pub record: HirId,
    pub fields: Vec<Initializer<'tcx>>,
    pub owners: Vec<HirId>,
    pub selected: FieldIdx,
    pub scopes: scopes::ScopeFacts<'tcx>,
}

pub(super) fn check_scope<'tcx>(tcx: TyCtxt<'tcx>, plan: &Plan<'tcx>) -> Result<()> {
    let owner = plan.constructor.owner();
    let hir::ExprKind::Block(root, None) = tcx.hir_body_owned_by(owner).value.kind else {
        return Err(Error::Scope);
    };
    let bindings: Vec<_> = std::iter::once(plan.record)
        .chain(plan.owners.iter().copied())
        .map(|id| (id, root.hir_id))
        .collect();
    if plan.scopes.blocks().collect::<Vec<_>>() != [(root.hir_id, None)]
        || plan.scopes.bindings() != bindings
        || plan.scopes.read_scope() != root.hir_id
    {
        return Err(Error::Scope);
    }
    scopes::certify_exit(
        tcx,
        owner,
        scopes::ContainmentClaims {
            blocks: vec![(root.hir_id, None)],
            bindings,
            read: root.hir_id,
        },
        plan.scopes.exit(),
    )?;
    Ok(())
}

pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    if tcx.def_kind(owner) != DefKind::Fn || tcx.generics_of(owner).count() != 0 {
        return Err(Error::Signature);
    }
    let signature = tcx.fn_sig(owner).instantiate_identity().skip_binder();
    let scalar = |ty| ty == tcx.types.i32 || ty == tcx.types.bool;
    if signature.abi() != ExternAbi::Rust
        || !signature.safety().is_safe()
        || signature.c_variadic()
        || signature.inputs().is_empty()
        || signature.inputs().len() > 128
        || signature.inputs().iter().any(|ty| !scalar(*ty))
        || !scalar(signature.output())
    {
        return Err(Error::Signature);
    }
    let body = tcx.hir_body_owned_by(owner);
    if body.params.len() != signature.inputs().len() {
        return Err(Error::Signature);
    }
    let parameters = body
        .params
        .iter()
        .zip(signature.inputs())
        .map(|(p, ty)| Ok((binding(p.pat)?, *ty)))
        .collect::<Result<Vec<_>>>()?;
    let hir::ExprKind::Block(block, None) = body.value.kind else {
        return Err(Error::BodyShape);
    };
    let mode = match block.expr {
        Some(e) if !matches!(e.kind, hir::ExprKind::Ret(_)) => exits::Mode::Tail,
        _ => exits::Mode::Return,
    };
    let (statements, exits::End::Exit(exit)) = exits::parts(block, mode)? else {
        return Err(Error::BodyShape);
    };
    if statements.len() > 128 {
        return Err(Error::Budget);
    }
    let checked = tcx.typeck(owner);
    let mut declarations = Vec::new();
    let mut seen: HashSet<_> = parameters.iter().map(|p| p.0).collect();
    for statement in statements {
        let hir::StmtKind::Let(declaration) = statement.kind else {
            return Err(Error::BodyShape);
        };
        if declaration.els.is_some() {
            return Err(Error::BodyShape);
        }
        let id = binding(declaration.pat)?;
        let init = declaration.init.ok_or(Error::BodyShape)?;
        if id.owner.def_id != owner
            || !seen.insert(id)
            || !checked.expr_adjustments(init).is_empty()
        {
            return Err(Error::SourceIdentity);
        }
        declarations.push((id, init));
    }
    let [(record, literal), (first, call), rest @ ..] = declarations.as_slice() else {
        return Err(Error::BodyShape);
    };
    let constructor = ScalarRecordBoxInput::read(tcx, owner, call).map_err(Error::BoxedRecord)?;
    let payload = constructor.payload();
    if local(checked, constructor.argument())? != *record
        || checked.node_type(*record) != payload.ty()
        || checked.expr_ty(literal) != payload.ty()
        || checked.node_type(*first) != constructor.result()
    {
        return Err(Error::SourceIdentity);
    }
    let hir::ExprKind::Struct(_, initializers, hir::StructTailExpr::None) = literal.kind else {
        return Err(Error::BodyShape);
    };
    if initializers.len() != payload.fields().len() {
        return Err(Error::SourceIdentity);
    }
    let mut indices = HashSet::new();
    let mut fields = Vec::new();
    for field in initializers {
        let index = checked.field_index(field.hir_id);
        let declared = payload
            .fields()
            .get(index.as_usize())
            .ok_or(Error::SourceIdentity)?;
        let parameter = local(checked, field.expr)?;
        if !indices.insert(index)
            || !parameters.contains(&(parameter, declared.ty()))
            || checked.expr_ty(field.expr) != declared.ty()
        {
            return Err(Error::SourceIdentity);
        }
        fields.push(Initializer {
            index,
            parameter,
            expression: field.expr,
        });
    }
    let mut owners = vec![*first];
    for (id, init) in rest {
        if local(checked, init)? != *owners.last().unwrap()
            || checked.node_type(*id) != constructor.result()
        {
            return Err(Error::SourceIdentity);
        }
        owners.push(*id);
    }
    let read = exit.value();
    let hir::ExprKind::Field(base, _) = read.kind else {
        return Err(Error::Read);
    };
    let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = base.kind else {
        return Err(Error::Read);
    };
    let selected = checked.field_index(read.hir_id);
    let field = payload
        .fields()
        .get(selected.as_usize())
        .ok_or(Error::Read)?;
    if local(checked, operand)? != *owners.last().unwrap()
        || checked.expr_ty(operand) != constructor.result()
        || checked.expr_ty(base) != payload.ty()
        || !checked.expr_adjustments(base).is_empty()
        || !checked.expr_adjustments(read).is_empty()
        || checked.expr_ty(read) != field.ty()
        || signature.output() != field.ty()
    {
        return Err(Error::Read);
    }
    let scopes = scopes::certify_exit(
        tcx,
        owner,
        scopes::ContainmentClaims {
            blocks: vec![(block.hir_id, None)],
            bindings: declarations
                .iter()
                .map(|(id, _)| (*id, block.hir_id))
                .collect(),
            read: block.hir_id,
        },
        exit,
    )?;
    Ok(Plan {
        constructor,
        parameters,
        literal,
        record: *record,
        fields,
        owners,
        selected,
        scopes,
    })
}
