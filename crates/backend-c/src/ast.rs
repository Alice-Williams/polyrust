//! C17 structural type foundation. These values are not source certificates.
//!
//! The complete dialect verifier, linker and certified renderer are separate
//! migration stages. No type constructor here advertises portable support.
//!
//! ```
//! use portable_backend_c::ast::{CFunctionType, CObjectType, CPointerTarget, CReturnType};
//! let callback = CFunctionType::new(CReturnType::Void, vec![]);
//! let pointer = CObjectType::pointer(CPointerTarget::Function(Box::new(callback)));
//! assert!(!pointer.is_array());
//! ```

mod alias_expansion;
mod call_construction;
mod call_model;
mod comments;
mod constant_expressions;
mod contextual;
mod control_construction;
mod declaration_construction;
mod declaration_model;
mod expression_construction;
mod expression_errors;
mod expression_model;
mod file_construction;
mod file_errors;
mod file_model;
mod identifiers;
mod initializer_construction;
mod initializer_model;
mod keywords;
mod known_constants;
mod known_objects;
mod literals;
mod operator_signatures;
mod ownership_conversions;
mod place_construction;
mod pointer_expressions;
mod registry;
mod scalar_abi;
mod signatures;
mod statement_construction;
mod statement_errors;
mod statement_model;
mod statement_placement;
mod type_compatibility;
mod types;

#[cfg(test)]
#[path = "tests/contextual_reconstruction.rs"]
mod contextual_reconstruction;

#[cfg(test)]
#[path = "tests/contextual_package.rs"]
mod contextual_package;

#[cfg(test)]
#[path = "tests/contextual_lexical.rs"]
mod contextual_lexical;

#[cfg(test)]
#[path = "tests/contextual_initialization.rs"]
mod contextual_initialization;

#[cfg(test)]
#[path = "tests/contextual_cases.rs"]
mod contextual_cases;

#[cfg(test)]
#[path = "tests/contextual_loops.rs"]
mod contextual_loops;

#[cfg(test)]
#[path = "tests/contextual_arrays.rs"]
mod contextual_arrays;

pub use call_model::{CCall, CCallable, CCallableKind, CEffect};
pub use comments::{CAssertDiagnostic, CComment};
pub use contextual::CContextError;
pub use declaration_construction::CDeclarations;
pub use declaration_model::{
    CDeclaration, CDeclarationKind, CDefinition, CDefinitionKind, CLinkage, CStorage,
};
pub use expression_construction::CExpressions;
pub use expression_errors::CExpressionError;
pub use expression_model::{
    CConversion, CIndexBase, CPlace, CPlaceKind, CPointerTest, CValue, CValueKind,
};
pub use file_errors::CFileError;
pub use file_model::{CFileItem, CSourceFile, CStaticAssertion};
pub use identifiers::{CIdentifier, CNameError, CReservedMacro};
pub use initializer_model::{CInitializer, CInitializerError, CInitializerKind};
pub use keywords::CKeyword;
pub use known_constants::CKnownConstant;
pub use known_objects::CKnownObject;
pub use literals::{CLiteral, CNullPointer, CSignedLiteral, CUnsignedLiteral};
pub use operator_signatures::{CBinaryOperator, COperatorError, CUnaryOperator};
pub use registry::{
    CAggregateRef, CAllocationRef, CAllocatorSource, CCallableContractOrigin, CCallableContractRef,
    CCleanupExitRef, CDeclarationKey, CEnumRef, CEnumeratorRef, CFileKey, CFileRef, CFileRole,
    CFrozenRegistry, CFunctionRef, CGeneratedOrigin, CInterfaceAdapterRef, CInterfaceTableRef,
    CInterfaceWitnessRef, CLocalRef, CLoopRef, CMemberBinding, CMemberRef, CObjectRef,
    CParameterRef, CRegistrationKind, CRegistrationOwner, CRegistrationSummary, CRegistry,
    CRegistryError, CScopeRef, CStructRef, CSwitchRef, CSynthesisReason, CTypedefRef, CUnionRef,
    CWitnessMethod,
};
pub use signatures::{CFunctionType, CParameterType, CReturnType, CReturnValue};
pub use statement_construction::CStatements;
pub use statement_errors::CStatementError;
pub use statement_model::{
    CBlock, CBreakTarget, CCaseConstant, CCountedProgress, CCountedStep, CLocalDeclaration,
    CStatement, CStatementKind, CSwitchArm,
};
pub use types::{
    CArrayLength, CConstness, CObjectType, CObjectTypeKind, CPointerTarget, CScalarType, CTypeError,
};
