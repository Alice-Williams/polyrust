use crate::ast::*;
use crate::dialect::JavaDialect;
use crate::tests::source_record_fixture as fixture;
use portable_codegen::*;
use std::sync::Arc;
#[path = "source_records/native.rs"]
mod native;
use fixture::*;

fn rejects(fixture: Fixture, message: &str) {
    let errors = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap_err();
    assert!(
        errors.iter().any(|error| error.message.contains(message)),
        "expected {message}: {errors:?}"
    );
}

#[test]
fn source_component_documentation_cannot_disagree_with_its_crate() {
    let mut fixture = Fixture::new();
    let JavaRecordComponentOrigin::RustSource(field) =
        &mut fixture.record.record_components[0].origin
    else {
        unreachable!()
    };
    let origin = Arc::make_mut(&mut field.origin);
    let mut module = (*origin.module_ancestors[0]).clone();
    module
        .documentation
        .push("conflicting shared module payload".into());
    origin.module_ancestors = vec![Arc::new(module)].into();
    rejects(fixture, "conflicting module documentation");
}

#[test]
fn invalid_field_origin_stops_java_metadata_inspection_before_later_fields() {
    let mut fixture = Fixture::new();
    for (index, component) in fixture.record.record_components.iter_mut().enumerate() {
        let JavaRecordComponentOrigin::RustSource(field) = &mut component.origin else {
            unreachable!()
        };
        let origin = Arc::make_mut(&mut field.origin);
        if index == 0 {
            origin.node = RustSourceNode::Binding(0);
        } else {
            origin.declaration.crate_id = 99;
        }
    }
    let package = fixture.finish();
    let calls = std::cell::Cell::new(0);
    let fields = crate::ast::source_fields::origins(&package).inspect(|_| {
        calls.set(calls.get() + 1);
        assert_eq!(calls.get(), 1, "inspected the field tail after rejection");
    });
    assert!(CheckedRustDocumentation::check(fields).is_err());
    assert_eq!(calls.get(), 1);
    let errors = JavaDialect.verify_package(&package);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("declaration origin"))
    );
    assert!(
        !errors
            .iter()
            .any(|error| error.message.contains("own RustCrate namespace")),
        "the Java wrapper inspected a later field: {errors:?}"
    );
}

#[test]
fn source_component_owner_and_origin_mutations_reject() {
    for mutation in 0..8 {
        let mut fixture = Fixture::new();
        let JavaRecordComponentOrigin::RustSource(field) =
            &mut fixture.record.record_components[0].origin
        else {
            unreachable!();
        };
        let expected = match mutation {
            0 => {
                field.owner = id(88);
                "registered source nominal owner"
            }
            1 => {
                Arc::make_mut(&mut field.origin).declaration = id(2);
                "multiple Java registrations"
            }
            2 => {
                Arc::make_mut(&mut field.origin).module = id(88);
                "registered source nominal owner"
            }
            3 => {
                Arc::make_mut(&mut Arc::make_mut(&mut field.origin).crate_exports).root = id(88);
                "registered source nominal owner"
            }
            4 => {
                Arc::make_mut(&mut field.origin).node = RustSourceNode::Binding(1);
                "declaration origin"
            }
            5 => {
                Arc::make_mut(&mut field.origin).declaration.crate_id = 8;
                "own RustCrate namespace"
            }
            6 => {
                Arc::make_mut(&mut field.origin).declaration = id(4);
                "multiple Java registrations"
            }
            7 => {
                fixture.record.record_components[0].origin =
                    JavaRecordComponentOrigin::Runtime(JavaRuntimeMember::ScalarValue);
                "registered source nominal owner"
            }
            _ => unreachable!(),
        };
        rejects(fixture, expected);
    }
}

#[test]
fn source_field_references_require_owner_identity_spelling_and_type() {
    for mutation in 0..6 {
        let mut fixture = Fixture::new();
        let record_id = fixture.record_id;
        let mut reference = reference(record_id, 3, "value", int());
        let JavaFieldRef::RustSource {
            owner,
            field: field_id,
            name,
            ty,
        } = &mut reference
        else {
            unreachable!();
        };
        match mutation {
            0 => *owner = fixture.facade_id,
            1 => *field_id = id(88),
            2 => *name = fixture::name("missing"),
            3 => *ty = boolean(),
            4 => {
                *field_id = RustDeclarationId {
                    crate_id: 8,
                    ..id(3)
                }
            }
            5 => {
                let mut foreign = fixture.builder.clone();
                *owner = foreign.generated_type(GeneratedType {
                    name: "Unregistered".into(),
                    kind: JavaDeclarationKind::Record,
                    visibility: JavaVisibility::Private,
                    origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
                    source: source(),
                });
                assert!(
                    fixture
                        .builder
                        .clone()
                        .build()
                        .generated_type(*owner)
                        .is_none()
                );
            }
            _ => unreachable!(),
        }
        let mut probe = method(
            "probe",
            int(),
            vec![JavaStmt::Return(Some(field(
                this(record_id),
                reference,
                int(),
            )))],
        );
        probe.modifiers = vec![JavaModifier::Private];
        fixture.record.members.push(JavaMember::Method(probe));
        rejects(fixture, "RustSource Java field reference");
    }
}

