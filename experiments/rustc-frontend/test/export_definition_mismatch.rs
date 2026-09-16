//! Test-only retained-DefId corruption; the binding graph itself is unchanged.
use portable_codegen::RustDeclarationId;
use rustc_hir::def_id::DefId;
use std::collections::BTreeMap;

pub(super) fn apply(definitions: &mut BTreeMap<RustDeclarationId, DefId>) {
    let mut entries = definitions.values().copied();
    if let (Some(first), Some(replacement)) = (entries.next(), entries.next()) {
        assert_ne!(first, replacement);
        *definitions.values_mut().next().unwrap() = replacement;
    }
}
