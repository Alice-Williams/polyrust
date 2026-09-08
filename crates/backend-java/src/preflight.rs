//! Exact Java admission before mapping-owned strategy selection.

mod admission;
mod matches;
mod registration;
mod shapes;

use admission::JavaFeatureAdmission;
pub(crate) use matches::{JavaMatchDispatch, match_dispatch, payload_free_enum};
use shapes::{fallible_constant_diagnostics, valid_shape};

use portable_build::{CapabilityId, Modules, PortableTests};
use portable_codegen::{
    CoreFeature, DeclarationFeature, FeatureShape, FeatureUse, TargetCapabilityRegistry, TargetId,
    VerifiedCore, collect_core_features,
};
use portable_core_ir::CoreProgram;
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};

use crate::capabilities::{JavaCapabilitySet, java_capabilities};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCapabilitySelection {
    program_prerequisites: Vec<CapabilityId>,
    selected: Vec<JavaFeatureAdmission>,
}

impl JavaCapabilitySelection {
    #[cfg(test)]
    pub(crate) fn for_test(core: &CoreProgram) -> Self {
        JavaCapabilityRegistry::default()
            .select(core)
            .expect("boundary fixture capabilities")
    }

    pub(crate) fn validate_for(
        &self,
        program: &CoreProgram,
        features: JavaCapabilitySet,
    ) -> Result<(), Vec<Diagnostic>> {
        let expected = JavaCapabilityRegistry::new(features).select(program)?;
        if self == &expected {
            Ok(())
        } else {
            Err(vec![java_capability_diagnostic(
                "Java admission inventory does not match exact feature uses, owners, and prerequisites",
                java_capability_source(),
            )])
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct JavaCapabilityRegistry {
    features: JavaCapabilitySet,
}

impl Default for JavaCapabilityRegistry {
    fn default() -> Self {
        Self::new(java_capabilities())
    }
}

impl JavaCapabilityRegistry {
    pub fn target(&self) -> TargetId {
        TargetId::parse("org.polyrust.java").expect("static Java target ID is valid")
    }

    fn admit(&self, usage: &FeatureUse) -> Result<JavaFeatureAdmission, Box<Diagnostic>> {
        if !valid_shape(usage.feature(), usage.shape()) {
            return Err(Box::new(java_capability_diagnostic(
                "feature was collected with a shape outside Java's typed lowering contract",
                usage.source().clone(),
            )));
        }
        Ok(self.confirm_registered_mapping(usage))
    }

    fn select(&self, program: &CoreProgram) -> Result<JavaCapabilitySelection, Vec<Diagnostic>> {
        let _ = self.registered::<Modules>();
        let _ = self.registered::<PortableTests>();
        let program_prerequisites = vec![CapabilityId::Modules, CapabilityId::PortableTests];
        let mut selected = Vec::new();
        let mut diagnostics = fallible_constant_diagnostics(program);
        for usage in collect_core_features(program).iter() {
            if usage.feature() == CoreFeature::Declaration(DeclarationFeature::Enum)
                && usage.shape() == &(FeatureShape::Aggregate { field_count: 0 })
            {
                diagnostics.push(java_capability_diagnostic(
                    "Java does not yet represent legacy zero-variant enums",
                    usage.source().clone(),
                ));
            }
            match self.admit(usage) {
                Ok(mut admission) => {
                    self.complete_match_admission(program, &mut admission);
                    selected.push(admission);
                }
                Err(diagnostic) => diagnostics.push(*diagnostic),
            }
        }
        sort_diagnostics(&mut diagnostics);
        if diagnostics.is_empty() {
            Ok(JavaCapabilitySelection {
                program_prerequisites,
                selected,
            })
        } else {
            Err(diagnostics)
        }
    }
}

impl TargetCapabilityRegistry<CoreProgram> for JavaCapabilityRegistry {
    type Selection = JavaCapabilitySelection;

    fn preflight(
        &self,
        core: &VerifiedCore<CoreProgram>,
    ) -> Result<Self::Selection, Vec<Diagnostic>> {
        self.select(core.value())
    }
}

fn java_capability_source() -> SourceRef {
    SourceRef::logical(["java-capability-selection"])
}

fn java_capability_diagnostic(message: impl Into<String>, source: SourceRef) -> Diagnostic {
    let mut diagnostic = Diagnostic::error(DiagnosticCode::UnsupportedCapability, message, source);
    diagnostic.target = Some("org.polyrust.java".to_owned());
    diagnostic
}

#[cfg(test)]
#[path = "tests/preflight_ownership.rs"]
mod preflight_ownership_tests;

#[cfg(test)]
#[path = "tests/preflight_admission.rs"]
mod preflight_admission_tests;

#[cfg(test)]
#[path = "tests/preflight_matches.rs"]
mod preflight_match_tests;

#[cfg(test)]
#[path = "tests/preflight_empty.rs"]
mod preflight_empty_tests;

#[cfg(test)]
#[path = "tests/capability_selection.rs"]
mod tests;
