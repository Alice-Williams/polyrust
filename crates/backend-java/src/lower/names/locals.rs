//! Binding names retain their checked CoreLocalId through every declaration/read.

use super::{JavaPortableNames, NamePool};
use crate::ast::JavaIdentifier;
use portable_core_ir::{CoreLocalId, CoreParameter, CoreProgram};

impl JavaPortableNames {
    pub(super) fn allocate_locals(&mut self, core: &CoreProgram) {
        let mut pool = NamePool::new(
            core.locals().iter().map(|value| value.name.as_str()),
            std::iter::empty(),
        );
        pool.occupied.extend(self.qualifier_names());
        // Core local identities already have package-wide arena indices. A single
        // name space also prevents accidental Java overlap across nested blocks.
        self.locals = core
            .locals()
            .iter()
            .map(|value| pool.allocate(&value.name))
            .collect();
    }

    pub(in crate::lower) fn local(&self, id: CoreLocalId) -> &JavaIdentifier {
        &self.locals[id.index()]
    }

    pub(in crate::lower) fn parameters(&self, parameters: &[CoreParameter]) -> Vec<JavaIdentifier> {
        let mut pool = NamePool::new(
            parameters.iter().map(|value| value.header.name.as_str()),
            std::iter::empty(),
        );
        pool.occupied.extend(self.qualifier_names());
        parameters
            .iter()
            .map(|parameter| match parameter.local {
                Some(id) => self.local(id).clone(),
                None => pool.allocate(&parameter.header.name),
            })
            .collect()
    }
}
