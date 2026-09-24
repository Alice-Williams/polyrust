//! Closed representation proofs are deliberately narrower than Java syntax.
mod expressions;
mod fixture;
mod native;
mod publication;
mod syntax;
use crate::ast::*;
use crate::dialect::{JavaDialect, JavaScalarResultFamily};
use crate::tests::source_record_fixture::{int, name};
use expressions::*;
use fixture::*;
use portable_codegen::*;

#[test]
fn scalar_result_families_certify_original_closed_declarations_without_runtime() {
    let fixture = Fixture::new();
    let types = fixture.family.types;
    let other = fixture.other.types;
    let certificate = certify(fixture.finish());
    let family = JavaScalarResultFamily::from_certificate(certificate.clone(), types).unwrap();
    let alternate = JavaScalarResultFamily::from_certificate(certificate, other).unwrap();
    assert_eq!(family.types(), types);
    assert_eq!(family.payload(), payload(types.success));
    assert_eq!(family.payload_name(), &name("value"));
    assert_ne!(family.types(), alternate.types());
    let output = render_certified_package(&crate::render::JavaRenderer, family.package()).unwrap();
    assert_eq!(output.files().len(), 1);
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("text")
    };
    assert!(text.contains("sealed interface Outcome"));
    assert!(text.contains("record Success(int value)"));
    assert!(text.contains("record Error()"));
    assert!(!text.contains("Runtime"));
    assert_eq!(
        output,
        render_certified_package(&crate::render::JavaRenderer, family.package()).unwrap()
    );
}

#[test]
fn scalar_result_family_projection_retains_exact_adapter_registrations() {
    let fixture = Fixture::new();
    let types = fixture.family.types;
    let other = fixture.other.types;
    let draft = fixture.finish();
    let certificate = certify(draft.clone());
    let item = &certificate.ast().files()[0].items()[0];
    assert_eq!(item.source_inventory.iter().len(), 6);
    assert_eq!(
        item.source_inventory,
        JavaSourceInventory::derive(&draft, &item.item).unwrap()
    );
    for id in [
        types.interface,
        types.success,
        types.error,
        other.interface,
        other.success,
        other.error,
    ] {
        let Some(JavaSourceDeclaration::Type(registration)) =
            item.source_inventory.get(GeneratedSymbolId::Type(id))
        else {
            panic!("retained adapter")
        };
        assert_eq!(Some(registration), draft.generated_type(id));
        assert_eq!(
            registration.origin,
            GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter)
        );
    }
    let mut tampered = item.clone();
    tampered.source_inventory = JavaSourceInventory::default();
    assert_ne!(
        &tampered, item,
        "post-link exact equality includes adapter registrations"
    );
    verify_linked_package(certificate.ast()).unwrap();
}

#[test]
fn scalar_result_family_selection_cannot_mix_equal_shapes_or_repeat_identities() {
    for mutation in 0..4 {
        let fixture = Fixture::new();
        let mut types = fixture.family.types;
        match mutation {
            0 => types.success = fixture.other.types.success,
            1 => types.error = fixture.other.types.error,
            2 => types.interface = fixture.other.types.interface,
            3 => types.error = types.success,
            _ => unreachable!(),
        }
        assert!(
            JavaScalarResultFamily::from_certificate(certify(fixture.finish()), types).is_err()
        );
    }
}

#[test]
fn scalar_result_family_rejects_java_valid_custom_accessors_and_constructor_changes() {
    for mutation in 0..3 {
        let mut fixture = Fixture::new();
        let types = fixture.family.types;
        if mutation == 0 {
            fixture
                .family
                .success
                .members
                .push(JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Structural,
                    annotations: vec![],
                    modifiers: vec![JavaModifier::Public],
                    type_parameters: vec![],
                    return_type: int(),
                    name: name("value"),
                    parameters: vec![],
                    body: Some(JavaBlock::new(vec![returned(JavaExpr::literal(
                        int(),
                        JavaLiteral::I32(0),
                    ))])),
                }));
        } else {
            let JavaMember::Constructor(constructor) = &mut fixture.family.success.members[0]
            else {
                unreachable!()
            };
            if mutation == 1 {
                let JavaStmt::Assign { value, .. } = &mut constructor.body.statements[0] else {
                    unreachable!()
                };
                *value = JavaExpr::literal(int(), JavaLiteral::I32(7));
            } else {
                constructor.parameters[0].final_parameter = false;
            }
        }
        let certificate = certify(fixture.finish());
        let error = JavaScalarResultFamily::from_certificate(certificate, types).unwrap_err();
        assert!(
            error.contains(if mutation == 0 {
                "custom accessors"
            } else {
                "exact final input"
            }),
            "{error}"
        );
    }
}

#[test]
fn scalar_result_family_rejects_java_valid_third_implementor() {
    let mut fixture = Fixture::new();
    let types = fixture.family.types;
    fixture
        .family
        .interface
        .permits
        .push(reference(fixture.other.types.error));
    let JavaHeritage::Interfaces(interfaces) = &mut fixture.other.error.heritage else {
        unreachable!()
    };
    interfaces.push(reference(types.interface));
    let error =
        JavaScalarResultFamily::from_certificate(certify(fixture.finish()), types).unwrap_err();
    assert!(error.contains("exactly its two"), "{error}");
}

#[test]
fn scalar_result_generated_upcasts_authenticate_source_target_and_membership() {
    for mutation in 0..3 {
        let mut fixture = Fixture::new();
        let types = fixture.family.types;
        let JavaMember::Method(method) = &mut fixture.facade.members[0] else {
            unreachable!()
        };
        let JavaStmt::If { then_block, .. } = &mut method.body.as_mut().unwrap().statements[0]
        else {
            unreachable!()
        };
        let JavaStmt::Return(Some(value)) = &mut then_block.statements[0] else {
            unreachable!()
        };
        let JavaExprKind::InterfaceCoercion {
            implementation,
            target,
            ..
        } = &mut value.kind
        else {
            unreachable!()
        };
        match mutation {
            0 => {
                *implementation = JavaInterfaceWitness::for_generated_adapter(
                    fixture.other.types.success,
                    types.interface,
                )
            }
            1 => *target = reference(fixture.other.types.interface),
            2 => {
                *implementation = JavaInterfaceWitness::for_generated_adapter(
                    types.success,
                    fixture.other.types.interface,
                )
            }
            _ => unreachable!(),
        }
        let errors = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("interface coercion disagrees")),
            "{errors:?}"
        );
    }
}

#[test]
fn scalar_result_payload_access_requires_a_success_typed_pattern_binding() {
    let mut fixture = Fixture::new();
    let types = fixture.family.types;
    let JavaMember::Method(method) = &mut fixture.facade.members[2] else {
        unreachable!()
    };
    method
        .body
        .as_mut()
        .unwrap()
        .statements
        .insert(1, returned(read_payload(types.success, "success")));
    let errors = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap_err();
    assert!(
        errors.iter().any(|error| error.message.contains("scope")),
        "{errors:?}"
    );
}
