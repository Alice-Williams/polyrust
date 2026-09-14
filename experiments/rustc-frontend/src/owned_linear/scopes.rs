//! Certify scope claims against canonical HIR containment, never debug metadata.
use super::{LinearError as Error, Result, exits};
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
    exit: exits::Exit<'tcx>,
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

impl<'tcx> ScopeFacts<'tcx> {
    pub(super) fn exit(&self) -> exits::Exit<'tcx> {
        self.exit
    }
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
    walk(tcx, owner, claims, exits::Mode::Tail)
}

pub(super) fn certify_exit<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    claims: ContainmentClaims,
    exit: exits::Exit<'tcx>,
) -> Result<ScopeFacts<'tcx>> {
    certify_route(tcx, owner, claims, exit, exit.mode())
}

pub(super) fn certify_route<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    claims: ContainmentClaims,
    exit: exits::Exit<'tcx>,
    mode: exits::Mode,
) -> Result<ScopeFacts<'tcx>> {
    let facts = walk(tcx, owner, claims, mode)?;
    if !facts.exit.same(exit) {
        return Err(Error::Scope);
    }
    Ok(facts)
}

fn walk<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    claims: ContainmentClaims,
    mode: exits::Mode,
) -> Result<ScopeFacts<'tcx>> {
    let hir::ExprKind::Block(mut block, None) = tcx.hir_body_owned_by(owner).value.kind else {
        return Err(Error::Scope);
    };
    let mut parent = None;
    let mut blocks = Vec::new();
    let mut bindings = Vec::new();
    let exit = loop {
        if blocks.len() >= 64 {
            return Err(Error::Budget);
        }
        blocks.push((block, parent));
        let (statements, end) = exits::parts(block, mode)?;
        for statement in statements {
            let hir::StmtKind::Let(local) = statement.kind else {
                return Err(Error::Scope);
            };
            let hir::PatKind::Binding(_, id, _, None) = local.pat.kind else {
                return Err(Error::Scope);
            };
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
        exit,
    };
    if !facts.blocks().eq(claims.blocks) {
        return Err(Error::Scope);
    }
    Ok(facts)
}
