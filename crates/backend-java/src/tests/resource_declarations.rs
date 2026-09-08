//! Isolated synthetic-declaration accounting, with native controls elsewhere.

use super::check;
use super::tests::{declaration, item, method};
use crate::ast::{
    JavaArrayOwnership, JavaBlock, JavaConstructor, JavaDeclarationKind, JavaIdentifier,
    JavaLocalFinality, JavaMember, JavaModifier, JavaPrimitive, JavaRecordComponent,
    JavaRecordComponentOrigin, JavaRuntimeMember, JavaStmt, JavaType,
};

#[test]
fn implicit_inner_constructor_descriptor_includes_outer_class_name() {
    let prefix = crate::ast::JavaPackage::Generated.name().len() + 1;
    for outer_length in [65_530, 65_531] {
        let mut child = declaration(vec![]);
        child.name = JavaIdentifier::new("A").unwrap();
        let mut outer = declaration(vec![JavaMember::NestedType(child)]);
        outer.name = JavaIdentifier::new("A".repeat(outer_length - prefix)).unwrap();
        let mut errors = Vec::new();
        let names = super::Names::new();
        super::declarations::Checker::new(&names, "Inner.java", &mut errors).declaration(
            &outer,
            &outer.members.iter().collect::<Vec<_>>(),
            prefix,
            None,
        );
        assert_eq!(errors.is_empty(), outer_length == 65_530, "{errors:?}");
        if outer_length == 65_531 {
            assert!(
                errors
                    .iter()
                    .any(|error| error.message.contains("method descriptor bytes"))
            );
        }
    }
}

#[test]
fn nongeneric_heritage_does_not_allocate_a_combined_signature_constant() {
    let mut builder = portable_codegen::TargetAstBuilder::new(crate::dialect::JavaDialect);
    let mut names = super::Names::new();
    let mut interfaces = Vec::new();
    for index in 0..2 {
        let id = builder.generated_type(portable_codegen::GeneratedType {
            name: format!("Interface{index}"),
            kind: JavaDeclarationKind::Interface,
            visibility: crate::ast::JavaVisibility::Public,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: portable_diagnostics::SourceRef::logical(["resource-signature"]),
        });
        names.insert(id, 40_000);
        interfaces.push(JavaType::Reference(crate::ast::JavaTypeName::Generated(id)));
    }
    let mut value = declaration(vec![]);
    value.heritage = crate::ast::JavaHeritage::Interfaces(interfaces);
    for generic in [false, true] {
        if generic {
            value
                .type_parameters
                .push(JavaIdentifier::new("T").unwrap());
        }
        let mut errors = Vec::new();
        super::declarations::Checker::new(&names, "Heritage.java", &mut errors).declaration(
            &value,
            &[],
            23,
            None,
        );
        assert_eq!(errors.is_empty(), !generic, "{errors:?}");
        if generic {
            assert!(
                errors
                    .iter()
                    .any(|error| error.message.contains("class generic signature bytes"))
            );
        }
    }
}

#[test]
fn generated_member_binary_names_include_the_package_prefix() {
    let owner = crate::dialect::JavaGeneratedContainer::PublicApi;
    let member = JavaIdentifier::new("A".repeat(65_527)).unwrap();
    assert_eq!(
        super::generated_member_length(owner, &member),
        crate::ast::JavaPackage::Generated.name().len()
            + 1
            + owner.text().len()
            + 1
            + member.as_str().len()
    );
    assert!(super::generated_member_length(owner, &member) > 65_535);
}

#[test]
fn canonical_record_constructor_slots_are_checked_without_explicit_members() {
    for count in [127, 128] {
        let mut record = declaration(vec![]);
        record.kind = JavaDeclarationKind::Record;
        record.record_components = (0..count)
            .map(|index| JavaRecordComponent {
                origin: JavaRecordComponentOrigin::Runtime(JavaRuntimeMember::ScalarValue),
                ty: JavaType::Primitive(JavaPrimitive::Long),
                name: JavaIdentifier::new(format!("p{index}")).unwrap(),
            })
            .collect();
        let file = item(record);
        assert_eq!(
            check(vec![("Fixture.java", vec![&file])]).is_ok(),
            count == 127
        );
    }
}

#[test]
fn constructor_slots_include_enum_and_enclosing_instance_parameters() {
    for (kind, nested, is_static, count, accepted) in [
        (JavaDeclarationKind::FinalClass, false, false, 254, true),
        (JavaDeclarationKind::FinalClass, false, false, 255, false),
        (JavaDeclarationKind::FinalClass, true, false, 253, true),
        (JavaDeclarationKind::FinalClass, true, false, 254, false),
        (JavaDeclarationKind::FinalClass, true, true, 254, true),
        (JavaDeclarationKind::Enum, false, false, 252, true),
        (JavaDeclarationKind::Enum, false, false, 253, false),
    ] {
        let JavaMember::Method(parameters) =
            method(JavaType::Primitive(JavaPrimitive::Int), count, false)
        else {
            unreachable!()
        };
        let constructor = JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Private],
            name: JavaIdentifier::new("Inner").unwrap(),
            parameters: parameters.parameters,
            body: JavaBlock::new(vec![]),
        });
        let mut inner = declaration(vec![constructor]);
        inner.kind = kind;
        inner.name = JavaIdentifier::new("Inner").unwrap();
        if is_static {
            inner.modifiers.push(JavaModifier::Static);
        }
        let file = item(if nested {
            declaration(vec![JavaMember::NestedType(inner)])
        } else {
            inner
        });
        assert_eq!(
            check(vec![("Fixture.java", vec![&file])]).is_ok(),
            accepted,
            "{kind:?}/{nested}/{is_static}/{count}"
        );
    }
}

#[test]
fn body_local_types_and_names_are_not_missed_by_header_checks() {
    for (dimensions, name, accepted) in [
        (255, "local".to_owned(), true),
        (256, "local".to_owned(), false),
        (1, "l".repeat(65_536), false),
    ] {
        let ty = (0..dimensions).fold(JavaType::Primitive(JavaPrimitive::Int), |component, _| {
            JavaType::Array {
                component: Box::new(component),
                ownership: JavaArrayOwnership::InternalMutable,
            }
        });
        let JavaMember::Method(mut value) =
            method(JavaType::Primitive(JavaPrimitive::Int), 0, false)
        else {
            unreachable!()
        };
        value.body = Some(JavaBlock::new(vec![JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty,
            name: JavaIdentifier::new(name).unwrap(),
            value: None,
        }]));
        let file = item(declaration(vec![JavaMember::Method(value)]));
        assert_eq!(check(vec![("Fixture.java", vec![&file])]).is_ok(), accepted);
    }
}

#[test]
fn runtime_fragment_members_share_the_class_capacity_walk() {
    let mut runtime = declaration(vec![]);
    runtime.name = JavaIdentifier::new("Runtime").unwrap();
    let shell = item(runtime);
    let fragment = crate::ast::JavaFileItem::RuntimeMembers {
        helper: crate::dialect::JavaRuntimeHelper::Core,
        members: vec![method(JavaType::Primitive(JavaPrimitive::Long), 128, false)],
    };
    let errors = check(vec![("Runtime.java", vec![&shell, &fragment])]).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("parameter slots"))
    );
}
