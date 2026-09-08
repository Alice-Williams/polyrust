//! Isolated synthetic-declaration accounting, with native controls elsewhere.

use super::check;
use super::tests::{declaration, item, method};
use crate::ast::{
    JavaArrayOwnership, JavaBlock, JavaConstructor, JavaDeclarationKind, JavaIdentifier,
    JavaLocalFinality, JavaMember, JavaModifier, JavaPrimitive, JavaRecordComponent,
    JavaRecordComponentOrigin, JavaRuntimeMember, JavaStmt, JavaType,
};

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
