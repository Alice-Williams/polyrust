//! Closed C library identities. Shared linking is a separate migration stage.

mod catalogue;
mod dependencies;
mod shared;
pub(crate) use shared::CDependencyAuthority;
#[cfg(test)]
pub(crate) use shared::{package_fixture, package_source_fixture};

pub use catalogue::{CHeader, CKnownCall, CKnownCallForm, CKnownOperands, CSystemLibrary};
pub use dependencies::{CFileDependencies, CTagDependency, CTypeRequirement, file_dependencies};
pub use shared::{
    CDefinedConstant, CDefinedFunction, CDependencyApi, CDependencyConstant, CDependencyFunction,
    CDependencyPackage, CDependencyStruct, CDialect, CForeignConstantExport, CGeneratedHeader,
    CHeaderGuard, CImportKind, CImportedCallable, CImportedConstant, CImportedFunction,
    CImportedMember, CImportedValue, CPrimitiveType, CReferencedType, CStdType,
    CStructuralRenderer, c_defined_constants, c_defined_functions, c_defined_members,
    c_imported_constants, c_imported_functions, c_output_byte_bound, c_source_package,
    c_system_libraries, c_used_imported_constants, project_c_package,
};
