"""Actual compiler source-to-C-certificate integration, without publication."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":defs.bzl", "compiler_adapter")

def c_graph_targets(name, adapter_sources):
    """Declare separately owned C graph admission and rejection tests.

    Args:
        name: Compiler-to-certificate integration test target.
        adapter_sources: The production adapter's explicit compiler source inputs.
    """
    mutations = []
    rust_c_bundle(
        name = "generated_root_bundle",
        crate = ":generated_root_metadata",
    )
    rust_c_bundle(
        name = "generated_alias_bundle",
        crate = ":generated_alias_metadata",
    )
    sh_test(
        name = "c_bundle_action_test",
        srcs = ["test/metadata_action_test.sh"],
        args = ["$(rootpath test/c_bundle_action_test.py)", "$(rootpath :generated_root_bundle)", "$(rootpath :generated_alias_bundle)"],
        data = ["test/c_bundle_action_test.py", "test/c_bundle_assertions.py", ":generated_root_bundle", ":generated_alias_bundle"],
    )
    compiler_adapter(
        name = "c_graph_inventory_contract",
        srcs = adapter_sources + ["test/c_import_manifest_contract.rs", "test/c_bundle_contract.rs"],
        crate_root = "src/main.rs",
        rustc_cfg = "c_graph_inventory_contract",
        deps = [":compiler_configuration", ":directory_publication", "//crates/backend-c:portable_backend_c", "//crates/codegen:portable_codegen"],
    )
    for case in ["owner", "declaration", "signature"]:
        target = "c_graph_wrong_" + case
        compiler_adapter(
            name = target,
            srcs = adapter_sources + ["test/c_foreign_mutations.rs"],
            crate_root = "src/main.rs",
            rustc_cfg = target,
            deps = [":compiler_configuration", ":directory_publication", "//crates/backend-c:portable_backend_c", "//crates/codegen:portable_codegen"],
        )
        mutations.append(":" + target)
    sh_test(
        name = name,
        srcs = ["test/metadata_action_test.sh"],
        args = ["$(rootpath test/c_graph_test.py)", "$(rootpath :metadata_emitter)", "$(rootpath :adapter)", "$(rootpath :c_graph_inventory_contract)"] + ["$(rootpath " + target + ")" for target in mutations],
        data = ["test/c_graph_test.py", "test/c_bundle_assertions.py", "test/c_bundle_determinism.py", ":metadata_emitter", ":adapter", ":c_graph_inventory_contract"] + mutations,
    )
