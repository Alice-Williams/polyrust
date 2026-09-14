//! Shared compiler metadata; target keys and analysis authority live elsewhere.
mod exports;
mod modules;
pub(crate) use modules::Cache;
use portable_codegen::{
    RustDeclarationId, RustSourceLocation, RustSourceNode, RustSourceOrigin, RustVisibility,
};
use rustc_hir::{def::DefKind, def_id::DefId};
use rustc_middle::ty::{TyCtxt, Visibility};
use rustc_span::Span;
use std::sync::Arc;

type Result<T> = std::result::Result<T, String>;

pub(crate) fn identity(tcx: TyCtxt<'_>, id: DefId) -> RustDeclarationId {
    let hash = tcx.def_path_hash(id);
    RustDeclarationId {
        crate_id: hash.stable_crate_id().as_u64(),
        definition_path_hash: hash.local_hash().as_u64(),
    }
}

pub(crate) fn read(
    tcx: TyCtxt<'_>,
    cache: &mut Cache,
    id: DefId,
    node: RustSourceNode,
    span: Span,
) -> Result<Arc<RustSourceOrigin>> {
    let mut module = id;
    while tcx.def_kind(module) != DefKind::Mod {
        module = tcx.opt_parent(module).ok_or("missing source module")?;
    }
    let visibility = match tcx.visibility(id) {
        Visibility::Public => RustVisibility::Public,
        Visibility::Restricted(scope) => RustVisibility::RestrictedTo(identity(tcx, scope)),
    };
    let externally_reachable = id
        .as_local()
        .is_some_and(|local| tcx.effective_visibilities(()).is_exported(local));
    let documentation = if node == RustSourceNode::Declaration {
        cache.documentation(tcx, id)?
    } else {
        Vec::new()
    };
    Ok(Arc::new(RustSourceOrigin {
        declaration: identity(tcx, id),
        node,
        module: identity(tcx, module),
        location: location(tcx, span),
        visibility,
        externally_reachable,
        documentation,
        module_ancestors: if node == RustSourceNode::Declaration {
            cache.ancestry(tcx, module)?
        } else {
            [].into()
        },
        crate_exports: cache.exports(tcx)?,
    }))
}

fn location(tcx: TyCtxt<'_>, span: Span) -> RustSourceLocation {
    let location = tcx.sess.source_map().lookup_char_pos(span.lo());
    RustSourceLocation {
        file: location
            .file
            .name
            .prefer_remapped_unconditionally()
            .to_string(),
        line: location.line,
        column: location.col.0,
    }
}
