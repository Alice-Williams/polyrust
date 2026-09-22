"""Independently cached native matrices, retaining the complete public suite."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")

NATIVE_CASES = {
    "java_characters_native_test": "tests::characters::native::characters_native_full_domain_and_compiling_faults",
    "java_finite_constants_native_test": "tests::finite_constants::native::finite_constants_native_corpus_and_inlining_sensitive_faults",
    "java_infinite_constants_native_test": "tests::infinite_constants::native::infinity_constants_native_aliases_and_inlining_sensitive_faults",
}

def native_test_suite(name, unit):
    """Split expensive native cases from the ordinary Rust test execution.

    Args:
        name: Complete suite retaining the historical public target name.
        unit: Compiled Rust test target excluding exactly NATIVE_CASES.
    """
    for target, case in NATIVE_CASES.items():
        sh_test(
            name = target,
            size = "large",
            srcs = ["test/native_case.sh"],
            args = ["run", "$(rootpath " + unit + ")", case],
            data = [unit, "@bazel_tools//tools/jdk:current_java_runtime"],
        )
    sh_test(
        name = "java_test_partition_contract_test",
        srcs = ["test/native_case.sh"],
        args = ["contract", "$(rootpath " + unit + ")"] + NATIVE_CASES.values(),
        data = [unit],
    )
    native.test_suite(
        name = name,
        tests = [unit, ":java_test_partition_contract_test"] + [":" + target for target in NATIVE_CASES],
        visibility = ["//visibility:public"],
    )
