#![forbid(unsafe_code)]

use portable_diagnostics::SourceRef;
use portable_diagnostics::code::DiagnosticCode;
use portable_diagnostics::model::{Diagnostic, Severity};

#[test]
fn downstream_crate_constructs_and_tests_structured_diagnostic() {
    let diagnostic = Diagnostic::error(
        DiagnosticCode::DuplicateDeclaration,
        "name already exists",
        SourceRef::logical(["module(example)", "constant(NAME)"]),
    );

    assert_eq!(diagnostic.code.as_str(), "P0102");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.labels.len(), 1);
}
