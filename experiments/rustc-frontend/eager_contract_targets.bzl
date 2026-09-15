"""Compile-negative checks for the executable eager Boolean slot."""

load(":defs.bzl", "compiler_adapter_compile_fail_test")

def eager_contract_targets(name, c_sources):
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
            target = "eager_" + language + "_" + case + "_test"
            tests.append(":" + target)
            compiler_adapter_compile_fail_test(
                name = target,
                crate_root = root,
                srcs = sources + ["test/eager_contract.rs"],
                rustc_cfgs = ["eager_contract", "eager_" + language, "eager_" + case],
                expected_error = error,
                deps = [
                    ":compiler_configuration",
                    "//crates/backend-" + language + ":portable_backend_" + language,
                    "//crates/codegen:portable_codegen",
                    "//crates/diagnostics:portable_diagnostics",
                ] + ([":directory_publication"] if language == "c" else []),
            )
    native.test_suite(name = name, tests = tests)
