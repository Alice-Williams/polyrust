//! Bounded ordinary scalar constant objects, separate from owned-value globals.
use crate::ast::{
    CConstness, CFileRole, CInitializer, CInitializerKind, CObjectRef, CObjectTypeKind,
    CScalarConstantValue, CScalarType,
};

pub(super) fn object(object: &CObjectRef) -> Result<(), String> {
    if object.file().key().role != CFileRole::GeneratedPublicHeader
        || object.ty().constness() != CConstness::Const
        || !matches!(
            object.ty().kind(),
            CObjectTypeKind::Scalar(
                CScalarType::Bool | CScalarType::I32 | CScalarType::I64 | CScalarType::F64
            )
        )
    {
        return Err(
            "C scalar constant requires public-header const bool/i32/i64/f64 storage".into(),
        );
    }
    Ok(())
}

pub(super) fn initializer(
    object_ref: &CObjectRef,
    initializer: &CInitializer,
) -> Result<(), String> {
    object(object_ref)?;
    let CInitializerKind::Expression(value) = initializer.kind() else {
        return Err("C scalar constant requires an exact scalar constant initializer".into());
    };
    let matches_type = CScalarConstantValue::from_expression(value)
        .is_some_and(|constant| object_ref.ty().kind() == constant.ty().kind());
    if !matches_type {
        return Err(
            "C scalar constant initializer must match its exact declared scalar type".into(),
        );
    }
    Ok(())
}
