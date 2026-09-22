//! Direct inventory projection, independently of general expression verification.
use super::*;
use crate::ast::{
    JavaExprKind, JavaFileItem, JavaMember, JavaPrecedence, JavaPrimitive, JavaType, JavaValueRef,
};
use crate::dialect::JavaKnownField;
use crate::tests::{infinite_constants as f, source_constant_fixture as c};
use portable_binary64::Binary64Sign;

#[test]
fn infinity_inventory_requires_exact_standard_field_type_shape_and_owner() {
    let api = c::admit(f::fixture(false).finish()).unwrap();
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
    for sign in [Binary64Sign::Positive, Binary64Sign::Negative] {
        let mut changed = field.clone();
        changed.initializer = Some(f::expression(sign));
        assert_eq!(
            admit(&changed, original).unwrap().value,
            JavaScalarConstantValue::Infinity(sign)
        );
        for ty in [
            JavaPrimitive::Boolean,
            JavaPrimitive::Int,
            JavaPrimitive::Long,
        ] {
            let mut malformed = changed.clone();
            malformed.initializer.as_mut().unwrap().ty = JavaType::primitive(ty);
            assert!(admit(&malformed, original).is_err());
        }
        changed.initializer.as_mut().unwrap().precedence = JavaPrecedence::Additive;
        assert!(admit(&changed, original).is_err());
    }
    for known in [
        JavaKnownField::IntegerMinValue,
        JavaKnownField::LongMaxValue,
        JavaKnownField::StandardCharsetsUtf8,
        JavaKnownField::CodingErrorReport,
    ] {
        let mut changed = field.clone();
        changed.initializer.as_mut().unwrap().kind =
            JavaExprKind::Value(JavaValueRef::KnownField(known));
        assert!(admit(&changed, original).is_err());
    }
    for modifier in [
        JavaModifier::Public,
        JavaModifier::Static,
        JavaModifier::Final,
    ] {
        let mut changed = field.clone();
        changed.modifiers.retain(|value| *value != modifier);
        assert!(admit(&changed, original).is_err());
        let mut duplicate = field.clone();
        duplicate.modifiers.push(modifier);
        assert!(admit(&duplicate, original).is_err());
    }
    let symbol = TargetSymbolRef::Generated(GeneratedSymbolId::Value(field.declared.unwrap()));
    for owner in [
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
        if let Some(owner) = owner {
            item.names.insert(symbol.clone(), owner);
        }
        assert!(admit(field, &item).is_err());
    }
}
