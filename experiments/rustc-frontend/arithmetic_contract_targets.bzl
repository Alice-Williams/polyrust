"""Compile-negative executable arithmetic-value slot and checked-input boundaries."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter_compile_fail_test")

def arithmetic_contract_targets(name, c_sources):
    """Declare one intended failure per independently cached adapter action.

    Args:
        name: Aggregate contract suite.
        c_sources: Complete C adapter source inventory.
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
            target = "arithmetic_" + language + "_" + case + "_test"
            tests.append(":" + target)
            compiler_adapter_compile_fail_test(
                name = target,
                crate_root = root,
                srcs = sources + ["test/arithmetic_contract.rs"],
                rustc_cfgs = ["arithmetic_contract", "arithmetic_" + language, "arithmetic_" + case],
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

    inputs = [
        "test/capability_missing.rs",
        "src/c_lower/capabilities/slots.rs",
        "test/java_capability_missing.rs",
        "src/java_lower/capabilities/slots.rs",
    ]
    sh_test(
        name = "missing_slot_fixture_test",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/missing_slot_fixture_test.py)"] + ["$(rootpath " + item + ")" for item in inputs],
        data = ["test/missing_slot_fixture_test.py"] + inputs,
    )
