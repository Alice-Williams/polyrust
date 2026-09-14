//! C keys wrap source-owned compiler metadata without duplicating its reader.
use super::{Result, c};
pub(super) use crate::source_origin::{Cache, identity};
use portable_backend_c::ast::{CDeclarationKey, CGeneratedOrigin, CIdentifier};
use portable_codegen::RustSourceNode;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

pub(super) fn key(
    tcx: TyCtxt<'_>,
    cache: &mut Cache,
    id: DefId,
    node: RustSourceNode,
    span: Span,
    name: &str,
) -> Result<CDeclarationKey> {
    Ok(CDeclarationKey {
        origin: CGeneratedOrigin::RustSource(crate::source_origin::read(
            tcx, cache, id, node, span,
        )?),
        name: c(CIdentifier::new(name))?,
    })
}
