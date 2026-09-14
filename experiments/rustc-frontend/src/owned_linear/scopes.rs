//! Certify scope claims against canonical HIR containment, never debug metadata.
use super::{LinearError as Error, Result};
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;

pub(super) struct Claims {
    pub blocks: Vec<(HirId, Option<HirId>)>,
    pub bindings: Vec<(HirId, HirId)>,
    pub read: HirId,
    pub drop: HirId,
}

pub(super) struct ContainmentClaims {
    pub blocks: Vec<(HirId, Option<HirId>)>,
    pub bindings: Vec<(HirId, HirId)>,
    pub read: HirId,
}

#[derive(Clone)]
pub(crate) struct ScopeFacts<'tcx> {
    blocks: Vec<(&'tcx hir::Block<'tcx>, Option<HirId>)>,
    bindings: Vec<(HirId, HirId)>,
    read: HirId,
}

#[derive(Clone)]
pub(crate) struct ScopeEvidence<'tcx> {
    blocks: Vec<(&'tcx hir::Block<'tcx>, Option<HirId>)>,
    bindings: Vec<(HirId, HirId)>,
    read: HirId,
    drop: HirId,
}

impl ScopeEvidence<'_> {
    pub(crate) fn blocks(&self) -> impl Iterator<Item = (HirId, Option<HirId>)> + '_ {
        self.blocks
            .iter()
            .map(|(block, parent)| (block.hir_id, *parent))
    }
    pub(crate) fn bindings(&self) -> &[(HirId, HirId)] {
        &self.bindings
    }
    pub(crate) fn read_scope(&self) -> HirId {
        self.read
    }
    pub(crate) fn drop_scope(&self) -> HirId {
        self.drop
    }
}

pub(super) fn certify<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    claims: Claims,
) -> Result<ScopeEvidence<'tcx>> {
    let facts = certify_containment(
        tcx,
        owner,
        ContainmentClaims {
            blocks: claims.blocks,
            bindings: claims.bindings,
            read: claims.read,
        },
    )?;
    if Some(claims.drop) != facts.bindings().last().map(|item| item.1) {
        return Err(Error::Scope);
    }
    Ok(ScopeEvidence {
        read: facts.read_scope(),
        blocks: facts.blocks,
        bindings: facts.bindings,
        drop: claims.drop,
    })
}

impl ScopeFacts<'_> {
    pub(crate) fn blocks(&self) -> impl Iterator<Item = (HirId, Option<HirId>)> + '_ {
        self.blocks
            .iter()
            .map(|(block, parent)| (block.hir_id, *parent))
    }
    pub(crate) fn bindings(&self) -> &[(HirId, HirId)] {
        &self.bindings
    }
    pub(crate) fn read_scope(&self) -> HirId {
        self.read
    }
}

pub(super) fn certify_containment<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    claims: ContainmentClaims,
) -> Result<ScopeFacts<'tcx>> {
    let hir::ExprKind::Block(mut block, None) = tcx.hir_body_owned_by(owner).value.kind else {
        return Err(Error::Scope);
    };
    let mut parent = None;
    let mut blocks = Vec::new();
    let mut bindings = Vec::new();
    loop {
        if blocks.len() >= 64 {
            return Err(Error::Budget);
        }
        blocks.push((block, parent));
        for statement in block.stmts {
            let hir::StmtKind::Let(local) = statement.kind else {
                return Err(Error::Scope);
            };
            let hir::PatKind::Binding(_, id, _, None) = local.pat.kind else {
                return Err(Error::Scope);
            };
            bindings.push((id, block.hir_id));
        }
        let tail = block.expr.ok_or(Error::Scope)?;
        if let hir::ExprKind::Block(child, None) = tail.kind {
            parent = Some(block.hir_id);
            block = child;
        } else {
            break;
        }
    }
    if claims.blocks.len() != blocks.len()
        || claims.bindings != bindings
        || claims.read != block.hir_id
    {
        return Err(Error::Scope);
    }
    let facts = ScopeFacts {
        blocks,
        bindings,
        read: claims.read,
    };
    if !facts.blocks().eq(claims.blocks) {
        return Err(Error::Scope);
    }
    Ok(facts)
}
