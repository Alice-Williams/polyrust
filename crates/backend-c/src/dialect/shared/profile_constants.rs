//! Bounded ordinary scalar constant objects, separate from owned-value globals.
use crate::ast::{
    CConstness, CFileRole, CInitializer, CInitializerKind, CLiteral, CObjectRef, CObjectTypeKind,
    CScalarType, CSignedLiteral, CValueKind,
};

pub(super) fn object(object: &CObjectRef) -> Result<(), String> {
    if object.file().key().role != CFileRole::GeneratedPublicHeader
        || object.ty().constness() != CConstness::Const
        || !matches!(
            object.ty().kind(),
            CObjectTypeKind::Scalar(CScalarType::Bool | CScalarType::I32 | CScalarType::I64)
        )
    {
        return Err("C scalar constant requires public-header const bool/i32/i64 storage".into());
    }
    Ok(())
}

pub(super) fn initializer(
    object_ref: &CObjectRef,
    initializer: &CInitializer,
) -> Result<(), String> {
    object(object_ref)?;
    let CInitializerKind::Expression(value) = initializer.kind() else {
        return Err("C scalar constant requires an exact scalar literal initializer".into());
    };
    let matches_type = matches!(
        (object_ref.ty().kind(), value.kind()),
        (
            CObjectTypeKind::Scalar(CScalarType::Bool),
            CValueKind::Literal(CLiteral::Bool(_))
        ) | (
            CObjectTypeKind::Scalar(CScalarType::I32),
            CValueKind::Literal(CLiteral::Signed(CSignedLiteral::I32(_)))
        ) | (
            CObjectTypeKind::Scalar(CScalarType::I64),
            CValueKind::Literal(CLiteral::Signed(CSignedLiteral::I64(_)))
        )
    );
    if !matches_type {
        return Err(
            "C scalar constant initializer must match its exact declared scalar type".into(),
        );
    }
    Ok(())
}
