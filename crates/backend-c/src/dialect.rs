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
    CDefinedFunction, CDependencyApi, CDependencyFunction, CDependencyPackage, CDialect,
    CGeneratedHeader, CHeaderGuard, CImportKind, CImportedCallable, CImportedFunction,
    CStructuralRenderer, c_defined_functions, c_imported_functions, c_output_byte_bound,
    project_c_package,
};
