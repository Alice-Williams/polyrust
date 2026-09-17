//! One bounded defining-identity union for body reads and public aliases.
use super::Inventory;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use std::collections::BTreeMap;

impl Inventory {
    pub(crate) fn constant_imports(
        &self,
        tcx: TyCtxt<'_>,
        referenced: &[DefId],
    ) -> Result<Vec<DefId>, String> {
        if referenced.len() > 4096 {
            return Err("constant import inventory scan budget exceeded".into());
        }
        let mut definitions = BTreeMap::new();
        for definition in referenced.iter().copied().chain(
            self.foreign_constants
                .values()
                .map(|value| value.definition()),
        ) {
            if definition.is_local() {
                return Err("foreign constant union contains a local declaration".into());
            }
            let identity = super::super::identity(tcx, definition);
            if definitions
                .insert(identity, definition)
                .is_some_and(|prior| prior != definition)
            {
                return Err("constant import union has conflicting compiler identities".into());
            }
            if definitions.len() > 4096 {
                return Err("constant import union exceeds 4096 defining declarations".into());
            }
        }
        Ok(definitions.into_values().collect())
    }
}
