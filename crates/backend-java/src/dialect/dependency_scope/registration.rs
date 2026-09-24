//! Incremental original-owner closure; each authority is traversed only once.
use super::JavaDependencyScope;
use crate::dialect::JavaDependencyPackage;
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

struct Limits {
    bindings: usize,
    owners: usize,
    names: usize,
    name: usize,
}
const LIMITS: Limits = Limits {
    bindings: 100_000,
    owners: super::verification::MAX_OWNERS,
    names: 64 * 1024 * 1024,
    name: 65_535,
};

#[cfg(test)]
#[path = "../../tests/result_import_registration.rs"]
mod tests;

fn error(code: DiagnosticCode, message: impl Into<String>) -> Vec<Diagnostic> {
    vec![Diagnostic::error(
        code,
        message,
        SourceRef::logical(["java", "dependency-scope"]),
    )]
}

impl JavaDependencyScope {
    pub(super) fn verify_registration(
        &mut self,
        original: &JavaDependencyPackage,
        new_name: Option<usize>,
    ) -> Result<(), Vec<Diagnostic>> {
        self.verify_registration_with_limits(original, new_name, &LIMITS)
    }
    fn verify_registration_with_limits(
        &mut self,
        original: &JavaDependencyPackage,
        new_name: Option<usize>,
        limits: &Limits,
    ) -> Result<(), Vec<Diagnostic>> {
        let bindings = self
            .functions
            .len()
            .saturating_add(self.values.len())
            .saturating_add(self.result_types.len())
            .saturating_add(self.result_constructors.len())
            .saturating_add(self.result_accessors.len());
        if bindings > limits.bindings {
            return Err(error(
                DiagnosticCode::TargetResourceLimit,
                "Java dependency scope exceeds registered binding limit",
            ));
        }
        if let Some(name) = new_name {
            self.name_bytes = self.name_bytes.saturating_add(name);
            if name > limits.name || self.name_bytes > limits.names {
                return Err(error(
                    DiagnosticCode::TargetResourceLimit,
                    "Java dependency scope exceeds qualified name byte limit",
                ));
            }
        }
        let mut pending = vec![original];
        while let Some(owner) = pending.pop() {
            let id = owner.root().crate_id;
            if let Some(previous) = self.owners.get(&id) {
                if previous != owner {
                    return Err(error(
                        DiagnosticCode::InterfaceNonconformance,
                        "Java dependency source crate has conflicting owner certificates",
                    ));
                }
                continue;
            }
            if self.owners.len() >= limits.owners {
                return Err(error(
                    DiagnosticCode::TargetResourceLimit,
                    "Java dependency closure exceeds owner limit",
                ));
            }
            self.owners.insert(id, owner.clone());
            pending.extend(owner.dependencies());
        }
        Ok(())
    }
}
