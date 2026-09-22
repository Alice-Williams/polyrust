//! Private inventory matrix isolates the constant contract from earlier verifiers.
use super::*;
use crate::ast::{JavaFileItem, JavaMember};
use crate::tests::{finite_constants as f, source_constant_fixture as c};

#[test]
fn finite_constant_inventory_checks_annotations_literals_modifiers_and_owners() {
    let api = c::admit(f::fixture(&[0], false).finish()).unwrap();
    let original = &api.package().ast().files()[0].items()[0];
    let JavaFileItem::Type { declaration, .. } = &original.item else {
        panic!("type")
    };
    let JavaMember::Field(field) = &declaration.members[1] else {
        panic!("field")
    };
    let public = api.constants().map(|value| value.declaration()).collect();
    let admit = |field: &JavaField, item: &ResolvedJavaFileItem| {
        verify(
            field,
            item,
            api.package_identity().namespace(),
            &declaration.name,
            &public,
        )
    };
    admit(field, original).unwrap();
    let types = [
        JavaPrimitive::Boolean,
        JavaPrimitive::Byte,
        JavaPrimitive::Char,
        JavaPrimitive::Int,
        JavaPrimitive::Long,
        JavaPrimitive::Double,
        JavaPrimitive::Void,
    ];
    let literals = [
        JavaLiteral::Boolean(false),
        JavaLiteral::I32(0),
        JavaLiteral::I64(0),
        f::literal(0),
        f::literal(1 << 63),
        JavaLiteral::CharScalar(0),
        JavaLiteral::String(String::new()),
        JavaLiteral::Utf16Units(vec![]),
        JavaLiteral::InternalNull(crate::ast::JavaNullPurpose::InternalSentinel),
    ];
    for ty in types {
        for value in &literals {
            let mut changed = field.clone();
            changed.initializer.as_mut().unwrap().ty = JavaType::primitive(ty);
            changed.initializer.as_mut().unwrap().kind = JavaExprKind::Literal(value.clone());
            assert_eq!(
                admit(&changed, original).is_ok(),
                ty == JavaPrimitive::Double && matches!(value, JavaLiteral::F64(_))
            );
        }
    }
    for wrong in [
        JavaType::primitive(JavaPrimitive::Long),
        JavaType::primitive(JavaPrimitive::Double).boxed(),
    ] {
        let mut changed = field.clone();
        changed.ty = wrong;
        assert!(admit(&changed, original).is_err());
    }
    for modifier in [
        JavaModifier::Public,
        JavaModifier::Static,
        JavaModifier::Final,
    ] {
        let mut changed = field.clone();
        changed.modifiers.retain(|current| *current != modifier);
        assert!(admit(&changed, original).is_err());
        let mut duplicate = field.clone();
        duplicate.modifiers.push(modifier);
        assert!(admit(&duplicate, original).is_err());
    }
    let symbol = TargetSymbolRef::Generated(GeneratedSymbolId::Value(field.declared.unwrap()));
    for changed_owner in [
        None,
        Some(JavaResolvedName::Local(field.name.clone())),
        Some(JavaResolvedName::DeclaredPath(JavaDeclaredPath {
            package: JavaPackage::RustCrate(999),
            owners: vec![declaration.name.clone()],
            member: field.name.clone(),
        })),
    ] {
        let mut item = original.clone();
        item.names.remove(&symbol);
        if let Some(name) = changed_owner {
            item.names.insert(symbol.clone(), name);
        }
        assert!(admit(field, &item).is_err());
    }
}
