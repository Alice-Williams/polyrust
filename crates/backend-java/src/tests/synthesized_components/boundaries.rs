use super::{fixture::*, rejects};
use crate::ast::*;
use crate::dialect::{JavaDependencyApi, JavaDialect};
use crate::tests::source_record_fixture::{add_file, int, name};
use portable_codegen::*;

#[test]
fn synthesized_private_fields_cannot_cross_nests_even_for_public_records() {
    let mut fixture = Fixture::new();
    let mut other = fixture.facade.clone();
    other.declared = None;
    other.name = name("OtherNest");
    other.members = vec![method(
        "read",
        int(),
        vec![parameter("item", ty(fixture.owner))],
        read(
            JavaExpr::local(ty(fixture.owner), name("item")),
            fixture.owner,
        ),
    )];
    add_file(&mut fixture.builder, "OtherNest", vec![], other);
    rejects(
        fixture,
        "private instance field is referenced outside its top-level nest",
    );
}

#[test]
fn synthesized_record_shape_does_not_admit_structural_read_or_nominal_substitution() {
    for structural in [true, false] {
        let mut fixture = Fixture::new();
        if structural {
            let mut value = read(
                JavaExpr::local(ty(fixture.owner), name("item")),
                fixture.owner,
            );
            let JavaExprKind::Field { field, .. } = &mut value.kind else {
                unreachable!()
            };
            *field = JavaFieldRef::Structural {
                name: name("value"),
                ty: int(),
            };
            fixture.facade.members.push(method(
                "bad",
                int(),
                vec![parameter("item", ty(fixture.owner))],
                value,
            ));
            rejects(fixture, "structural Java field reference");
        } else {
            fixture.facade.members.push(method(
                "bad",
                ty(fixture.owner),
                vec![parameter("other", ty(fixture.other_id))],
                JavaExpr::local(ty(fixture.other_id), name("other")),
            ));
            let errors = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap_err();
            assert!(
                errors.iter().any(|error| error.message.contains("return")),
                "{errors:?}"
            );
        }
    }
}

#[test]
fn source_facade_cannot_publish_synthesized_nominals_as_private_source_records() {
    use crate::tests::source_dependency_fixture as source;
    for visibility in [JavaVisibility::Private, JavaVisibility::Public] {
        let draft = source::package_with(7, source::functions(42), |builder, declared, facade| {
            let owner = register(
                builder,
                "Payload",
                JavaDeclarationKind::Record,
                visibility,
                SynthesisReason::InterfaceAdapter,
            );
            declared.push(GeneratedSymbolId::Type(owner));
            facade.members.push(JavaMember::NestedType(build_record(
                owner, "Payload", visibility,
            )));
        });
        let certificate = certify(draft);
        let error = JavaDependencyApi::from_certificate(certificate).unwrap_err();
        assert!(error.contains("record"), "{visibility:?}: {error}");
    }
}

#[test]
fn synthesized_component_does_not_skip_method_capacity_certification() {
    for count in [255, 256] {
        let mut fixture = Fixture::new();
        fixture.facade.members.push(method(
            "many",
            int(),
            (0..count)
                .map(|n| parameter(&format!("p{n}"), int()))
                .collect(),
            JavaExpr::literal(int(), JavaLiteral::I32(0)),
        ));
        let verified = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap();
        let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
        let result = certify_resolved_package(&JavaDialect, linked);
        if count == 255 {
            assert!(result.is_ok());
        } else {
            let errors = result.unwrap_err();
            assert!(
                errors.iter().any(|error| error.code
                    == portable_diagnostics::DiagnosticCode::TargetResourceLimit
                    && error.message.contains("parameter slots")),
                "{errors:?}"
            );
        }
    }
}
