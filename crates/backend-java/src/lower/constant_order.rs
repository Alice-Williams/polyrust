//! Java lowering: constant order.

use super::constant_dependencies::collect_constant_dependencies;
use super::{Lowering, diagnostic};
use portable_core_ir::{CoreConstantId, CoreDeclaration};
use portable_diagnostics::Diagnostic;
use std::collections::BTreeSet;

impl Lowering<'_> {
    pub(super) fn ordered_constant_ids(&self) -> Result<Vec<CoreConstantId>, Vec<Diagnostic>> {
        let mut ordered = Vec::new();
        let mut visiting = BTreeSet::new();
        let mut emitted = BTreeSet::new();
        for declaration in &self.core.module().declarations {
            if let CoreDeclaration::Constant(id) = declaration {
                self.visit_constant(*id, &mut visiting, &mut emitted, &mut ordered)?;
            }
        }
        Ok(ordered)
    }

    fn visit_constant(
        &self,
        id: CoreConstantId,
        visiting: &mut BTreeSet<CoreConstantId>,
        emitted: &mut BTreeSet<CoreConstantId>,
        ordered: &mut Vec<CoreConstantId>,
    ) -> Result<(), Vec<Diagnostic>> {
        if emitted.contains(&id) {
            return Ok(());
        }
        if !visiting.insert(id) {
            return Err(vec![diagnostic(
                "verified CoreIR contains a cyclic Java constant dependency",
            )]);
        }
        let constant = self
            .core
            .constant(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR constant dependency")])?;
        let mut dependencies = BTreeSet::new();
        collect_constant_dependencies(&constant.value, &mut dependencies);
        for dependency in dependencies {
            self.visit_constant(dependency, visiting, emitted, ordered)?;
        }
        visiting.remove(&id);
        emitted.insert(id);
        ordered.push(id);
        Ok(())
    }
}
