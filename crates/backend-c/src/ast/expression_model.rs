//! Private contextual values and places retain their actual children/origins.

use super::{
    CBinaryOperator, CEnumeratorRef, CFunctionRef, CLiteral, CLocalRef, CMemberRef, CObjectRef,
    CObjectType, CParameterRef, CScalarType, CUnaryOperator, registry::RegistryScope,
};

/// A locally typed node, NOT a verified or render-ready certificate.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CObjectType, CValue};
/// fn forge(mut value: CValue, ty: CObjectType) { value.ty = ty; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CValue {
    pub(super) brand: RegistryScope,
    pub(super) ty: CObjectType,
    pub(super) kind: CValueKind,
}

impl CValue {
    pub const fn ty(&self) -> &CObjectType {
        &self.ty
    }
    pub const fn kind(&self) -> &CValueKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CValueKind {
    KnownConstant(super::CKnownConstant),
    Call(super::CCall),
    Literal(CLiteral),
    Read(Box<CPlace>),
    Enumerator(CEnumeratorRef),
    FunctionAddress(CFunctionRef),
    Unary {
        operator: CUnaryOperator,
        operand: Box<CValue>,
    },
    Binary {
        operator: CBinaryOperator,
        left: Box<CValue>,
        right: Box<CValue>,
    },
    PointerTest(CPointerTest),
    Conditional {
        condition: Box<CValue>,
        then_value: Box<CValue>,
        else_value: Box<CValue>,
    },
    Convert {
        conversion: CConversion,
        operand: Box<CValue>,
    },
    SizeOf(CObjectType),
    AlignOf(CObjectType),
    AddressOf(Box<CPlace>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CConversion {
    AllocationRestore(Box<super::CAllocationRef>),
    AdapterErase(Box<super::CInterfaceAdapterRef>),
    AdapterRestore(Box<super::CInterfaceAdapterRef>),
    Numeric(CScalarType),
    AddConst(CObjectType),
    ObjectToVoid(CObjectType),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CPointerTest {
    IsNull(Box<CValue>),
    IsNonNull(Box<CValue>),
    SameSlot {
        left: Box<CValue>,
        right: Box<CValue>,
    },
}

/// Places retain declaration owners even when their cached type is canonical.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CPlace, CPlaceKind};
/// fn retarget(mut place: CPlace, kind: CPlaceKind) { place.kind = kind; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CPlace {
    pub(super) brand: RegistryScope,
    pub(super) ty: CObjectType,
    pub(super) kind: CPlaceKind,
}

impl CPlace {
    pub const fn ty(&self) -> &CObjectType {
        &self.ty
    }
    pub const fn kind(&self) -> &CPlaceKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CPlaceKind {
    Local(CLocalRef),
    Parameter(CParameterRef),
    Global(CObjectRef),
    Member {
        base: Box<CPlace>,
        member: CMemberRef,
    },
    Dereference(Box<CValue>),
    Index {
        base: CIndexBase,
        index: Box<CValue>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CIndexBase {
    Array(Box<CPlace>),
    Pointer(Box<CValue>),
}
