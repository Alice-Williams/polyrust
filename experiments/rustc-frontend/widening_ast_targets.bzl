"""Canonical compiler input and typed widening-negation AST observations."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def widening_ast_targets(name, c_sources):
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
            name = "widening_" + language + "_ast_probe",
            crate_root = root,
            rustc_cfg = "widening_ast_probe",
            directory_publisher = ":directory_publisher" if language == "c" else None,
            srcs = sources + ["test/widening_" + language + "_ast.rs", "test/widening_input_probe.rs", "test/floating_source_ast.rs", "test/widening_" + language + "_dataflow.rs"],
            deps = [
                ":compiler_configuration",
                "//crates/backend-" + language + ":portable_backend_" + language,
                "//crates/binary64:portable_binary64",
                "//crates/codegen:portable_codegen",
                "//crates/diagnostics:portable_diagnostics",
            ] + ([":directory_publication"] if language == "c" else []),
        )
    inputs = [":widening_c_ast_probe", ":adapter", ":widening_java_ast_probe", ":java_adapter", "test/widening_ast_fixture.rs"]
    sh_test(
        name = name,
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/widening_ast.py)"] + ["$(rootpath " + item + ")" for item in inputs],
        data = ["test/widening_ast.py"] + inputs,
    )
