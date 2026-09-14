//! Admit canonical HIR with one constructor and a closed whole-value move chain.
use super::{LinearError as Error, Result, scopes};
use crate::owned_source::BoxConstructionInput;
use rustc_abi::ExternAbi;
use rustc_ast::BindingMode;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::LocalDefId,
};
use rustc_middle::ty::{TyCtxt, TypeckResults};
use std::collections::HashSet;

pub(super) struct Plan<'tcx> {
    pub constructor: BoxConstructionInput<'tcx>,
    pub parameter: hir::HirId,
    pub bindings: Vec<hir::HirId>,
    pub scope: hir::HirId,
    pub scopes: scopes::ScopeEvidence<'tcx>,
}

enum Nesting {
    RootOnly,
    TailBlocks,
}

fn binding(pattern: &hir::Pat<'_>) -> Result<hir::HirId> {
    match pattern.kind {
        hir::PatKind::Binding(BindingMode::NONE, id, _, None) => Ok(id),
        _ => Err(Error::BodyShape),
    }
}

fn local<'tcx>(checked: &TypeckResults<'tcx>, expression: &hir::Expr<'tcx>) -> Result<hir::HirId> {
    if !checked.expr_adjustments(expression).is_empty() {
        return Err(Error::BodyShape);
    }
    let hir::ExprKind::Path(path) = &expression.kind else {
        return Err(Error::BodyShape);
    };
    match checked.qpath_res(path, expression.hir_id) {
        Res::Local(id) => Ok(id),
        _ => Err(Error::SourceIdentity),
    }
}

pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    read_shape(tcx, owner, Nesting::RootOnly)
}

pub(super) fn read_tail_scopes(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    read_shape(tcx, owner, Nesting::TailBlocks)
}

fn read_shape(tcx: TyCtxt<'_>, owner: LocalDefId, nesting: Nesting) -> Result<Plan<'_>> {
    if tcx.def_kind(owner) != DefKind::Fn || tcx.generics_of(owner).count() != 0 {
        return Err(Error::Signature);
    }
    let signature = tcx.fn_sig(owner).instantiate_identity().skip_binder();
    if signature.abi() != ExternAbi::Rust
        || !signature.safety().is_safe()
        || signature.c_variadic()
        || signature.inputs() != [tcx.types.i32]
        || signature.output() != tcx.types.i32
    {
        return Err(Error::Signature);
    }
    let body = tcx.hir_body_owned_by(owner);
    let [parameter] = body.params else {
        return Err(Error::Signature);
    };
    let parameter = binding(parameter.pat)?;
    let hir::ExprKind::Block(mut block, None) = body.value.kind else {
        return Err(Error::BodyShape);
    };
    if matches!(nesting, Nesting::RootOnly) && block.stmts.is_empty() {
        return Err(Error::BodyShape);
    }
    let root = block.hir_id;
    let mut parent = None;
    let mut scope_blocks = Vec::new();
    let mut binding_scopes = Vec::new();
    let checked = tcx.typeck(owner);
    let mut bindings = Vec::new();
    let mut constructor = None;
    let mut previous = parameter;
    let mut seen = HashSet::from([parameter]);
    let tail = loop {
        if scope_blocks.len() >= 64 {
            return Err(Error::Budget);
        }
        scope_blocks.push((block.hir_id, parent));
        for statement in block.stmts {
            if bindings.len() >= 128 {
                return Err(Error::Budget);
            }
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
            let initializer = declaration.init.ok_or(Error::BodyShape)?;
            if !checked.expr_adjustments(initializer).is_empty() {
                return Err(Error::BodyShape);
            }
            if constructor.is_none() {
                let input = BoxConstructionInput::read(tcx, owner, initializer)
                    .map_err(Error::Constructor)?;
                if local(checked, input.argument())? != parameter {
                    return Err(Error::Argument);
                }
                constructor = Some(input);
            } else if local(checked, initializer)? != previous {
                return Err(Error::SourceIdentity);
            }
            previous = id;
            bindings.push(id);
            binding_scopes.push((id, block.hir_id));
        }
        let tail = block.expr.ok_or(Error::BodyShape)?;
        match (tail.kind, &nesting) {
            (hir::ExprKind::Block(child, None), Nesting::TailBlocks) => {
                parent = Some(block.hir_id);
                block = child;
            }
            _ => break tail,
        }
    };
    let constructor = constructor.ok_or(Error::BodyShape)?;
    for &id in &bindings {
        if checked.node_type(id) != constructor.result() {
            return Err(Error::SourceIdentity);
        }
    }
    let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = tail.kind else {
        return Err(Error::BodyShape);
    };
    if local(checked, operand)? != previous
        || checked.expr_ty(tail) != tcx.types.i32
        || !checked.expr_adjustments(tail).is_empty()
    {
        return Err(Error::SourceIdentity);
    }
    Ok(Plan {
        constructor,
        parameter,
        bindings,
        scope: root,
        scopes: scopes::certify(
            tcx,
            owner,
            scopes::Claims {
                blocks: scope_blocks,
                drop: binding_scopes.last().ok_or(Error::Scope)?.1,
                bindings: binding_scopes,
                read: block.hir_id,
            },
        )?,
    })
}
