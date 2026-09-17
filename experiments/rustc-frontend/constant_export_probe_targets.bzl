"""Alias-specific certificate and atomic-publication controls."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def constant_export_probe_targets(name, adapter_sources):
    """Reuse real alias sources and original-owner mutation adapters.

    Args:
      name: Name of the atomic publication proof test.
      adapter_sources: Production compiler adapter Rust sources.
    """
    compiler_adapter(
        name = "constant_export_c_probe",
        crate_root = "src/main.rs",
        srcs = adapter_sources + ["test/constant_export_manifest.rs"],
        rustc_cfg = "constant_export_manifest_probe",
        directory_publisher = ":directory_publisher",
        deps = [
            ":compiler_configuration",
            ":directory_publication",
            "//crates/backend-c:portable_backend_c",
            "//crates/binary64:portable_binary64",
            "//crates/codegen:portable_codegen",
            "//crates/diagnostics:portable_diagnostics",
        ],
    )
    artifacts = [
        ":adapter",
        ":java_graph_adapter",
        ":constant_export_c_probe",
        ":metadata_emitter",
        ":generated_c_constant_export_root",
        ":generated_java_constant_export_root",
    ] + [
        ":constant_import_" + language + "_" + case
        for language in ["c", "java"]
        for case in ["wrong_declaration", "wrong_type", "wrong_value", "replaced_owner", "wrong_owner"]
    ] + [
        "fixtures/public_constant_data.rs",
        "fixtures/constant_export_second.rs",
        "fixtures/constant_export_middle.rs",
        "fixtures/constant_export_root.rs",
    ] + [":constant_export_" + owner + "_metadata" for owner in ["constants", "second", "middle", "root"]]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/constant_export_publication.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = ["test/constant_export_publication.py"] + artifacts,
    )
