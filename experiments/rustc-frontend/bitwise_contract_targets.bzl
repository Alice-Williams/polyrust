"""Compile-negative checks for the executable integer bitwise slot."""

load(":defs.bzl", "compiler_adapter_compile_fail_test")

def bitwise_contract_targets(name, c_sources):
    """Check registration and unforgeable input, separately for each backend.

    Args:
        name: Aggregate contract suite.
        c_sources: Complete C compiler adapter source inventory.
    """
    tests = []
    for language in ["c", "java"]:
        root = "src/main.rs" if language == "c" else "src/java_main.rs"
        sources = c_sources if language == "c" else [root, "src/inputs.rs", "src/source_admission.rs"] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ])
        for case, error in [
            ("missing", "error[E0599]"),
            ("duplicate", "error[E0599]"),
            ("wrong_capability", "error[E0271]"),
            ("wrong_context", "error[E0271]"),
            ("wrong_output", "error[E0271]"),
            ("wrong_input", "error[E0308]"),
            ("private_input", "error[E0451]"),
        ]:
            target = "bitwise_" + language + "_" + case + "_test"
            tests.append(":" + target)
            compiler_adapter_compile_fail_test(
                name = target,
                crate_root = root,
                srcs = sources + ["test/bitwise_contract.rs"],
                rustc_cfgs = ["bitwise_contract", "bitwise_" + language, "bitwise_" + case],
                expected_error = error,
                deps = [
                    ":compiler_configuration",
                    "//crates/backend-" + language + ":portable_backend_" + language,
                    "//crates/binary64:portable_binary64",
                    "//crates/codegen:portable_codegen",
                    "//crates/diagnostics:portable_diagnostics",
                ] + ([":directory_publication"] if language == "c" else []),
            )
    native.test_suite(name = name, tests = tests)
