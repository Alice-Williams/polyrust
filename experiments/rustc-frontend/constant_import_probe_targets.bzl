"""Compiler-reference probes and fail-closed constant-import mutations."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")
load(":java_graph_targets.bzl", "java_graph_sources")

def constant_import_probe_targets(name, adapter_sources):
    """Exercise original producer witnesses, graph closure and atomic outputs.

    Args:
        name: Atomic publication test target name.
        adapter_sources: Exact production C compiler adapter source inputs.
    """
    mutations = []
    for language in ["c", "java"]:
        root = "src/main.rs" if language == "c" else "src/java_main.rs"
        sources = adapter_sources if language == "c" else java_graph_sources()
        deps = [
            ":compiler_configuration",
            ":directory_publication",
            "//crates/backend-" + language + ":portable_backend_" + language,
            "//crates/codegen:portable_codegen",
            "//crates/diagnostics:portable_diagnostics",
        ] + ([":java_bundle"] if language == "java" else [])
        compiler_adapter(
            name = "constant_import_" + language + "_probe",
            crate_root = root,
            srcs = sources + ["test/constant_import_" + language + "_ast.rs"] +
                   (["test/c_import_manifest_contract.rs", "test/c_bundle_contract.rs"] if language == "c" else ["test/constant_import_java_graph.rs"]),
            rustc_cfgs = ["constant_import_probe", "c_graph_inventory_contract" if language == "c" else "java_graph"],
            directory_publisher = ":directory_publisher",
            deps = deps,
        )
        for case in ["wrong_declaration", "wrong_type", "wrong_value", "replaced_owner", "wrong_owner"]:
            target = "constant_import_" + language + "_" + case
            mutations.append(":" + target)
            compiler_adapter(
                name = target,
                crate_root = root,
                srcs = sources + (["test/constant_import_mutations.rs"] if case in ["wrong_type", "wrong_value"] else []),
                rustc_cfgs = ["constant_import_" + case] + (["java_graph"] if language == "java" else []),
                directory_publisher = ":directory_publisher",
                deps = deps,
            )
    artifacts = [
        ":adapter",
        ":java_graph_adapter",
        ":constant_import_c_probe",
        ":constant_import_java_probe",
        ":metadata_emitter",
        ":generated_c_constant_import",
        ":generated_java_constant_import",
    ] + mutations + [
        "fixtures/public_constant_data.rs",
        "fixtures/constant_import_bridge.rs",
        "fixtures/constant_import_root.rs",
    ] + [":constant_import_" + owner + "_metadata" for owner in ["constants", "left", "right", "root"]]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/constant_import_publication.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = ["test/constant_import_publication.py"] + artifacts,
    )
