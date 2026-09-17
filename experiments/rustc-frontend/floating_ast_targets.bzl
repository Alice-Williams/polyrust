"""Canonical compiler input and typed floating-negation AST observations."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def floating_ast_targets(name, c_sources):
    """Add read-only probes and production-byte comparison.

    Args:
        name: Executable observation test.
        c_sources: Complete C adapter source inventory.
    """
    for language in ["c", "java"]:
        root = "src/main.rs" if language == "c" else "src/java_main.rs"
        sources = c_sources if language == "c" else [root, "src/inputs.rs", "src/source_admission.rs"] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ])
        compiler_adapter(
            name = "floating_" + language + "_ast_probe",
            crate_root = root,
            rustc_cfg = "floating_ast_probe",
            directory_publisher = ":directory_publisher" if language == "c" else None,
            srcs = sources + ["test/floating_" + language + "_ast.rs", "test/floating_input_probe.rs", "test/floating_source_ast.rs", "test/floating_" + language + "_dataflow.rs"],
            deps = [
                ":compiler_configuration",
                "//crates/backend-" + language + ":portable_backend_" + language,
                "//crates/binary64:portable_binary64",
                "//crates/codegen:portable_codegen",
                "//crates/diagnostics:portable_diagnostics",
            ] + ([":directory_publication"] if language == "c" else []),
        )
    inputs = [":floating_c_ast_probe", ":adapter", ":floating_java_ast_probe", ":java_adapter", "test/floating_ast_fixture.rs"]
    sh_test(
        name = name,
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/floating_ast.py)"] + ["$(rootpath " + item + ")" for item in inputs],
        data = ["test/floating_ast.py"] + inputs,
    )
