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
pub(crate) mod contextual;
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
pub(crate) mod registry;
mod scalar_abi;
mod scalar_representation;
mod signatures;
mod statement_construction;
mod statement_errors;
mod statement_model;
mod statement_placement;
mod type_compatibility;
mod type_identity;
mod types;

#[cfg(test)]
#[path = "tests/contextual_reconstruction.rs"]
mod contextual_reconstruction;

#[cfg(test)]
#[path = "tests/owner_aggregates.rs"]
mod owner_aggregates;
#[cfg(test)]
#[path = "tests/owner_contracts.rs"]
mod owner_contracts;
#[cfg(test)]
#[path = "tests/owner_control.rs"]
mod owner_control;
#[cfg(test)]
#[path = "tests/owner_leaf_fixture.rs"]
mod owner_leaf_fixture;
#[cfg(test)]
#[path = "tests/owner_local.rs"]
mod owner_local;
#[cfg(test)]
#[path = "tests/owner_loops.rs"]
mod owner_loops;
#[cfg(test)]
#[path = "tests/owner_numbers.rs"]
mod owner_numbers;

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

#[cfg(test)]
#[path = "tests/contextual_completeness.rs"]
mod contextual_completeness;

#[cfg(test)]
#[path = "tests/contextual_conditional.rs"]
mod contextual_conditional;

#[cfg(test)]
#[path = "tests/contextual_scope_mutations.rs"]
mod contextual_scope_mutations;

#[cfg(test)]
#[path = "tests/contextual_origins.rs"]
mod contextual_origins;

#[cfg(test)]
#[path = "tests/contextual_control_mutations.rs"]
mod contextual_control_mutations;

#[cfg(test)]
#[path = "tests/contextual_aliases.rs"]
mod contextual_aliases;

#[cfg(test)]
#[path = "tests/contextual_value_variants.rs"]
mod contextual_value_variants;

#[cfg(test)]
#[path = "tests/contextual_initializer_variants.rs"]
mod contextual_initializer_variants;

#[cfg(test)]
#[path = "tests/contextual_callable_variants.rs"]
mod contextual_callable_variants;

#[cfg(test)]
#[path = "tests/contextual_declaration_variants.rs"]
mod contextual_declaration_variants;

#[cfg(test)]
#[path = "tests/contextual_union_joins.rs"]
mod contextual_union_joins;

#[cfg(test)]
#[path = "tests/contextual_owner_files.rs"]
mod contextual_owner_files;

#[cfg(test)]
#[path = "tests/contextual_definition_completeness.rs"]
mod contextual_definition_completeness;

#[cfg(test)]
#[path = "tests/contextual_definition_origins.rs"]
mod contextual_definition_origins;

#[cfg(test)]
#[path = "tests/contextual_safety_boundary.rs"]
mod contextual_safety_boundary;

#[cfg(test)]
#[path = "tests/contextual_control_edges.rs"]
mod contextual_control_edges;

#[cfg(test)]
#[path = "tests/contextual_return_paths.rs"]
mod contextual_return_paths;

#[cfg(test)]
#[path = "tests/contextual_address_operands.rs"]
mod contextual_address_operands;

#[cfg(test)]
#[path = "tests/known_call_fixtures.rs"]
mod known_call_fixtures;

#[cfg(test)]
#[path = "tests/known_calls.rs"]
mod known_calls;

#[cfg(test)]
#[path = "tests/contextual_known_calls.rs"]
mod contextual_known_calls;

#[cfg(test)]
#[path = "tests/constant_packages.rs"]
mod constant_packages;

#[cfg(test)]
#[path = "tests/sequencing_callables.rs"]
mod sequencing_callables;
#[cfg(test)]
#[path = "tests/sequencing_nested.rs"]
mod sequencing_nested;
#[cfg(test)]
#[path = "tests/sequencing_places.rs"]
mod sequencing_places;
#[cfg(test)]
#[path = "tests/sequencing_roots.rs"]
mod sequencing_roots;

