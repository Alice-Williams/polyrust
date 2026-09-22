"""Infinity-value typed mapping observations and original-owner failure controls."""

load("@rules_rust//rust:defs.bzl", "rust_clippy_test", "rust_library", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")
load(":java_graph_targets.bzl", "java_graph_sources")

def infinite_constant_probe_targets(name, c_sources):
    """Add infinity constant probes.

    Args:
        name: Typed mapping and atomic publication proof target.
        c_sources: Exact production C adapter inventory.
    """
    for language in ["c", "java"]:
        cfgs = ["constant_ast_probe", "local_constant_ast_probe", "constant_import_probe"]
        extra = [
            "test/constant_" + language + "_ast.rs",
            "test/local_constant_" + language + "_ast.rs",
            "test/constant_import_" + language + "_ast.rs",
        ]
        if language == "c":
            cfgs.append("c_graph_inventory_contract")
            extra.extend(["test/c_import_manifest_contract.rs", "test/c_bundle_contract.rs"])
        else:
            cfgs.append("java_graph")
            extra.append("test/constant_import_java_graph.rs")
        compiler_adapter(
            name = "infinite_constant_" + language + "_probe",
            crate_root = "src/main.rs" if language == "c" else "src/java_main.rs",
            srcs = (c_sources if language == "c" else java_graph_sources()) + extra,
            rustc_cfgs = cfgs,
            directory_publisher = ":directory_publisher",
            deps = [
                ":compiler_configuration",
                ":directory_publication",
                "//crates/backend-" + language + ":portable_backend_" + language,
                "//crates/binary64:portable_binary64",
                "//crates/codegen:portable_codegen",
                "//crates/diagnostics:portable_diagnostics",
            ] + ([":java_bundle"] if language == "java" else []),
        )
    artifacts = [
        ":adapter",
        ":java_graph_adapter",
        ":infinite_constant_c_probe",
        ":infinite_constant_java_probe",
        ":public_constant_c_probe",
        ":public_constant_java_probe",
        ":generated_c_infinite_constant_root",
        ":generated_java_infinite_constant_root",
        "fixtures/infinite_constant_ast.rs",
    ] + ["fixtures/infinite_constant_" + name + ".rs" for name in ["data", "second", "middle", "root"]] + [
        ":infinite_constant_" + owner + "_metadata"
        for owner in ["constants", "second", "middle", "root"]
    ] + [
        ":constant_import_" + language + "_" + fault
        for language in ["c", "java"]
        for fault in ["wrong_declaration", "wrong_type", "wrong_value", "replaced_owner", "wrong_owner"]
    ]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/infinite_constant_source_ast.py)"] + ["$(rootpath " + target + ")" for target in artifacts],
        data = ["test/infinite_constant_source_ast.py"] + artifacts,
    )
    rust_library(
        name = "infinite_constant_ast_fixture",
        testonly = True,
        srcs = ["fixtures/infinite_constant_ast.rs"],
        edition = "2024",
    )
    rust_clippy_test(name = "infinite_constant_ast_fixture_clippy_test", targets = [":infinite_constant_ast_fixture"])
    rustfmt_test(name = "infinite_constant_ast_fixture_rustfmt_test", targets = [":infinite_constant_ast_fixture"])
