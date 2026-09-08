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
mod identifiers;
mod keywords;
mod registry;
mod scalar_abi;
mod signatures;
mod types;

pub use identifiers::{CIdentifier, CNameError, CReservedMacro};
pub use keywords::CKeyword;
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
pub use types::{
    CArrayLength, CConstness, CObjectType, CObjectTypeKind, CPointerTarget, CScalarType, CTypeError,
};