#[cfg(test)]
#[path = "tests/counted_fixture.rs"]
mod counted_fixture;
#[cfg(test)]
#[path = "tests/counted_mutations.rs"]
mod counted_mutations;
#[cfg(test)]
#[path = "tests/counted_nested.rs"]
mod counted_nested;
#[cfg(test)]
#[path = "tests/counted_paths.rs"]
mod counted_paths;
#[cfg(test)]
#[path = "tests/counted_shapes.rs"]
mod counted_shapes;
#[cfg(test)]
#[path = "tests/numeric_exposure.rs"]
mod numeric_exposure;
#[cfg(test)]
#[path = "tests/numeric_fixture.rs"]
pub(crate) mod numeric_fixture;
#[cfg(test)]
#[path = "tests/numeric_floating.rs"]
mod numeric_floating;
#[cfg(test)]
#[path = "tests/numeric_generated_calls.rs"]
mod numeric_generated_calls;
#[cfg(test)]
#[path = "tests/numeric_guards.rs"]
mod numeric_guards;
#[cfg(test)]
#[path = "tests/numeric_lineage.rs"]
mod numeric_lineage;
#[cfg(test)]
#[path = "tests/numeric_loop_flow.rs"]
mod numeric_loop_flow;
#[cfg(test)]
#[path = "tests/numeric_loop_paths.rs"]
mod numeric_loop_paths;
#[cfg(test)]
#[path = "tests/numeric_relations.rs"]
mod numeric_relations;
#[cfg(test)]
#[path = "tests/numeric_sizes.rs"]
mod numeric_sizes;
#[cfg(test)]
#[path = "tests/numeric_storage.rs"]
mod numeric_storage;
#[cfg(test)]
#[path = "tests/numeric_switches.rs"]
mod numeric_switches;

#[cfg(test)]
#[path = "tests/index_extent_fixture.rs"]
mod index_extent_fixture;
#[cfg(test)]
#[path = "tests/index_extent_guards.rs"]
mod index_extent_guards;
#[cfg(test)]
#[path = "tests/index_extent_paths.rs"]
mod index_extent_paths;
#[cfg(test)]
#[path = "tests/index_extents.rs"]
mod index_extents;

#[cfg(test)]
#[path = "tests/constant_initializers.rs"]
mod constant_initializers;

