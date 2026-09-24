//! Selection validation is separate from syntax certification.
use super::{fixture::*, model::*};
use portable_backend_java::{ast::*, dialect::*};
use portable_codegen::*;

#[test]
fn role_selections_reject_duplicates_swaps_foreign_constants_and_type_aliases() {
    let family = checked(Fault::None);
    let fixture = Fixture::new(Fault::None);
    let original_kinds = fixture.kinds;
    let mut kinds = original_kinds;
    let types = fixture.types;
    let package = certify(fixture.finish()).unwrap();
    kinds.zero = kinds.empty;
    assert!(
        JavaErrorResultFamily::from_certificate(package.clone(), types, kinds)
            .unwrap_err()
            .contains("distinct")
    );
    kinds = original_kinds;
    std::mem::swap(&mut kinds.positive_overflow, &mut kinds.negative_overflow);
    assert!(
        JavaErrorResultFamily::from_certificate(package.clone(), types, kinds)
            .unwrap_err()
            .contains("registration")
    );
    let mut bad_types = family.types();
    bad_types.error = bad_types.success;
    assert!(
        JavaErrorResultFamily::from_certificate(package, bad_types, original_kinds)
            .unwrap_err()
            .contains("distinct")
    );

    let mut fixture = Fixture::new(Fault::None);
    let foreign_owner = register(&mut fixture.builder, "Foreign", JavaDeclarationKind::Enum);
    let foreign = fixture.builder.value(GeneratedValue {
        name: "FOREIGN_ZERO".into(),
        ty: TargetTypeRef::Generated(foreign_owner),
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
        source: source(),
    });
    let mut declaration = declaration(foreign_owner, "Foreign", JavaDeclarationKind::Enum);
    declaration
        .members
        .push(JavaMember::EnumConstant(JavaEnumConstant {
            declared: foreign,
            name: name("FOREIGN_ZERO"),
        }));
    fixture
        .facade
        .members
        .push(JavaMember::NestedType(declaration));
    let mut kinds = fixture.kinds;
    kinds.zero = foreign;
    let types = fixture.types;
    assert!(
        JavaErrorResultFamily::from_certificate(certify(fixture.finish()).unwrap(), types, kinds)
            .unwrap_err()
            .contains("selected enum")
    );
}

#[test]
fn changed_enum_inventory_storage_and_conformance_never_publish_a_family() {
    for mutation in 0..10 {
        let mut fixture = if mutation == 9 {
            Fixture::with_value_origin(Fault::None, SynthesisReason::TestHarness)
        } else {
            Fixture::new(Fault::None)
        };
        match mutation {
            0 => {
                fixture.error.members.pop();
            }
            1 => {
                fixture.error.members.push(fixture.error.members[0].clone());
            }
            2 => {
                fixture.error.heritage = JavaHeritage::None;
            }
            3 => {
                fixture.interface.permits.pop();
            }
            4 => {
                fixture.error.type_parameters.push(name("T"));
            }
            5 => {
                fixture.error.members.push(JavaMember::Field(JavaField {
                    declared: None,
                    modifiers: vec![
                        JavaModifier::Private,
                        JavaModifier::Static,
                        JavaModifier::Final,
                    ],
                    ty: int(),
                    name: name("extra"),
                    initializer: Some(JavaExpr::literal(int(), JavaLiteral::I32(7))),
                }));
            }
            6 => {
                let JavaMember::Constructor(ctor) = &mut fixture.success.members[0] else {
                    panic!("constructor")
                };
                let JavaStmt::Assign { value, .. } = &mut ctor.body.statements[0] else {
                    panic!("assignment")
                };
                *value = JavaExpr::literal(int(), JavaLiteral::I32(0));
            }
            7 => {
                fixture.error.kind = JavaDeclarationKind::Record;
            }
            8 => {
                fixture.error.visibility = JavaVisibility::Private;
            }
            9 => {}
            _ => unreachable!(),
        }
        let (types, kinds) = (fixture.types, fixture.kinds);
        let admitted = certify(fixture.finish())
            .and_then(|package| JavaErrorResultFamily::from_certificate(package, types, kinds));
        assert!(admitted.is_err(), "mutation {mutation}");
    }
}

#[test]
fn extra_well_typed_constant_and_old_empty_error_record_fail_the_new_profile() {
    let mut fixture = Fixture::new(Fault::None);
    let extra = fixture.builder.value(GeneratedValue {
        name: "EXTRA".into(),
        ty: TargetTypeRef::Generated(fixture.types.error),
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
        source: source(),
    });
    fixture
        .error
        .members
        .push(JavaMember::EnumConstant(JavaEnumConstant {
            declared: extra,
            name: name("EXTRA"),
        }));
    let (types, kinds) = (fixture.types, fixture.kinds);
    let package = certify(fixture.finish()).unwrap();
    assert!(
        JavaErrorResultFamily::from_certificate(package, types, kinds)
            .unwrap_err()
            .contains("six")
    );

    // A second, independently valid payload-free family keeps every original
    // registration placed and proves that the old profile still accepts it.
    let mut fixture = Fixture::new(Fault::None);
    let types = JavaScalarResultTypes {
        interface: register(
            &mut fixture.builder,
            "OldOutcome",
            JavaDeclarationKind::SealedInterface,
        ),
        success: register(
            &mut fixture.builder,
            "OldSuccess",
            JavaDeclarationKind::Record,
        ),
        error: register(
            &mut fixture.builder,
            "OldError",
            JavaDeclarationKind::Record,
        ),
    };
    let mut interface = declaration(
        types.interface,
        "OldOutcome",
        JavaDeclarationKind::SealedInterface,
    );
    interface.permits = vec![reference(types.success), reference(types.error)];
    let field = JavaSynthesizedField {
        owner: types.success,
        role: JavaSynthesizedFieldRole::ScalarResultPayload,
    };
    let mut success = fixture.success.clone();
    success.declared = Some(types.success);
    success.name = name("OldSuccess");
    success.heritage = JavaHeritage::Interfaces(vec![reference(types.interface)]);
    success.record_components[0].origin = JavaRecordComponentOrigin::Synthesized(field);
    let JavaMember::Constructor(ctor) = &mut success.members[0] else {
        panic!("constructor")
    };
    ctor.name = name("OldSuccess");
    let JavaStmt::Assign { target, .. } = &mut ctor.body.statements[0] else {
        panic!("assignment")
    };
    let JavaExprKind::Field {
        receiver,
        field: selected,
    } = &mut target.kind
    else {
        panic!("field")
    };
    receiver.ty = reference(types.success);
    *selected = JavaFieldRef::Synthesized {
        field,
        name: name("value"),
        ty: int(),
    };
    let mut error = declaration(types.error, "OldError", JavaDeclarationKind::Record);
    error.heritage = JavaHeritage::Interfaces(vec![reference(types.interface)]);
    error.members.push(JavaMember::Constructor(JavaConstructor {
        name: name("OldError"),
        modifiers: vec![JavaModifier::Public],
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    }));
    fixture.facade.members.extend(
        [interface, success, error]
            .into_iter()
            .map(JavaMember::NestedType),
    );
    let kinds = fixture.kinds;
    let package = certify(fixture.finish()).unwrap();
    assert!(JavaScalarResultFamily::from_certificate(package.clone(), types).is_ok());
    assert!(JavaErrorResultFamily::from_certificate(package, types, kinds).is_err());
}
