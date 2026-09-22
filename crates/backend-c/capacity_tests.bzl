"""Explicit execution partitions for the expensive checked-C native proofs."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")

CAPACITY_CASES = {
    "c_u32_constants_native_test": "dialect::shared::u32_constants::native::u32_constants_native_corpus_imports_faults_headers_and_ubsan",
    "c_characters_native_test": "dialect::shared::characters::native::characters_native_full_domain_comparisons_faults_and_headers",
    "c_infinite_constants_native_test": "dialect::shared::infinite_constants::native::infinity_constants_native_faults_headers_and_ubsan",
    "c_finite_constants_native_test": "dialect::shared::finite_constants::native::finite_constants_native_corpus_faults_headers_and_ubsan",
    "c_capacity_calls_test": "dialect::shared::call_native_tests::generated_call_paths_fit_native_frames_and_stack",
    "c_capacity_boundaries_test": "dialect::shared::capacity_policy_tests::actual_ast_boundaries_and_one_over_are_capacity_not_typing_failures",
    "c_capacity_storage_test": "dialect::shared::capacity_policy_tests::many_simultaneous_locals_and_aggregate_copies_have_nonzero_frame_cost",
    "c_capacity_dimensions_test": "dialect::shared::resource_tests::candidate_probes_have_the_intended_measured_dimensions",
    "c_capacity_native_test": "dialect::shared::spelling_tests::generated_scalar_units_compile_and_execute_with_strict_pinned_gcc",
}

def capacity_test_suite(name, unit):
    """Keep every Rust test enabled with independently cached expensive actions.

    Args:
        name: Complete suite retaining the historical public target name.
        unit: Compiled Rust test target, excluding exactly CAPACITY_CASES.
    """
    for target, case in CAPACITY_CASES.items():
        sh_test(
            name = target,
            size = "large",
            srcs = ["test/capacity_case.sh"],
            args = ["$(rootpath " + unit + ")", case],
            data = [unit, "//tools/c:zig_native_oracle"],
        )
    sh_test(
        name = "c_test_partition_contract_test",
        srcs = ["test/partition_contract.sh"],
        args = ["$(rootpath " + unit + ")"] + CAPACITY_CASES.values(),
        data = [unit],
    )
    native.test_suite(
        name = name,
        tests = [unit, ":c_test_partition_contract_test"] + [":" + target for target in CAPACITY_CASES],
        visibility = ["//visibility:public"],
    )
