//! Full generated runtime packages use the same AST reservations as production.

use super::Lowering;
use crate::{
    JavaBackend, capabilities::java_capabilities, dialect::JavaDialect,
    preflight::JavaCapabilitySelection,
};
use portable_codegen::{
    Backend, BackendOptions, TargetLinker, certify_resolved_package, render_certified_package,
    verify_unresolved_package,
};

#[test]
fn generated_runtime_and_interface_class_metrics_fit_ast_budgets() {
    for checked in [
        crate::tests::capability_fixtures::capability_coverage_fixture(),
        portable_check::v0::check_program(portable_build::interface_composition_fixture().document)
            .unwrap(),
    ] {
        let core = portable_core_ir::lower_checked(&checked).unwrap();
        let selection = JavaCapabilitySelection::for_test(&core);
        let ast = Lowering::new(&core, &selection, java_capabilities())
            .lower()
            .unwrap();
        let verified = verify_unresolved_package(&JavaDialect, ast).unwrap();
        let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
        let certified = certify_resolved_package(&JavaDialect, linked).unwrap();
        let budgets = crate::tests::budget_oracle::collect(certified.ast());
        let rendered = render_certified_package(&crate::render::JavaRenderer, &certified).unwrap();
        let manifest = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        for file in rendered.files() {
            assert_eq!(
                file.contents(),
                manifest.file(file.path()).unwrap().contents(),
                "oracle must compile the exact AST used for accounting"
            );
        }
        crate::tests::budget_oracle::verify_manifest(&manifest, &budgets);
    }
}
