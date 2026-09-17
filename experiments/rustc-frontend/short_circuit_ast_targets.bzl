"""Read-only HIR-to-target branch and evaluation placement observations."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def short_circuit_ast_targets(name, c_sources):
    """Create two target-specific probe adapters and compare real output.

    Args:
        name: Probe/production equality test.
        c_sources: Complete C compiler adapter source inventory.
    """
    for language in ["c", "java"]:
        root = "src/main.rs" if language == "c" else "src/java_main.rs"
        sources = c_sources if language == "c" else [root, "src/inputs.rs", "src/source_admission.rs"] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ])
        compiler_adapter(
            name = "lazy_" + language + "_ast_probe",
            crate_root = root,
            rustc_cfg = "lazy_ast_probe",
            directory_publisher = ":directory_publisher" if language == "c" else None,
            srcs = sources + ["test/short_circuit_" + language + "_ast.rs", "test/short_circuit_hir_ast.rs"],
            deps = [
                ":compiler_configuration",
                "//crates/backend-" + language + ":portable_backend_" + language,
                "//crates/binary64:portable_binary64",
                "//crates/codegen:portable_codegen",
                "//crates/diagnostics:portable_diagnostics",
            ] + ([":directory_publication"] if language == "c" else []),
        )
    inputs = [":lazy_c_ast_probe", ":adapter", ":lazy_java_ast_probe", ":java_adapter", "fixtures/short_circuit.rs"]
    sh_test(
        name = name,
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/short_circuit_ast.py)"] + ["$(rootpath " + item + ")" for item in inputs],
        data = ["test/short_circuit_ast.py"] + inputs,
    )
