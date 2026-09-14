// Test-only registrations, included at AST scope to retain private fixture access.
#[path = "allocation_aggregate_paths.rs"]
mod allocation_aggregate_paths;
#[path = "allocation_exits.rs"]
mod allocation_exits;
#[path = "allocation_fixture.rs"]
mod allocation_fixture;
#[path = "allocation_flow.rs"]
mod allocation_flow;
#[path = "allocation_identity.rs"]
mod allocation_identity;
#[path = "allocation_loops.rs"]
mod allocation_loops;
#[path = "buffer_aggregate_paths.rs"]
mod buffer_aggregate_paths;
#[path = "buffer_constant_paths.rs"]
mod buffer_constant_paths;
#[path = "buffer_count_bindings.rs"]
mod buffer_count_bindings;
#[path = "buffer_counted_paths.rs"]
mod buffer_counted_paths;
#[path = "buffer_fixture.rs"]
pub(crate) mod buffer_fixture;
#[path = "buffer_order_guards.rs"]
mod buffer_order_guards;
#[path = "buffer_prefix_aggregates.rs"]
mod buffer_prefix_aggregates;
#[path = "buffer_prefix_boundaries.rs"]
mod buffer_prefix_boundaries;
#[path = "buffer_prefix_cleanup.rs"]
mod buffer_prefix_cleanup;
#[path = "buffer_prefix_control.rs"]
mod buffer_prefix_control;
#[path = "buffer_prefix_fixture.rs"]
pub(crate) mod buffer_prefix_fixture;
#[path = "buffer_prefix_lifetimes.rs"]
mod buffer_prefix_lifetimes;
#[path = "buffer_prefix_numbers.rs"]
mod buffer_prefix_numbers;
#[path = "buffer_prefix_partial.rs"]
mod buffer_prefix_partial;
#[path = "buffer_prefix_pointer_steps.rs"]
mod buffer_prefix_pointer_steps;
#[path = "buffer_prefix_reactivation.rs"]
mod buffer_prefix_reactivation;
#[path = "buffer_range_copies.rs"]
mod buffer_range_copies;
#[path = "buffer_range_joins.rs"]
mod buffer_range_joins;
#[path = "buffer_reactivation.rs"]
mod buffer_reactivation;
#[path = "buffer_scope_paths.rs"]
mod buffer_scope_paths;
#[path = "buffer_symbolic_fixture.rs"]
mod buffer_symbolic_fixture;
#[path = "buffer_symbolic_joins.rs"]
mod buffer_symbolic_joins;
#[path = "buffer_symbolic_numbers.rs"]
mod buffer_symbolic_numbers;
#[path = "buffer_symbolic_paths.rs"]
mod buffer_symbolic_paths;
#[path = "constant_initializers.rs"]
mod constant_initializers;
#[path = "constant_packages.rs"]
mod constant_packages;
#[path = "contextual_address_operands.rs"]
mod contextual_address_operands;
#[path = "contextual_aliases.rs"]
mod contextual_aliases;
#[path = "contextual_arrays.rs"]
mod contextual_arrays;
#[path = "contextual_callable_variants.rs"]
mod contextual_callable_variants;
#[path = "contextual_cases.rs"]
mod contextual_cases;
#[path = "contextual_completeness.rs"]
mod contextual_completeness;
#[path = "contextual_conditional.rs"]
mod contextual_conditional;
#[path = "contextual_control_edges.rs"]
mod contextual_control_edges;
#[path = "contextual_control_mutations.rs"]
mod contextual_control_mutations;
#[path = "contextual_declaration_variants.rs"]
mod contextual_declaration_variants;
#[path = "contextual_definition_completeness.rs"]
mod contextual_definition_completeness;
#[path = "contextual_definition_origins.rs"]
mod contextual_definition_origins;
#[path = "contextual_initialization.rs"]
mod contextual_initialization;
#[path = "contextual_initializer_variants.rs"]
mod contextual_initializer_variants;
#[path = "contextual_known_calls.rs"]
mod contextual_known_calls;
#[path = "contextual_lexical.rs"]
mod contextual_lexical;
#[path = "contextual_loops.rs"]
mod contextual_loops;
#[path = "contextual_origins.rs"]
mod contextual_origins;
#[path = "contextual_owner_files.rs"]
mod contextual_owner_files;
#[path = "contextual_package.rs"]
mod contextual_package;
#[path = "contextual_reconstruction.rs"]
mod contextual_reconstruction;
#[path = "contextual_return_paths.rs"]
mod contextual_return_paths;
#[path = "contextual_safety_boundary.rs"]
mod contextual_safety_boundary;
#[path = "contextual_scope_mutations.rs"]
mod contextual_scope_mutations;
#[path = "contextual_union_joins.rs"]
mod contextual_union_joins;
#[path = "contextual_value_variants.rs"]
mod contextual_value_variants;
#[path = "counted_fixture.rs"]
mod counted_fixture;
#[path = "counted_mutations.rs"]
mod counted_mutations;
#[path = "counted_nested.rs"]
mod counted_nested;
#[path = "counted_paths.rs"]
mod counted_paths;
#[path = "counted_shapes.rs"]
mod counted_shapes;
#[path = "heap_aggregates.rs"]
mod heap_aggregates;
#[path = "heap_aliases.rs"]
mod heap_aliases;
#[path = "heap_binding_flow.rs"]
mod heap_binding_flow;
#[path = "heap_fixture.rs"]
mod heap_fixture;
#[path = "heap_join_identity.rs"]
mod heap_join_identity;
#[path = "heap_loops.rs"]
mod heap_loops;
#[path = "heap_scalars.rs"]
mod heap_scalars;
#[path = "heap_types.rs"]
mod heap_types;
#[path = "index_extent_fixture.rs"]
mod index_extent_fixture;
#[path = "index_extent_guards.rs"]
mod index_extent_guards;
#[path = "index_extent_paths.rs"]
mod index_extent_paths;
#[path = "index_extents.rs"]
mod index_extents;
#[path = "known_call_fixtures.rs"]
mod known_call_fixtures;
#[path = "known_calls.rs"]
mod known_calls;
#[path = "member_role_contracts.rs"]
mod member_role_contracts;
#[path = "member_role_shapes.rs"]
mod member_role_shapes;
#[path = "numeric_exposure.rs"]
mod numeric_exposure;
#[path = "numeric_fixture.rs"]
pub(crate) mod numeric_fixture;
#[path = "numeric_floating.rs"]
mod numeric_floating;
#[path = "numeric_generated_calls.rs"]
mod numeric_generated_calls;
#[path = "numeric_guards.rs"]
mod numeric_guards;
#[path = "numeric_lineage.rs"]
mod numeric_lineage;
#[path = "numeric_loop_flow.rs"]
mod numeric_loop_flow;
#[path = "numeric_loop_paths.rs"]
mod numeric_loop_paths;
#[path = "numeric_memory_aggregates.rs"]
mod numeric_memory_aggregates;
#[path = "numeric_memory_fixture.rs"]
mod numeric_memory_fixture;
#[path = "numeric_memory_kernel.rs"]
mod numeric_memory_kernel;
#[path = "numeric_memory_lifetimes.rs"]
mod numeric_memory_lifetimes;
#[path = "numeric_memory_loops.rs"]
mod numeric_memory_loops;
#[path = "numeric_memory_scalars.rs"]
mod numeric_memory_scalars;
#[path = "numeric_relations.rs"]
mod numeric_relations;
#[path = "numeric_sizes.rs"]
mod numeric_sizes;
#[path = "numeric_storage.rs"]
mod numeric_storage;
#[path = "numeric_switches.rs"]
mod numeric_switches;
#[path = "owner_aggregates.rs"]
mod owner_aggregates;
#[path = "owner_contracts.rs"]
mod owner_contracts;
#[path = "owner_control.rs"]
mod owner_control;
#[path = "owner_leaf_fixture.rs"]
mod owner_leaf_fixture;
#[path = "owner_local.rs"]
mod owner_local;
#[path = "owner_loops.rs"]
mod owner_loops;
#[path = "owner_numbers.rs"]
mod owner_numbers;
#[path = "rust_source_origins.rs"]
mod rust_source_origins;
#[path = "sequencing_callables.rs"]
mod sequencing_callables;
#[path = "sequencing_nested.rs"]
mod sequencing_nested;
#[path = "sequencing_places.rs"]
mod sequencing_places;
#[path = "sequencing_roots.rs"]
mod sequencing_roots;
#[path = "storage_adapters.rs"]
mod storage_adapters;
#[path = "storage_aggregates.rs"]
mod storage_aggregates;
#[path = "storage_aliases.rs"]
mod storage_aliases;
#[path = "storage_boundaries.rs"]
mod storage_boundaries;
#[path = "storage_fixture.rs"]
mod storage_fixture;
#[path = "storage_flow.rs"]
mod storage_flow;
#[path = "storage_initialization.rs"]
mod storage_initialization;
#[path = "storage_lifetimes.rs"]
mod storage_lifetimes;
#[path = "storage_pointers.rs"]
mod storage_pointers;
#[path = "storage_type_aliases.rs"]
mod storage_type_aliases;
#[path = "storage_type_identity.rs"]
mod storage_type_identity;