#[test]
fn source_fields_retain_constructor_final_assignment_rules() {
    for mutation in 0..3 {
        let mut fixture = Fixture::new();
        let expected = match mutation {
            0 => {
                fixture.constructor().body.statements.remove(0);
                "without assigning blank final field"
            }
            1 => {
                let duplicate = fixture.constructor().body.statements[0].clone();
                fixture.constructor().body.statements.push(duplicate);
                "more than once"
            }
            2 => {
                let mut mutate = method(
                    "mutate",
                    JavaType::primitive(JavaPrimitive::Void),
                    vec![JavaStmt::Assign {
                        target: field(
                            this(fixture.record_id),
                            reference(fixture.record_id, 3, "value", int()),
                            int(),
                        ),
                        value: JavaExpr::literal(int(), JavaLiteral::I32(2)),
                    }],
                );
                mutate.modifiers = vec![JavaModifier::Private];
                fixture.record.members.push(JavaMember::Method(mutate));
                "declaring constructor"
            }
            _ => unreachable!(),
        };
        rejects(fixture, expected);
    }
}

#[test]
fn structural_fields_cannot_bypass_source_identity_for_reads_or_initialization() {
    for constructor in [false, true] {
        let mut fixture = Fixture::new();
        let reference = JavaFieldRef::Structural {
            name: name("value"),
            ty: int(),
        };
        if constructor {
            let JavaStmt::Assign { target, .. } = &mut fixture.constructor().body.statements[0]
            else {
                unreachable!();
            };
            let JavaExprKind::Field { field, .. } = &mut target.kind else {
                unreachable!();
            };
            *field = reference;
        } else {
            let mut probe = method(
                "probe",
                int(),
                vec![JavaStmt::Return(Some(field(
                    this(fixture.record_id),
                    reference,
                    int(),
                )))],
            );
            probe.modifiers = vec![JavaModifier::Private];
            fixture.record.members.push(JavaMember::Method(probe));
        }
        rejects(fixture, "structural Java field reference");
    }
}

#[test]
fn source_components_cannot_attach_to_an_unregistered_owner() {
    let mut fixture = Fixture::new();
    fixture.record.declared = None;
    rejects(fixture, "registered source nominal owner");
}

#[test]
fn source_components_cannot_attach_to_a_synthesized_nominal_owner() {
    rejects(
        Fixture::with_nominal_origin(Some(GeneratedOrigin::Synthesized(
            SynthesisReason::TestHarness,
        ))),
        "registered source nominal owner",
    );
}

#[test]
fn source_record_constructor_signature_and_field_types_remain_checked() {
    let mut fixture = Fixture::new();
    fixture.constructor().parameters[0].ty = boolean();
    rejects(fixture, "canonical component signature");
    let mut fixture = Fixture::new();
    fixture.record.record_components[0].ty = boolean();
    rejects(fixture, "RustSource Java field reference");
}

#[test]
fn private_source_record_fields_cannot_cross_top_level_nests() {
    let mut fixture = Fixture::new();
    let owner = fixture.record_id;
    let receiver = JavaExpr::local(
        JavaType::Reference(JavaTypeName::Generated(owner)),
        name("cell"),
    );
    let mut probe = method(
        "probe",
        int(),
        vec![JavaStmt::Return(Some(field(
            receiver.clone(),
            reference(owner, 3, "value", int()),
            int(),
        )))],
    );
    probe.parameters = vec![JavaParameter {
        ty: receiver.ty,
        name: name("cell"),
        final_parameter: true,
    }];
    let mut other = fixture.facade.clone();
    other.declared = None;
    other.name = name("Other");
    other.members = vec![JavaMember::Method(probe)];
    add_file(&mut fixture.builder, "Other", vec![], other);
    rejects(
        fixture,
        "private instance field is referenced outside its top-level nest",
    );
}
