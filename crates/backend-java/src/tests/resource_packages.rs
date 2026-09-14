//! Actual namespace lengths enter synthetic JVM descriptor reservations.
use super::{Names, check_with_names};
use crate::ast::{JavaDeclarationKind, JavaIdentifier, JavaPackage};

#[test]
fn relative_generated_member_budget_uses_its_containing_package() {
    for package in [JavaPackage::Generated, JavaPackage::RustCrate(1)] {
        let member = JavaIdentifier::new("Member").unwrap();
        assert_eq!(
            super::generated_member_length(
                package,
                crate::dialect::JavaGeneratedContainer::PublicApi,
                &member
            ),
            package.name().len() + ".Generated.Member".len(),
        );
    }
}

#[test]
fn relative_generated_binary_names_have_exact_capacity_boundaries() {
    use portable_codegen::{GeneratedOrigin, GeneratedType, SynthesisReason, TargetAstBuilder};
    for package in [
        JavaPackage::Generated,
        JavaPackage::RustCrate(0),
        JavaPackage::RustCrate(u64::MAX),
    ] {
        for length in [65_535, 65_536] {
            let member = JavaIdentifier::new(
                "M".repeat(length - package.name().len() - ".Generated.".len()),
            )
            .unwrap();
            let actual = super::generated_member_length(
                package,
                crate::dialect::JavaGeneratedContainer::PublicApi,
                &member,
            );
            assert_eq!(actual, length);
            let mut builder = TargetAstBuilder::new(crate::dialect::JavaDialect);
            let id = builder.generated_type(GeneratedType {
                name: member.as_str().into(),
                kind: JavaDeclarationKind::FinalClass,
                visibility: crate::ast::JavaVisibility::Public,
                origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
                source: portable_diagnostics::SourceRef::logical(["package-binary-capacity"]),
            });
            let mut declaration = super::tests::declaration(vec![]);
            declaration.declared = Some(id);
            declaration.name = member;
            let names = Names::from([(id, actual)]);
            let mut errors = Vec::new();
            super::declarations::Checker::new(&names, "Fixture.java", &mut errors).declaration(
                &declaration,
                &[],
                package.name().len() + 1,
                None,
            );
            assert_eq!(
                errors.is_empty(),
                length == 65_535,
                "{package:?}: {errors:?}"
            );
            if length == 65_536 {
                assert!(errors.iter().any(|error| {
                    error
                        .message
                        .contains("binary class name bytes requires 65536; limit is 65535")
                }));
            }
        }
    }
}

#[test]
fn record_descriptor_boundary_uses_the_actual_package_prefix() {
    for module in [
        JavaPackage::Generated,
        JavaPackage::RustCrate(0),
        JavaPackage::RustCrate(u64::MAX),
    ] {
        let prefix = module.name().len() + 1;
        for length in [65_512, 65_513] {
            let mut declaration = super::tests::declaration(vec![]);
            declaration.kind = JavaDeclarationKind::Record;
            declaration.name = JavaIdentifier::new("R".repeat(length - prefix)).unwrap();
            let item = super::tests::item(declaration);
            let result = check_with_names(
                vec![("Record.java", prefix, vec![&item])],
                Names::new(),
                Names::new(),
                vec![],
            );
            assert_eq!(result.is_ok(), length == 65_512, "{module:?}: {result:?}");
        }
    }
}
