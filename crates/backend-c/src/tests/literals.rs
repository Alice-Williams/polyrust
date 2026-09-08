//! Literal identities, width extremes and exact null-pointer categories.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CArrayLength, CConstness, CExpressions, CFunctionType, CKnownObject, CLiteral, CNullPointer,
    CObjectType, CPointerTarget, CRegistryError, CReturnType, CScalarType, CSignedLiteral,
    CTypeError, CUnsignedLiteral, CValueKind,
};

#[test]
fn every_signed_width_preserves_identity_at_its_extremes() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    use CSignedLiteral as S;
    let cases = [
        (
            S::PlainChar(i8::MIN),
            S::PlainChar(i8::MAX),
            CScalarType::PlainChar,
        ),
        (S::Int(i32::MIN), S::Int(i32::MAX), CScalarType::Int),
        (S::I8(i8::MIN), S::I8(i8::MAX), CScalarType::I8),
        (S::I16(i16::MIN), S::I16(i16::MAX), CScalarType::I16),
        (S::I32(i32::MIN), S::I32(i32::MAX), CScalarType::I32),
        (S::I64(i64::MIN), S::I64(i64::MAX), CScalarType::I64),
    ];
    for (min, max, scalar) in cases {
        for value in [min, max] {
            assert_eq!(CLiteral::Signed(value).ty(), CObjectType::scalar(scalar));
            assert_eq!(
                ast.literal(CLiteral::Signed(value)).unwrap().kind(),
                &CValueKind::Literal(CLiteral::Signed(value))
            );
        }
        assert_ne!(min, max);
    }
    assert_ne!(
        CLiteral::Signed(S::Int(0)).ty(),
        CLiteral::Signed(S::I32(0)).ty()
    );
}

#[test]
fn every_unsigned_width_preserves_identity_and_size_progress_literals() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    use CUnsignedLiteral as U;
    let cases = [
        (U::U8(0), U::U8(u8::MAX), CScalarType::U8),
        (U::U16(0), U::U16(u16::MAX), CScalarType::U16),
        (U::U32(0), U::U32(u32::MAX), CScalarType::U32),
        (U::U64(0), U::U64(u64::MAX), CScalarType::U64),
        (U::Size(0), U::Size(u64::MAX), CScalarType::Size),
    ];
    for (min, max, scalar) in cases {
        for value in [min, max] {
            assert_eq!(CLiteral::Unsigned(value).ty(), CObjectType::scalar(scalar));
            assert_eq!(
                ast.literal(CLiteral::Unsigned(value)).unwrap().kind(),
                &CValueKind::Literal(CLiteral::Unsigned(value))
            );
        }
        assert_ne!(min, max);
    }
    for value in [0, 1] {
        assert_eq!(
            CLiteral::Unsigned(U::Size(value)).ty(),
            CObjectType::scalar(CScalarType::Size)
        );
        assert_ne!(
            CLiteral::Unsigned(U::Size(value)).ty(),
            CLiteral::Unsigned(U::U64(value)).ty()
        );
    }
    for value in [false, true] {
        assert_eq!(
            ast.literal(CLiteral::Bool(value)).unwrap().kind(),
            &CValueKind::Literal(CLiteral::Bool(value))
        );
        assert_eq!(
            CLiteral::Bool(value).ty(),
            CObjectType::scalar(CScalarType::Bool)
        );
    }
    for value in [0, 127, 128, 255] {
        assert_eq!(
            ast.literal(CLiteral::CharByte(value)).unwrap().kind(),
            &CValueKind::Literal(CLiteral::CharByte(value))
        );
        assert_eq!(
            CLiteral::CharByte(value).ty(),
            CObjectType::scalar(CScalarType::U8)
        );
    }
}

#[test]
fn null_accepts_all_pointer_categories_but_not_scalar_array_or_known_objects() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let object = CObjectType::scalar(CScalarType::I32);
    let targets = [
        CPointerTarget::Void(CConstness::Unqualified),
        CPointerTarget::Void(CConstness::Const),
        CPointerTarget::Object(Box::new(object.clone())),
        CPointerTarget::Object(Box::new(
            object.clone().with_constness(CConstness::Const).unwrap(),
        )),
        CPointerTarget::Function(Box::new(CFunctionType::new(CReturnType::Void, vec![]))),
    ];
    for target in targets {
        let ty = CObjectType::pointer(target);
        let qualified = ty.clone().with_constness(CConstness::Const).unwrap();
        let null = CNullPointer::new(qualified.clone()).unwrap();
        assert_eq!(null.declared_type(), &qualified);
        assert_eq!(
            ast.literal(CLiteral::NullPointer(null.clone()))
                .unwrap()
                .kind(),
            &CValueKind::Literal(CLiteral::NullPointer(null.clone()))
        );
        assert_eq!(CLiteral::NullPointer(null).ty(), ty);
    }
    for ty in CScalarType::ALL
        .into_iter()
        .map(CObjectType::scalar)
        .chain([
            CObjectType::array(object, CArrayLength::new(1).unwrap()).unwrap(),
            CObjectType::known(CKnownObject::File),
            CObjectType::known(CKnownObject::MaxAlign),
        ])
    {
        assert_eq!(CNullPointer::new(ty), Err(CTypeError::ExpectedPointer));
    }
}

#[test]
fn null_aliases_keep_authentication_provenance() {
    let (mut owner, file) = registry();
    let target = CObjectType::pointer(CPointerTarget::Void(CConstness::Const));
    let alias = owner
        .register_typedef(&file, key("Context"), target.clone())
        .unwrap();
    let declared = CObjectType::typedef(alias);
    let null = CNullPointer::new(declared.clone()).unwrap();
    assert_eq!(null.ty(), &target);
    assert_eq!(null.declared_type(), &declared);
    let (foreign, _) = registry();
    assert_eq!(
        foreign.check_type(null.declared_type()),
        Err(CRegistryError::CrossRegistry)
    );
}
