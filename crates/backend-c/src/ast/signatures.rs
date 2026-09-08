//! Exact non-variadic C prototypes, with no arity-numbered API.

use super::types::{CConstness, CObjectType, CTypeError};

/// A prototype parameter after C's top-level qualifier normalization.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CParameterType(CObjectType);

impl CParameterType {
    pub fn new(ty: CObjectType) -> Result<Self, CTypeError> {
        if ty.is_array() {
            return Err(CTypeError::ArrayParameterRequiresPointer);
        }
        // A definition's local parameter constness is separate from its
        // prototype type. Pointee qualifiers remain part of this exact type.
        Ok(Self(ty.without_top_level_const()))
    }

    pub const fn ty(&self) -> &CObjectType {
        &self.0
    }
}

/// A non-array unqualified C return value.
///
/// An object/array cannot be inserted directly as a return witness:
///
/// ```compile_fail
/// use portable_backend_c::ast::{CArrayLength, CObjectType, CReturnType, CScalarType};
/// let array = CObjectType::array(CObjectType::scalar(CScalarType::I32),
///                                CArrayLength::new(3).unwrap());
/// let invalid = CReturnType::Value(array);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CReturnValue(CObjectType);

impl CReturnValue {
    pub fn new(ty: CObjectType) -> Result<Self, CTypeError> {
        if ty.is_array() {
            return Err(CTypeError::ArrayReturn);
        }
        if ty.constness() != CConstness::Unqualified {
            return Err(CTypeError::QualifiedReturn);
        }
        Ok(Self(ty))
    }

    pub const fn ty(&self) -> &CObjectType {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CReturnType {
    Void,
    Value(CReturnValue),
}

/// The empty parameter slice always denotes an explicit `(void)` prototype.
///
/// Parameters must pass their category-specific construction:
///
/// ```compile_fail
/// use portable_backend_c::ast::{CFunctionType, CObjectType, CReturnType, CScalarType};
/// let invalid = CFunctionType::new(CReturnType::Void,
///                                  vec![CObjectType::scalar(CScalarType::I32)]);
/// ```
///
/// Bare function types are not array element object types:
///
/// ```compile_fail
/// use portable_backend_c::ast::{CArrayLength, CFunctionType, CObjectType, CReturnType};
/// let function = CFunctionType::new(CReturnType::Void, vec![]);
/// let invalid = CObjectType::array(function, CArrayLength::new(1).unwrap());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CFunctionType {
    return_type: CReturnType,
    parameters: Vec<CParameterType>,
}

impl CFunctionType {
    pub const fn new(return_type: CReturnType, parameters: Vec<CParameterType>) -> Self {
        Self {
            return_type,
            parameters,
        }
    }

    pub const fn return_type(&self) -> &CReturnType {
        &self.return_type
    }

    pub fn parameters(&self) -> &[CParameterType] {
        &self.parameters
    }
}
