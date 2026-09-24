//! Synthesized fields are authenticated target members, not source identities.
mod boundaries;
mod fixture;
mod native;
use crate::ast::*;
use crate::dialect::{JavaDependencyApi, JavaDialect};
use crate::tests::source_record_fixture::{int, name};
use fixture::*;
use portable_codegen::*;

fn rejects(fixture: Fixture, message: &str) {
    let errors = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap_err();
    assert!(
        errors.iter().any(|error| error.message.contains(message)),
        "{message}: {errors:?}"
    );
}

#[test]
fn synthesized_components_certify_without_runtime_or_dependency_publication() {
    let certificate = certify(Fixture::new().finish());
    let rendered = render_certified_package(&crate::render::JavaRenderer, &certificate).unwrap();
    assert_eq!(
        rendered,
        render_certified_package(&crate::render::JavaRenderer, &certificate).unwrap()
    );
    assert_eq!(rendered.files().len(), 1);
    let OutputContents::Text(text) = rendered.files()[0].contents() else {
        panic!("text")
    };
    assert!(text.contains("public record Payload(int value)"));
    assert!(text.contains("item.value()"));
    assert!(!text.contains("Runtime"));
    assert!(JavaDependencyApi::from_certificate(certificate).is_err());
}

#[test]
fn synthesized_component_declarations_require_exact_owner_storage_and_unique_role() {
    for mutation in 0..5 {
        let mut fixture = Fixture::new();
        match mutation {
            0 => {
                fixture.record.record_components[0].origin =
                    JavaRecordComponentOrigin::Synthesized(field_id(fixture.other_id))
            }
            1 => fixture.record.record_components[0].ty = JavaType::primitive(JavaPrimitive::Long),
            2 => fixture.record.declared = None,
            3 => fixture.record.type_parameters.push(name("T")),
            4 => {
                let mut duplicate = fixture.record.record_components[0].clone();
                duplicate.name = name("another");
                fixture.record.record_components.push(duplicate);
            }
            _ => unreachable!(),
        }
        rejects(
            fixture,
            if mutation == 4 {
                "record component"
            } else {
                "exact synthesized owner/role/type"
            },
        );
    }
    let mut source = crate::tests::source_record_fixture::Fixture::new();
    source.record.record_components[0].origin =
        JavaRecordComponentOrigin::Synthesized(field_id(source.record_id));
    let errors = verify_unresolved_package(&JavaDialect, source.finish()).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("exact synthesized owner/role/type"))
    );
}

#[test]
fn synthesized_field_reads_authenticate_owner_name_type_and_receiver() {
    for mutation in 0..5 {
        let mut fixture = Fixture::new();
        let mut expression = read(
            JavaExpr::local(ty(fixture.owner), name("item")),
            fixture.owner,
        );
        let JavaExprKind::Field {
            field:
                JavaFieldRef::Synthesized {
                    field,
                    name: spelling,
                    ty: field_type,
                },
            receiver,
        } = &mut expression.kind
        else {
            unreachable!()
        };
        match mutation {
            0 => field.owner = fixture.other_id,
            1 => *spelling = name("missing"),
            2 => *field_type = JavaType::primitive(JavaPrimitive::Long),
            3 => receiver.ty = ty(fixture.other_id),
            4 => field.owner = fixture.facade_id,
            _ => unreachable!(),
        }
        fixture.facade.members.push(method(
            "bad",
            int(),
            vec![parameter("item", ty(fixture.owner))],
            expression,
        ));
        rejects(fixture, "synthesized Java field reference");
    }
}

#[test]
fn synthesized_accessors_authenticate_identity_and_complete_signature() {
    for mutation in 0..8 {
        let mut fixture = Fixture::new();
        let mut expression = accessor(
            JavaExpr::local(ty(fixture.owner), name("item")),
            fixture.owner,
        );
        let JavaExprKind::Call {
            callable:
                JavaCallableRef::Member {
                    owner,
                    name: spelling,
                    signature,
                    origin,
                },
            ..
        } = &mut expression.kind
        else {
            unreachable!()
        };
        match mutation {
            0 => *origin = JavaMemberOrigin::SynthesizedField(field_id(fixture.other_id)),
            1 => *spelling = name("missing"),
            2 => signature.result = JavaType::primitive(JavaPrimitive::Long),
            3 => signature.parameters.push(int()),
            4 => signature.nullable_result = true,
            5 => signature.pure = false,
            6 => signature
                .checked_exceptions
                .push(JavaKnownType::CharacterCodingException),
            7 => *owner = ty(fixture.other_id),
            _ => unreachable!(),
        }
        fixture.facade.members.push(method(
            "bad",
            int(),
            vec![parameter("item", ty(fixture.owner))],
            expression,
        ));
        let errors = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("callable")),
            "{mutation}: {errors:?}"
        );
    }
}

#[test]
fn synthesized_components_keep_final_assignment_and_structural_bypass_guards() {
    for mutation in 0..5 {
        let mut fixture = Fixture::new();
        let expected = match mutation {
            0 => {
                fixture.constructor().body.statements.clear();
                "without assigning blank final field"
            }
            1 => {
                let duplicate = fixture.constructor().body.statements[0].clone();
                fixture.constructor().body.statements.push(duplicate);
                "more than once"
            }
            2 => {
                let JavaStmt::Assign { target, .. } = &mut fixture.constructor().body.statements[0]
                else {
                    unreachable!()
                };
                let JavaExprKind::Field { field, .. } = &mut target.kind else {
                    unreachable!()
                };
                *field = JavaFieldRef::Structural {
                    name: name("value"),
                    ty: int(),
                };
                "structural Java field reference"
            }
            3 => {
                let JavaStmt::Assign { target, value } =
                    &mut fixture.constructor().body.statements[0]
                else {
                    unreachable!()
                };
                *value = target.clone();
                "before"
            }
            4 => {
                let assignment = fixture.constructor().body.statements[0].clone();
                let mut mutator = method(
                    "mutate",
                    int(),
                    vec![parameter("value", int())],
                    JavaExpr::literal(int(), JavaLiteral::I32(0)),
                );
                let JavaMember::Method(method) = &mut mutator else {
                    unreachable!()
                };
                method.modifiers = vec![JavaModifier::Private];
                method
                    .body
                    .as_mut()
                    .unwrap()
                    .statements
                    .insert(0, assignment);
                fixture.record.members.push(mutator);
                "declaring constructor"
            }
            _ => unreachable!(),
        };
        rejects(fixture, expected);
    }
}
