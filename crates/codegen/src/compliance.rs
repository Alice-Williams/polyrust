use portable_check::v0::CheckedProgram;
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};
use serde::Serialize;

use crate::{
    BackendOptions, ContentHash, DeclaredDependency, InjectedHelper, OutputManifest, TypedCompiler,
};

pub trait TypedComplianceOracle: Send + Sync {
    fn canonical_semantics(&self, manifest: &OutputManifest) -> Result<String, String>;
    fn interface_composition(&self, manifest: &OutputManifest) -> Result<String, String>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedComplianceExpectations {
    pub canonical_semantics: String,
    pub interface_composition: String,
    pub dependencies: Vec<DeclaredDependency>,
    pub helpers: Vec<InjectedHelper>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TypedComplianceEvidence {
    target: String,
    manifest_hash: ContentHash,
    canonical_semantics: String,
    interface_composition: String,
}

impl TypedComplianceEvidence {
    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn manifest_hash(&self) -> &ContentHash {
        &self.manifest_hash
    }

    pub fn canonical_semantics(&self) -> &str {
        &self.canonical_semantics
    }

    pub fn interface_composition(&self) -> &str {
        &self.interface_composition
    }

    pub fn canonical_json(&self) -> String {
        serde_json::to_string(self).expect("typed compliance evidence always serializes")
    }
}

pub fn prove_typed_compiler(
    compiler: &dyn TypedCompiler,
    program: &CheckedProgram,
    options: &BackendOptions,
    oracle: &dyn TypedComplianceOracle,
    expectations: &TypedComplianceExpectations,
) -> Result<TypedComplianceEvidence, Vec<Diagnostic>> {
    let descriptor = compiler.descriptor();
    let source = SourceRef::logical(["typed-compliance", descriptor.target.as_str()]);
    let mut diagnostics = Vec::new();
    let mut manifests = Vec::new();
    for attempt in 0..3 {
        match compiler.compile_checked(program, options) {
            Ok(manifest) => manifests.push(manifest),
            Err(error) => diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                format!("typed compliance generation {attempt} failed: {error:?}"),
                source.clone(),
            )),
        }
    }
    if manifests.len() != 3 {
        sort_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }
    let canonical = manifests
        .iter()
        .map(OutputManifest::canonical_json)
        .collect::<Vec<_>>();
    if canonical[1..].iter().any(|value| value != &canonical[0]) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "typed compiler produced different manifests across three generations",
            source.clone(),
        ));
    }
    let manifest = &manifests[0];
    if manifest.schema_version() != 2
        || manifest
            .generation()
            .is_none_or(|generation| generation.target() != descriptor.target.as_str())
    {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "typed compiler did not produce matching manifest-v2 identity",
            source.clone(),
        ));
    }
    if manifest.dependencies() != expectations.dependencies {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "typed compiler dependency set is not the expected minimal set",
            source.clone(),
        ));
    }
    if manifest.helpers() != expectations.helpers {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "typed compiler helper set is not the expected minimal set",
            source.clone(),
        ));
    }

    let semantics = match oracle.canonical_semantics(manifest) {
        Ok(value) => value,
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                format!("canonical semantic oracle failed: {message}"),
                source.clone(),
            ));
            String::new()
        }
    };
    if semantics != expectations.canonical_semantics {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "generated canonical semantics differ from the reference result",
            source.clone(),
        ));
    }
    let composition = match oracle.interface_composition(manifest) {
        Ok(value) => value,
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                format!("interface/composition oracle failed: {message}"),
                source.clone(),
            ));
            String::new()
        }
    };
    if composition != expectations.interface_composition {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InterfaceNonconformance,
            "generated interface/composition behavior differs from the reference result",
            source,
        ));
    }

    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(TypedComplianceEvidence {
            target: descriptor.target.to_string(),
            manifest_hash: manifest.stable_hash(),
            canonical_semantics: semantics,
            interface_composition: composition,
        })
    } else {
        Err(diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_pipeline::tests::compliance_adapter;

    struct TestOracle;

    impl TypedComplianceOracle for TestOracle {
        fn canonical_semantics(&self, manifest: &OutputManifest) -> Result<String, String> {
            Ok(format!("files={}", manifest.files().len()))
        }

        fn interface_composition(&self, manifest: &OutputManifest) -> Result<String, String> {
            manifest
                .file("src/generated.test")
                .map(|_| "dispatch=composition".to_owned())
                .ok_or_else(|| "generated interface fixture is absent".to_owned())
        }
    }

    fn expectations() -> TypedComplianceExpectations {
        TypedComplianceExpectations {
            canonical_semantics: "files=2".to_owned(),
            interface_composition: "dispatch=composition".to_owned(),
            dependencies: vec![],
            helpers: vec![],
        }
    }

    #[test]
    fn compliance_kit_proves_semantics_composition_minimality_and_three_runs() {
        let (compiler, program) = compliance_adapter();
        let evidence = prove_typed_compiler(
            compiler.as_ref(),
            &program,
            &BackendOptions::default(),
            &TestOracle,
            &expectations(),
        )
        .unwrap();
        assert_eq!(evidence.target(), "org.polyrust.typed-test");
        assert_eq!(evidence.canonical_semantics(), "files=2");
        assert_eq!(evidence.interface_composition(), "dispatch=composition");
        assert!(evidence.manifest_hash().as_str().starts_with("fnv1a64:"));
        assert_eq!(evidence.canonical_json(), evidence.canonical_json());
    }

    #[test]
    fn compliance_kit_rejects_semantic_composition_and_minimality_faults() {
        let (compiler, program) = compliance_adapter();
        let mut wrong = expectations();
        wrong.canonical_semantics = "files=2".to_owned();
        wrong.interface_composition = "dispatch=inheritance".to_owned();
        wrong.dependencies.push(DeclaredDependency {
            ecosystem: "cargo".to_owned(),
            name: "unexpected".to_owned(),
            requirement: "1".to_owned(),
            features: vec![],
        });
        assert!(
            prove_typed_compiler(
                compiler.as_ref(),
                &program,
                &BackendOptions::default(),
                &TestOracle,
                &wrong,
            )
            .is_err()
        );
    }
}
