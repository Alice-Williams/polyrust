//! Checked projection of the existing C tree into the shared phase machinery.

mod bindings;
#[cfg(test)]
#[path = "../tests/shared_constant_export_fixture.rs"]
mod constant_export_fixture;
#[cfg(test)]
#[path = "../tests/shared_constant_export_native.rs"]
mod constant_export_native_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_export_projection.rs"]
mod constant_export_projection_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_export_rejections.rs"]
mod constant_export_rejection_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_export_resources.rs"]
mod constant_export_resource_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_exports.rs"]
mod constant_export_tests;
mod constant_exports;
mod constant_import_view;
mod constant_view;
mod definition_view;
mod dependency_api;
mod dependency_exports;
mod dependency_symbols;
mod documentation;
mod file_imports;
mod import_view;
pub use constant_import_view::{
    CImportedConstant, c_imported_constants, c_used_imported_constants,
};
mod imported_values;
mod linking;
mod nodes;
mod package;
mod platform;
mod platform_binary64;
mod profile;
mod projection;
mod registration;
mod resolved_names;
#[path = "../resources/hir.rs"]
mod resources;
mod source_package;
pub use source_package::c_source_package;
#[cfg(test)]
#[path = "../tests/shared_source_package.rs"]
mod source_package_tests;
#[path = "../render/hir.rs"]
mod spelling;
mod unit_bindings;
mod vocabulary;

#[cfg(test)]
#[path = "../tests/shared_constant_consumer_collisions.rs"]
mod constant_consumer_collision_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_consumer_composition.rs"]
mod constant_consumer_composition_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_consumer_fixture.rs"]
mod constant_consumer_fixture;
#[cfg(test)]
#[path = "../tests/shared_constant_consumer_native_chain.rs"]
mod constant_consumer_native_chain_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_consumer_native.rs"]
mod constant_consumer_native_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_consumer_projection.rs"]
mod constant_consumer_projection_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_consumer_registry.rs"]
mod constant_consumer_registry_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_consumers.rs"]
mod constant_consumer_tests;

#[cfg(test)]
#[path = "../tests/shared_constant_producer_dependencies.rs"]
mod constant_producer_dependency_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_producer_metadata.rs"]
mod constant_producer_metadata_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_producer_native.rs"]
mod constant_producer_native_tests;
#[cfg(test)]
#[path = "../tests/shared_constant_producers.rs"]
mod constant_producer_tests;
#[cfg(test)]
#[path = "../tests/shared_owned_constant_dependency_fixture.rs"]
mod owned_constant_dependency_fixture;
#[cfg(test)]
#[path = "../tests/shared_owned_constant_dependencies.rs"]
mod owned_constant_dependency_tests;
#[cfg(test)]
#[path = "../tests/shared_owned_constant_fixture.rs"]
mod owned_constant_fixture;
#[cfg(test)]
#[path = "../tests/shared_owned_constant_native.rs"]
mod owned_constant_native_tests;
#[cfg(test)]
#[path = "../tests/shared_owned_constant_projection.rs"]
mod owned_constant_projection_tests;
#[cfg(test)]
#[path = "../tests/shared_owned_constant_rejections.rs"]
mod owned_constant_rejection_tests;
#[cfg(test)]
#[path = "../tests/shared_owned_constants.rs"]
mod owned_constant_tests;

#[cfg(test)]
#[path = "../tests/shared_bitwise_admission.rs"]
mod bitwise_tests;

#[cfg(test)]
#[path = "../tests/shared_dependency_fixture.rs"]
mod dependency_fixture;
#[cfg(test)]
#[path = "../tests/shared_dependency_native.rs"]
mod dependency_native_tests;
#[cfg(test)]
#[path = "../tests/shared_dependency_resources.rs"]
mod dependency_resource_tests;

#[cfg(test)]
#[path = "../tests/shared_projection.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/shared_spelling.rs"]
mod spelling_tests;

#[cfg(test)]
#[path = "../tests/shared_record_fixture.rs"]
mod record_fixture;

#[cfg(test)]
#[path = "../tests/shared_resources.rs"]
mod resource_tests;

#[cfg(test)]
#[path = "../tests/shared_capacity_fixture.rs"]
mod capacity_fixture;

#[cfg(test)]
#[path = "../tests/shared_frame_reports.rs"]
mod frame_reports;

#[cfg(test)]
#[path = "../tests/shared_capacity_policy.rs"]
mod capacity_policy_tests;

#[cfg(test)]
#[path = "../tests/shared_profile_inventory.rs"]
mod profile_inventory_tests;

#[cfg(test)]
#[path = "../tests/shared_i64_admission.rs"]
mod i64_admission_tests;

#[cfg(test)]
#[path = "../tests/shared_i64_platform_pair.rs"]
mod i64_platform_pair_tests;

#[cfg(test)]
#[path = "../tests/shared_storage_fixture.rs"]
mod storage_fixture;

#[cfg(test)]
#[path = "../tests/shared_platform.rs"]
mod platform_tests;

