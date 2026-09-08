//! Direct certification has the same ordered resource gate as the compiler.

use std::sync::{Arc, Mutex};

use super::{
    LinkedPackage, RenderReadyPackage, TargetDialect, certify_linked_package,
    certify_resolved_package,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Input {
    language_valid: bool,
    resources_valid: bool,
}

#[derive(Clone, Debug)]
struct Probe(Arc<Mutex<Vec<&'static str>>>);

fn error(code: DiagnosticCode, phase: &str) -> Vec<Diagnostic> {
    vec![Diagnostic::error(
        code,
        format!("deliberate {phase} rejection"),
        SourceRef::logical(["certification-test", phase]),
    )]
}

impl TargetDialect for Probe {
    type Unresolved = Input;
    type Resolved = Input;

    fn verify_unresolved(&self, _: &Input) -> Result<(), Vec<Diagnostic>> {
        panic!("post-link certification must not invoke unresolved verification")
    }

    fn verify_resolved(&self, input: &Input) -> Result<(), Vec<Diagnostic>> {
        self.0.lock().unwrap().push("language");
        if input.language_valid {
            Ok(())
        } else {
            Err(error(DiagnosticCode::InvalidStructure, "language"))
        }
    }

    fn verify_resources(&self, input: &Input) -> Result<(), Vec<Diagnostic>> {
        self.0.lock().unwrap().push("resources");
        if input.resources_valid {
            Ok(())
        } else {
            Err(error(DiagnosticCode::TargetResourceLimit, "resources"))
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Entry {
    Resolved,
    Linked,
}

fn consume_render_ready(probe: &Probe, ready: &RenderReadyPackage<Probe>, expected: Input) {
    probe.0.lock().unwrap().push("render");
    assert_eq!(ready.ast(), &expected);
}

#[test]
fn every_public_certifier_gates_the_exact_payload_on_both_checks() {
    for entry in [Entry::Resolved, Entry::Linked] {
        for language_valid in [false, true] {
            for resources_valid in [false, true] {
                let input = Input {
                    language_valid,
                    resources_valid,
                };
                let probe = Probe(Arc::new(Mutex::new(vec![])));
                let result = match entry {
                    Entry::Resolved => certify_resolved_package(&probe, input),
                    Entry::Linked => certify_linked_package(&probe, LinkedPackage::new(input)),
                };
                assert_eq!(
                    result.is_ok(),
                    language_valid && resources_valid,
                    "{entry:?}"
                );
                let expected = match result {
                    Ok(ready) => {
                        consume_render_ready(&probe, &ready, input);
                        assert_eq!(ready.clone().ast(), &input);
                        vec!["language", "resources", "render"]
                    }
                    Err(diagnostics) => {
                        assert_eq!(diagnostics.len(), 1);
                        if language_valid {
                            assert_eq!(diagnostics[0].code, DiagnosticCode::TargetResourceLimit);
                            vec!["language", "resources"]
                        } else {
                            assert_eq!(diagnostics[0].code, DiagnosticCode::InvalidStructure);
                            vec!["language"]
                        }
                    }
                };
                assert_eq!(*probe.0.lock().unwrap(), expected, "{entry:?}/{input:?}");
            }
        }
    }
}
