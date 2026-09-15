//! One-way construction plus reverse maps: C registrations own every signature.

use super::{CDialect, CInvocation, CStdType};
use crate::ast::{
    CFunctionRef, CLocalRef, CMemberRef, CObjectType, CObjectTypeKind, CParameterRef, CReturnType,
    CScalarType, CStructRef,
};
use portable_codegen::{
    GeneratedCallableId, GeneratedSymbolId, GeneratedTypeId, GeneratedValueId,
    TargetCallableSignature, TargetTypeRef,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CBindings {
    pub imports: BTreeMap<CFunctionRef, super::CImportedCallable>,
    pub types: BTreeMap<CStructRef, GeneratedTypeId>,
    pub functions: BTreeMap<CFunctionRef, GeneratedCallableId>,
    pub values: BTreeMap<CValueBinding, GeneratedValueId>,
    pub reverse_types: BTreeMap<GeneratedTypeId, CStructRef>,
    pub reverse_functions: BTreeMap<GeneratedCallableId, CFunctionRef>,
    pub reverse_values: BTreeMap<GeneratedValueId, CValueBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CValueBinding {
    Member(CMemberRef),
    Parameter(CParameterRef),
    Local(CLocalRef),
}

impl CValueBinding {
    pub fn key(&self) -> &crate::ast::CDeclarationKey {
        match self {
            Self::Member(v) => v.key(),
            Self::Parameter(v) => v.key(),
            Self::Local(v) => v.key(),
        }
    }
    pub fn ty(&self) -> &CObjectType {
        match self {
            Self::Member(v) => v.ty(),
            Self::Parameter(v) => v.ty(),
            Self::Local(v) => v.ty(),
        }
    }
}

impl CBindings {
    pub fn symbols(&self) -> Vec<GeneratedSymbolId> {
        self.reverse_types
            .keys()
            .copied()
            .map(GeneratedSymbolId::Type)
            .chain(
                self.reverse_functions
                    .keys()
                    .copied()
                    .map(GeneratedSymbolId::Callable),
            )
            .chain(
                self.reverse_values
                    .keys()
                    .copied()
                    .map(GeneratedSymbolId::Value),
            )
            .collect()
    }
    pub fn ty(&self, ty: &CObjectType) -> TargetTypeRef<CDialect> {
        if ty.constness() == crate::ast::CConstness::Const {
            return TargetTypeRef::Constructed(ty.clone());
        }
        match ty.kind() {
            CObjectTypeKind::Scalar(CScalarType::I32) => TargetTypeRef::Known(CStdType::I32),
            CObjectTypeKind::Scalar(CScalarType::I64) => TargetTypeRef::Known(CStdType::I64),
            CObjectTypeKind::Scalar(value) => TargetTypeRef::Primitive(*value),
            CObjectTypeKind::Struct(value) => TargetTypeRef::Generated(self.types[value]),
            // The exact qualified pointer tree and registered pointee are kept,
            // not encoded in a string or replaced by an opaque erased pointer.
            _ => TargetTypeRef::Constructed(ty.clone()),
        }
    }
    pub fn signature(&self, function: &CFunctionRef) -> TargetCallableSignature<CDialect> {
        let CReturnType::Value(result) = function.signature().return_type() else {
            unreachable!("profile admits only scalar value returns");
        };
        TargetCallableSignature {
            invocation: CInvocation::Function,
            receiver: None,
            parameters: function
                .signature()
                .parameters()
                .iter()
                .map(|p| self.ty(p.declared_type()))
                .collect(),
            return_type: self.ty(result.declared_type()),
        }
    }
}