#[cfg(test)]
#[path = "../tests/shared_call_fixture.rs"]
mod call_fixture;
#[cfg(test)]
#[path = "../tests/shared_call_mutations.rs"]
mod call_mutation_tests;
#[cfg(test)]
#[path = "../tests/shared_call_native_reports.rs"]
mod call_native_reports;
#[cfg(test)]
#[path = "../tests/shared_call_native.rs"]
mod call_native_tests;
#[cfg(test)]
#[path = "../tests/shared_call_resources.rs"]
mod call_resource_tests;
#[cfg(test)]
#[path = "../tests/shared_native_stack.rs"]
mod native_stack;
#[cfg(test)]
#[path = "../tests/shared_package_fixture.rs"]
pub(crate) mod package_fixture;
#[cfg(test)]
#[path = "../tests/shared_package_mutations.rs"]
mod package_mutation_tests;
#[cfg(test)]
#[path = "../tests/shared_package_native.rs"]
mod package_native_tests;
#[cfg(test)]
#[path = "../tests/shared_package_profile.rs"]
mod package_profile_tests;
#[cfg(test)]
#[path = "../tests/shared_package_projection.rs"]
mod package_projection_tests;
#[cfg(test)]
#[path = "../tests/shared_package_resources.rs"]
mod package_resource_tests;
#[cfg(test)]
#[path = "../tests/shared_package_source_fixture.rs"]
pub(crate) mod package_source_fixture;
#[cfg(test)]
#[path = "../tests/shared_platform_native.rs"]
mod platform_native_tests;

pub use constant_view::{CDefinedConstant, c_defined_constants};
pub use definition_view::{
    CDefinedFunction, c_defined_functions, c_defined_members, c_output_byte_bound,
};
pub(crate) use dependency_api::CDependencyAuthority;
pub use dependency_api::{
    CDependencyApi, CDependencyConstant, CDependencyFunction, CDependencyPackage,
    CForeignConstantExport, c_system_libraries,
};
pub use dependency_symbols::CImportedCallable;
pub use file_imports::{CGeneratedHeader, CHeaderGuard, CImportKind};
pub use import_view::{CImportedFunction, c_imported_functions};
pub use imported_values::CImportedValue;
pub use nodes::CResolvedUnit;
pub use package::CProjectedUnit;
pub use projection::project_c_package;
pub use spelling::CStructuralRenderer;
pub use vocabulary::{
    CDialect, CFileGrammar, CInvocation, CNamespace, CPrimitiveType, CSharedTypeKind, CStdType,
    CUnavailable, CVisibility,
};

use portable_codegen::{
    AstViolation, LinkedTargetPackage, TargetAstPackage, TargetDialect, verify_linked_package,
    verify_target_ast,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

fn violation(message: impl Into<String>) -> AstViolation {
    AstViolation::new(DiagnosticCode::InvalidStructure, message)
}

fn diagnostic(message: impl Into<String>) -> Vec<Diagnostic> {
    vec![Diagnostic::error(
        DiagnosticCode::InvalidStructure,
        message,
        SourceRef::logical(["c", "shared-projection"]),
    )]
}

impl TargetDialect for CDialect {
    type Unresolved = TargetAstPackage<Self>;
    type Resolved = LinkedTargetPackage<Self>;

    fn verify_unresolved(&self, ast: &Self::Unresolved) -> Result<(), Vec<Diagnostic>> {
        verify_target_ast(ast)
    }
    fn verify_resolved(&self, ast: &Self::Resolved) -> Result<(), Vec<Diagnostic>> {
        verify_linked_package(ast)
    }
    fn verify_resources(&self, package: &Self::Resolved) -> Result<(), Vec<Diagnostic>> {
        resources::resolved(package)
    }
}

#[cfg(test)]
#[path = "../tests/shared_constant_reference_view.rs"]
mod constant_reference_view_tests;

#[cfg(test)]
#[path = "../tests/shared_void_fixture.rs"]
mod void_fixture;
#[cfg(test)]
#[path = "../tests/shared_void_tests.rs"]
mod void_tests;

#[cfg(test)]
#[path = "../tests/shared_void_native.rs"]
mod void_native;

#[cfg(test)]
#[path = "../tests/shared_void_local.rs"]
mod void_local;

#[cfg(test)]
#[path = "../tests/shared_wrapping_negation.rs"]
mod wrapping_negation;

#[cfg(test)]
#[path = "../tests/shared_wrapping_addition.rs"]
mod wrapping_addition;

#[cfg(test)]
#[path = "../tests/shared_wrapping_integer.rs"]
mod wrapping_integer;

#[cfg(test)]
#[path = "../tests/shared_wrapping_subtraction.rs"]
mod wrapping_subtraction;

#[cfg(test)]
#[path = "../tests/shared_wrapping_multiplication.rs"]
mod wrapping_multiplication;

#[cfg(test)]
#[path = "../tests/shared_signed_widening.rs"]
mod signed_widening;

#[cfg(test)]
#[path = "../tests/shared_binary64.rs"]
mod binary64;

#[cfg(test)]
#[path = "../tests/shared_finite_constants.rs"]
mod finite_constants;

#[cfg(test)]
#[path = "../tests/shared_infinite_constants.rs"]
mod infinite_constants;

#[cfg(test)]
#[path = "../tests/shared_characters.rs"]
mod characters;

#[cfg(test)]
#[path = "../tests/shared_u32_constants.rs"]
mod u32_constants;
