//! Exact C prototype categories; no second scalar or known-object model.

use super::CKnownCall;
use crate::ast::{
    CConstness, CFunctionType, CKnownObject, CObjectType, CParameterType, CPointerTarget,
    CReturnType, CReturnValue, CScalarType,
};

impl CKnownCall {
    pub fn signature(self) -> CFunctionType {
        let scalar = CObjectType::scalar;
        let void_pointer = |qualifier| CObjectType::pointer(CPointerTarget::Void(qualifier));
        let stream = || {
            CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::known(
                CKnownObject::File,
            ))))
        };
        let size = || scalar(CScalarType::Size);
        let double = || scalar(CScalarType::F64);
        let integer = || scalar(CScalarType::Int);
        let writable = || void_pointer(CConstness::Unqualified);
        let readable = || void_pointer(CConstness::Const);
        let (result, parameters) = match self {
            Self::Allocate => (Some(writable()), vec![size()]),
            Self::Release => (None, vec![writable()]),
            Self::CopyBytes => (Some(writable()), vec![writable(), readable(), size()]),
            Self::CompareBytes => (Some(integer()), vec![readable(), readable(), size()]),
            Self::FloatRemainder => (Some(double()), vec![double(), double()]),
            Self::FloatTruncate => (Some(double()), vec![double()]),
            Self::IsNan | Self::SignBit => (Some(integer()), vec![double()]),
            Self::WriteBytes => (Some(size()), vec![readable(), size(), size(), stream()]),
            Self::StreamError => (Some(integer()), vec![stream()]),
        };
        CFunctionType::new(
            result.map_or(CReturnType::Void, |ty| {
                CReturnType::Value(CReturnValue::new(ty).expect("catalogue return category"))
            }),
            parameters
                .into_iter()
                .map(|ty| CParameterType::new(ty).expect("catalogue parameter category"))
                .collect(),
        )
    }
}