pub use crate::ownership::CSafetyError;
#[cfg(test)]
#[path = "tests/storage_adapters.rs"]
mod storage_adapters;
#[cfg(test)]
#[path = "tests/storage_aggregates.rs"]
mod storage_aggregates;
#[cfg(test)]
#[path = "tests/storage_aliases.rs"]
mod storage_aliases;
#[cfg(test)]
#[path = "tests/storage_boundaries.rs"]
mod storage_boundaries;
#[cfg(test)]
#[path = "tests/storage_fixture.rs"]
mod storage_fixture;
#[cfg(test)]
#[path = "tests/storage_flow.rs"]
mod storage_flow;
#[cfg(test)]
#[path = "tests/storage_initialization.rs"]
mod storage_initialization;
#[cfg(test)]
#[path = "tests/storage_lifetimes.rs"]
mod storage_lifetimes;
#[cfg(test)]
#[path = "tests/storage_pointers.rs"]
mod storage_pointers;
#[cfg(test)]
#[path = "tests/storage_type_aliases.rs"]
mod storage_type_aliases;
pub use call_model::{CCall, CCallContract, CCallable, CCallableKind, CEffect};
#[cfg(test)]
#[path = "tests/allocation_aggregate_paths.rs"]
mod allocation_aggregate_paths;
#[cfg(test)]
#[path = "tests/allocation_exits.rs"]
mod allocation_exits;
#[cfg(test)]
#[path = "tests/allocation_fixture.rs"]
mod allocation_fixture;
#[cfg(test)]
#[path = "tests/allocation_flow.rs"]
mod allocation_flow;
#[cfg(test)]
#[path = "tests/allocation_identity.rs"]
mod allocation_identity;
#[cfg(test)]
#[path = "tests/allocation_loops.rs"]
mod allocation_loops;
#[cfg(test)]
#[path = "tests/buffer_aggregate_paths.rs"]
mod buffer_aggregate_paths;
#[cfg(test)]
#[path = "tests/buffer_constant_paths.rs"]
mod buffer_constant_paths;
#[cfg(test)]
#[path = "tests/buffer_count_bindings.rs"]
mod buffer_count_bindings;
#[cfg(test)]
#[path = "tests/buffer_counted_paths.rs"]
mod buffer_counted_paths;
#[cfg(test)]
#[path = "tests/buffer_fixture.rs"]
pub(crate) mod buffer_fixture;
#[cfg(test)]
#[path = "tests/buffer_order_guards.rs"]
mod buffer_order_guards;
#[cfg(test)]
#[path = "tests/buffer_prefix_aggregates.rs"]
mod buffer_prefix_aggregates;
#[cfg(test)]
#[path = "tests/buffer_prefix_boundaries.rs"]
mod buffer_prefix_boundaries;
#[cfg(test)]
#[path = "tests/buffer_prefix_cleanup.rs"]
mod buffer_prefix_cleanup;
#[cfg(test)]
#[path = "tests/buffer_prefix_control.rs"]
mod buffer_prefix_control;
#[cfg(test)]
#[path = "tests/buffer_prefix_fixture.rs"]
pub(crate) mod buffer_prefix_fixture;
#[cfg(test)]
#[path = "tests/buffer_prefix_lifetimes.rs"]
mod buffer_prefix_lifetimes;
#[cfg(test)]
#[path = "tests/buffer_prefix_numbers.rs"]
mod buffer_prefix_numbers;
#[cfg(test)]
#[path = "tests/buffer_prefix_partial.rs"]
mod buffer_prefix_partial;
#[cfg(test)]
#[path = "tests/buffer_prefix_pointer_steps.rs"]
mod buffer_prefix_pointer_steps;
#[cfg(test)]
#[path = "tests/buffer_prefix_reactivation.rs"]
mod buffer_prefix_reactivation;
#[cfg(test)]
#[path = "tests/buffer_range_copies.rs"]
mod buffer_range_copies;
#[cfg(test)]
#[path = "tests/buffer_range_joins.rs"]
mod buffer_range_joins;
#[cfg(test)]
#[path = "tests/buffer_reactivation.rs"]
mod buffer_reactivation;
#[cfg(test)]
#[path = "tests/buffer_scope_paths.rs"]
mod buffer_scope_paths;
#[cfg(test)]
#[path = "tests/buffer_symbolic_fixture.rs"]
mod buffer_symbolic_fixture;
#[cfg(test)]
#[path = "tests/buffer_symbolic_joins.rs"]
mod buffer_symbolic_joins;
#[cfg(test)]
#[path = "tests/buffer_symbolic_numbers.rs"]
mod buffer_symbolic_numbers;
#[cfg(test)]
#[path = "tests/buffer_symbolic_paths.rs"]
mod buffer_symbolic_paths;
#[cfg(test)]
#[path = "tests/heap_aggregates.rs"]
mod heap_aggregates;
#[cfg(test)]
#[path = "tests/heap_aliases.rs"]
mod heap_aliases;
#[cfg(test)]
#[path = "tests/heap_binding_flow.rs"]
mod heap_binding_flow;
#[cfg(test)]
#[path = "tests/heap_fixture.rs"]
mod heap_fixture;
#[cfg(test)]
#[path = "tests/heap_join_identity.rs"]
mod heap_join_identity;
#[cfg(test)]
#[path = "tests/heap_loops.rs"]
mod heap_loops;
#[cfg(test)]
#[path = "tests/heap_scalars.rs"]
mod heap_scalars;
#[cfg(test)]
#[path = "tests/heap_types.rs"]
mod heap_types;
#[cfg(test)]
#[path = "tests/numeric_memory_aggregates.rs"]
mod numeric_memory_aggregates;
#[cfg(test)]
#[path = "tests/numeric_memory_fixture.rs"]
mod numeric_memory_fixture;
#[cfg(test)]
#[path = "tests/numeric_memory_kernel.rs"]
mod numeric_memory_kernel;
#[cfg(test)]
#[path = "tests/numeric_memory_lifetimes.rs"]
mod numeric_memory_lifetimes;
#[cfg(test)]
#[path = "tests/numeric_memory_loops.rs"]
mod numeric_memory_loops;
#[cfg(test)]
#[path = "tests/numeric_memory_scalars.rs"]
mod numeric_memory_scalars;
#[cfg(test)]
#[path = "tests/storage_type_identity.rs"]
mod storage_type_identity;
pub use comments::{CAssertDiagnostic, CComment};
#[cfg(test)]
#[path = "tests/member_role_contracts.rs"]
mod member_role_contracts;
#[cfg(test)]
#[path = "tests/member_role_shapes.rs"]
mod member_role_shapes;
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
    CAggregateRef, CAllocationRef, CAllocationShape, CAllocatorSource, CBufferCountRef,
    CCallableContractOrigin, CCallableContractRef, CCleanupExitRef, CDeclarationKey, CEnumRef,
    CEnumeratorRef, CFileKey, CFileRef, CFileRole, CFrozenRegistry, CFunctionRef, CGeneratedOrigin,
    CInterfaceAdapterRef, CInterfaceTableRef, CInterfaceWitnessRef, CLocalRef, CLoopRef,
    CMemberBinding, CMemberOwnership, CMemberOwnershipRef, CMemberRef, CObjectRef, COwnerSlotRef,
    CParameterRef, CRegistrationKind, CRegistrationOwner, CRegistrationSummary, CRegistry,
    CRegistryError, CScopeRef, CStructRef, CSwitchRef, CSynthesisReason, CTypedefRef, CUnionRef,
    CWitnessMethod,
};
pub use scalar_representation::{CIntegerWidth, CScalarRepresentation};
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
