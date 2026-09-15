//! Canonical three-parameter root frame for the initial nested body grammar.
use super::super::{LinearError as E, Result, exits, scopes, source::binding};
use rustc_abi::ExternAbi;
use rustc_hir::{self as hir, HirId, def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;
use std::collections::HashSet;

pub(super) struct Frame<'tcx> {
    pub owner: LocalDefId,
    pub parameters: [HirId; 3],
    pub declarations: Vec<(HirId, &'tcx hir::Expr<'tcx>)>,
    pub scopes: scopes::ScopeFacts<'tcx>,
}
pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId) -> Result<Frame<'_>> {
    if tcx.def_kind(owner) != DefKind::Fn || tcx.generics_of(owner).count() != 0 {
        return Err(E::Signature);
    }
    let signature = tcx.fn_sig(owner).instantiate_identity().skip_binder();
    if signature.abi() != ExternAbi::Rust
        || !signature.safety().is_safe()
        || signature.c_variadic()
        || signature.inputs() != [tcx.types.i32; 3]
        || signature.output() != tcx.types.i32
    {
        return Err(E::Signature);
    }
    let body = tcx.hir_body_owned_by(owner);
    let [a, b, c] = body.params else {
        return Err(E::Signature);
    };
    let parameters = [binding(a.pat)?, binding(b.pat)?, binding(c.pat)?];
    let checked = tcx.typeck(owner);
    if parameters
        .iter()
        .any(|id| id.owner.def_id != owner || checked.node_type(*id) != tcx.types.i32)
    {
        return Err(E::SourceIdentity);
    }
    let hir::ExprKind::Block(root, None) = body.value.kind else {
        return Err(E::BodyShape);
    };
    let mode = match root.expr {
        Some(e) if !matches!(e.kind, hir::ExprKind::Ret(_)) => exits::Mode::Tail,
        _ => exits::Mode::Return,
    };
    let (statements, exits::End::Exit(exit)) = exits::parts(root, mode)? else {
        return Err(E::BodyShape);
    };
    if statements.len() < 6 || statements.len() > 128 {
        return Err(E::Budget);
    }
    let mut seen: HashSet<_> = parameters.into_iter().collect();
    let mut declarations = Vec::new();
    for statement in statements {
        let hir::StmtKind::Let(declaration) = statement.kind else {
            return Err(E::BodyShape);
        };
        if declaration.els.is_some() {
            return Err(E::BodyShape);
        }
        let id = binding(declaration.pat)?;
        let init = declaration.init.ok_or(E::BodyShape)?;
        if id.owner.def_id != owner
            || !seen.insert(id)
            || !checked.expr_adjustments(init).is_empty()
            || checked.node_type(id) != checked.expr_ty(init)
        {
            return Err(E::SourceIdentity);
        }
        declarations.push((id, init));
    }
    if !checked.expr_adjustments(exit.value()).is_empty()
        || checked.expr_ty(exit.value()) != tcx.types.i32
    {
        return Err(E::Read);
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
        parameters,
        declarations,
        scopes,
    })
}
