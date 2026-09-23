"""Checked compiler export inventory without enabling target publication."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter", "compiler_adapter_compile_fail_test")

def foreign_export_targets(name):
    """Declare source-only classification and private-construction tests.

    Args:
        name: Behavioral compiler inventory test target name.
    """
    sources = [
        "src/inputs.rs",
        "test/foreign_exports_main.rs",
        "test/source_constant_facts_support.rs",
        "src/source_capabilities/constant_values.rs",
        "src/source_capabilities/constant_evaluation.rs",
    ] + native.glob(["src/source_origin/**/*.rs"])
    compiler_adapter(
        name = "foreign_export_probe",
        srcs = sources,
        crate_root = "test/foreign_exports_main.rs",
        deps = ["//crates/codegen:portable_codegen", "//crates/binary64:portable_binary64"],
    )
    compiler_adapter(
        name = "foreign_export_mismatch_probe",
        srcs = sources + ["test/export_definition_mismatch.rs"],
        crate_root = "test/foreign_exports_main.rs",
        rustc_cfg = "export_definition_mismatch",
        deps = ["//crates/codegen:portable_codegen", "//crates/binary64:portable_binary64"],
    )
    compiler_adapter_compile_fail_test(
        name = "foreign_export_private_test",
        srcs = sources,
        crate_root = "test/foreign_exports_main.rs",
        rustc_cfg = "foreign_export_private",
        expected_error = "error[E0451]",
        deps = ["//crates/codegen:portable_codegen", "//crates/binary64:portable_binary64"],
    )
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = [
            "$(rootpath test/foreign_exports.py)",
            "$(rootpath :metadata_emitter)",
            "$(rootpath :foreign_export_probe)",
            "$(rootpath :foreign_export_mismatch_probe)",
        ],
        data = ["test/foreign_exports.py", ":metadata_emitter", ":foreign_export_probe", ":foreign_export_mismatch_probe"],
    )
