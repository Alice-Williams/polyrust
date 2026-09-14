//! Compare each distinct shared export-graph allocation at most once.
use portable_codegen::RustCrateExports;
use std::{collections::BTreeSet, sync::Arc};

pub(super) struct Agreement<'a> {
    expected: &'a Arc<RustCrateExports>,
    checked: BTreeSet<*const RustCrateExports>,
    #[cfg(test)]
    comparisons: usize,
}

impl<'a> Agreement<'a> {
    pub(super) fn new(expected: &'a Arc<RustCrateExports>) -> Self {
        Self {
            expected,
            checked: BTreeSet::from([Arc::as_ptr(expected)]),
            #[cfg(test)]
            comparisons: 0,
        }
    }

    pub(super) fn check(&mut self, actual: &'a Arc<RustCrateExports>) -> Result<(), String> {
        // The certificate keeps every allocation alive. Pointer identity is
        // memoization only: never source authority, output order or spelling.
        if self.checked.contains(&Arc::as_ptr(actual)) {
            return Ok(());
        }
        #[cfg(test)]
        {
            self.comparisons += 1;
        }
        if actual.as_ref() != self.expected.as_ref() {
            return Err("Java dependency registrations disagree on source exports".into());
        }
        self.checked.insert(Arc::as_ptr(actual));
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/dependency_export_allocations.rs"]
mod tests;
