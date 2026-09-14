//! Distinct compiler-parameter identities anchor otherwise same-typed owners.
use super::super::{
    LinearError as Error, Result, exits, scopes,
    source::{binding, local},
};
use crate::owned_source::BoxConstructionInput;
use rustc_abi::ExternAbi;
use rustc_hir::{self as hir, HirId, def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;
use std::collections::HashSet;

pub(super) struct Chain<'tcx> {
    pub constructor: BoxConstructionInput<'tcx>,
    pub parameter: HirId,
    pub bindings: Vec<HirId>,
}
pub(super) struct Plan<'tcx> {
    pub parameters: Vec<HirId>,
    pub chains: Vec<Chain<'tcx>>,
    pub scopes: scopes::ScopeFacts<'tcx>,
    pub read: HirId,
}

pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    read_shape(tcx, owner, exits::Mode::Tail)
}
pub(super) fn read_return(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Plan<'_>> {
    read_shape(tcx, owner, exits::Mode::Return)
}
fn read_shape(tcx: TyCtxt<'_>, owner: LocalDefId, mode: exits::Mode) -> Result<Plan<'_>> {
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
    let mut seen: HashSet<_> = parameters.iter().copied().collect();
    if seen.len() != parameters.len() {
        return Err(Error::SourceIdentity);
    }
    let hir::ExprKind::Block(mut block, None) = body.value.kind else {
        return Err(Error::BodyShape);
    };
    let checked = tcx.typeck(owner);
    let mut chains: Vec<Chain<'_>> = Vec::new();
    let mut anchors = HashSet::new();
    let mut blocks = Vec::new();
    let mut bindings = Vec::new();
    let mut parent = None;
    let exit = loop {
        if blocks.len() >= 64 {
            return Err(Error::Budget);
        }
        blocks.push((block.hir_id, parent));
        let (statements, end) = exits::parts(block, mode)?;
        for statement in statements {
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
            let init = declaration.init.ok_or(Error::BodyShape)?;
            if !checked.expr_adjustments(init).is_empty() {
                return Err(Error::BodyShape);
            }
            match init.kind {
                hir::ExprKind::Call(..) => {
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
                        bindings: vec![id],
                    });
                }
                hir::ExprKind::Path(_) => {
                    let previous = local(checked, init)?;
                    let mut matching = chains
                        .iter_mut()
                        .filter(|c| c.bindings.last() == Some(&previous));
                    let chain = matching.next().ok_or(Error::SourceIdentity)?;
                    if matching.next().is_some()
                        || checked.node_type(id) != chain.constructor.result()
                    {
                        return Err(Error::SourceIdentity);
                    }
                    chain.bindings.push(id);
                }
                _ => return Err(Error::BodyShape),
            }
            bindings.push((id, block.hir_id));
        }
        match end {
            exits::End::Nested(child) => {
                parent = Some(block.hir_id);
                block = child;
            }
            exits::End::Exit(exit) => break exit,
        }
    };
    if chains.is_empty() {
        return Err(Error::BodyShape);
    }
    let tail = exit.value();
    let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = tail.kind else {
        return Err(Error::BodyShape);
    };
    let read = local(checked, operand)?;
    if chains
        .iter()
        .filter(|c| c.bindings.last() == Some(&read))
        .count()
        != 1
        || checked.expr_ty(tail) != tcx.types.i32
        || !checked.expr_adjustments(tail).is_empty()
    {
        return Err(Error::Read);
    }
    Ok(Plan {
        parameters,
        chains,
        read,
        scopes: scopes::certify_exit(
            tcx,
            owner,
            scopes::ContainmentClaims {
                blocks,
                bindings,
                read: block.hir_id,
            },
            exit,
        )?,
    })
}
