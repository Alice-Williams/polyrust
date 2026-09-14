use super::*;

#[test]
fn documentation_coherence_is_required_by_the_real_package_verifier() {
    let mut origin = metadata();
    Arc::make_mut(&mut origin.crate_exports)
        .module_ancestries
        .clear();
    let errors = verify_unresolved_package(
        &JavaDialect,
        fixture(
            JavaPackage::RustCrate(7),
            GeneratedOrigin::RustSource(Arc::new(origin)),
        )
        .build(),
    )
    .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("export ancestry keys")),
        "{errors:?}"
    );
}

#[test]
fn every_registration_category_participates_in_documentation_validation() {
    for kind in [
        Registration::Type,
        Registration::Callable,
        Registration::InterfaceMethod,
        Registration::Value,
    ] {
        let mut builder = fixture(
            JavaPackage::RustCrate(7),
            GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
        );
        let mut origin = metadata();
        origin.module_ancestors = vec![].into();
        register(
            &mut builder,
            kind,
            GeneratedOrigin::RustSource(Arc::new(origin)),
        );
        let errors = JavaDialect.verify_package(&builder.build());
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("ancestry omits the crate root")),
            "{kind:?}: {errors:?}"
        );
    }
}
