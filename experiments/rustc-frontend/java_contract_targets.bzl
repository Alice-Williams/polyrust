"""Each Java capability slot rejects absent, duplicate and wrong mappings."""

load(":defs.bzl", "compiler_adapter_compile_fail_test")

def java_contract_targets(name):
    """Declare six independent ten-slot compile-negative gates.

    Args:
        name: Aggregate test suite name.
    """
    tests = []
    for case, source, error in [
        ("missing", "missing", "error[E0599]"),
        ("duplicate", "duplicate", "error[E0599]"),
        ("wrong_capability", "signatures", "error[E0271]"),
        ("wrong_context", "signatures", "error[E0271]"),
        ("wrong_output", "signatures", "error[E0271]"),
        ("wrong_input", "wrong_input", "error[E0308]"),
    ]:
        compiler_adapter_compile_fail_test(
            name = "java_capability_" + case + "_test",
            crate_root = "src/java_main.rs",
            srcs = ["src/java_main.rs", "src/inputs.rs", "test/java_capability_" + source + ".rs"] + ["src/source_admission.rs"] + native.glob([
                "src/java_lower/**/*.rs",
                "src/source_capabilities/**/*.rs",
                "src/source_origin/**/*.rs",
            ]),
            rustc_cfg = "java_contract_" + case,
            expected_error = error,
            expected_errors = 10,
            deps = [
                ":compiler_configuration",
                "//crates/backend-java:portable_backend_java",
                "//crates/codegen:portable_codegen",
                "//crates/diagnostics:portable_diagnostics",
            ],
        )
        tests.append(":java_capability_" + case + "_test")
    native.test_suite(name = name, tests = tests)
