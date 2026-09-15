//! One canonical root frame, independent of any ownership-effect classification.
use super::{LinearError as Error, Result, exits, scopes, source::binding};
use rustc_abi::ExternAbi;
use rustc_hir::{self as hir, HirId, def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::{FnSig, TyCtxt};
use std::collections::HashSet;

pub(super) struct Frame<'tcx> {
    pub owner: LocalDefId,
    pub signature: FnSig<'tcx>,
    pub parameter: HirId,
    pub declarations: Vec<(HirId, &'tcx hir::Expr<'tcx>)>,
    pub scopes: scopes::ScopeFacts<'tcx>,
}
pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Frame<'_>> {
    if tcx.def_kind(owner) != DefKind::Fn || tcx.generics_of(owner).count() != 0 {
        return Err(Error::Signature);
    }
    let signature = tcx.fn_sig(owner).instantiate_identity().skip_binder();
    if signature.abi() != ExternAbi::Rust
        || !signature.safety().is_safe()
        || signature.c_variadic()
        || signature.inputs().len() != 1
    {
        return Err(Error::Signature);
    }
    let body = tcx.hir_body_owned_by(owner);
    let [parameter] = body.params else {
        return Err(Error::Signature);
    };
    let parameter = binding(parameter.pat)?;
    let hir::ExprKind::Block(root, None) = body.value.kind else {
        return Err(Error::BodyShape);
    };
    let mode = match root.expr {
        Some(e) if !matches!(e.kind, hir::ExprKind::Ret(_)) => exits::Mode::Tail,
        _ => exits::Mode::Return,
    };
    let (statements, exits::End::Exit(exit)) = exits::parts(root, mode)? else {
        return Err(Error::BodyShape);
    };
    if statements.len() > 128 {
        return Err(Error::Budget);
    }
    let checked = tcx.typeck(owner);
    let mut declarations = Vec::new();
    let mut seen = HashSet::from([parameter]);
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
    if !checked.expr_adjustments(exit.value()).is_empty()
        || checked.expr_ty(exit.value()) != signature.output()
        || checked.node_type(parameter) != signature.inputs()[0]
    {
        return Err(Error::SourceIdentity);
    }
    let scopes = scopes::certify_exit(
        tcx,
        owner,
        scopes::ContainmentClaims {
            blocks: vec![(root.hir_id, None)],
            bindings: declarations
                .iter()
                .map(|(id, _)| (*id, root.hir_id))
                .collect(),
            read: root.hir_id,
        },
        exit,
    )?;
    Ok(Frame {
        owner,
        signature,
        parameter,
        declarations,
        scopes,
    })
}
