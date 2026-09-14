"""Compiler-only metadata emission and publication tests, independently cached."""

load("@rules_rust//rust:defs.bzl", "rust_clippy_test", "rust_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")
load(":metadata_output.bzl", "rust_source_metadata")

def metadata_action_targets(name):
    """Declare the metadata build action and its native/compiler contracts.

    Args:
        name: The metadata action integration test.
    """
    compiler_adapter(
        name = "metadata_emitter",
        srcs = ["src/metadata_main.rs", "src/metadata_output.rs", "src/metadata_stage.rs", "src/metadata_dependencies.rs", "src/metadata_cli.rs", "src/inputs.rs", "src/compiler_dependencies.rs"],
        crate_root = "src/metadata_main.rs",
        deps = [":compiler_configuration"],
    )
    rust_source_metadata(
        name = "generated_leaf_metadata",
        source = "fixtures/crate_identity.rs",
        crate_name = "metadata_fixture",
        crate_key = "metadata.action.v1",
    )
    rust_source_metadata(
        name = "generated_middle_metadata",
        source = "fixtures/metadata_middle.rs",
        crate_name = "metadata_middle",
        crate_key = "metadata.middle.v1",
        dependencies = {"renamed_leaf": ":generated_leaf_metadata"},
    )
    rust_source_metadata(
        name = "generated_root_metadata",
        source = "fixtures/metadata_root.rs",
        crate_name = "metadata_root",
        crate_key = "metadata.root.v1",
        dependencies = {"renamed_middle": ":generated_middle_metadata"},
    )
    rust_source_metadata(
        name = "generated_alias_metadata",
        source = "fixtures/metadata_aliases.rs",
        crate_name = "metadata_aliases",
        crate_key = "metadata.aliases.v1",
        dependencies = {"left": ":generated_leaf_metadata", "right": ":generated_leaf_metadata"},
    )
    sh_test(
        name = "metadata_closure_test",
        srcs = ["test/metadata_action_test.sh"],
        args = [
            "$(rootpath test/metadata_closure_test.py)",
            "$(rootpath :metadata_emitter)",
            "$(rootpath fixtures/crate_identity.rs)",
            "$(rootpath fixtures/metadata_middle.rs)",
            "$(rootpath fixtures/metadata_root.rs)",
            "$(rootpath :generated_leaf_metadata)",
            "$(rootpath :generated_middle_metadata)",
            "$(rootpath :generated_root_metadata)",
            "$(rootpath :generated_alias_metadata)",
        ],
        data = ["test/metadata_closure_test.py", ":metadata_emitter", "fixtures/crate_identity.rs", "fixtures/metadata_middle.rs", "fixtures/metadata_root.rs", ":generated_leaf_metadata", ":generated_middle_metadata", ":generated_root_metadata", ":generated_alias_metadata"],
    )
    rust_test(
        name = "metadata_output_test",
        srcs = ["test/metadata_output.rs", "src/metadata_output.rs", "src/metadata_stage.rs"],
        crate_root = "test/metadata_output.rs",
        edition = "2024",
    )
    rust_clippy_test(
        name = "metadata_output_clippy_test",
        targets = [":metadata_output_test"],
    )
    rust_test(
        name = "metadata_dependencies_test",
        srcs = ["test/metadata_dependencies.rs", "src/metadata_dependencies.rs"],
        crate_root = "test/metadata_dependencies.rs",
        edition = "2024",
        deps = [":compiler_configuration"],
    )
    rust_clippy_test(
        name = "metadata_dependencies_clippy_test",
        targets = [":metadata_dependencies_test"],
    )
    rust_test(
        name = "metadata_cli_test",
        srcs = ["test/metadata_cli.rs", "src/metadata_cli.rs"],
        crate_root = "test/metadata_cli.rs",
        edition = "2024",
        deps = [":compiler_configuration"],
    )
    rust_clippy_test(
        name = "metadata_cli_clippy_test",
        targets = [":metadata_cli_test"],
    )
    sh_test(
        name = name,
        srcs = ["test/metadata_action_test.sh"],
        args = ["$(rootpath test/metadata_action_test.py)", "$(rootpath :metadata_emitter)", "$(rootpath :metadata_probe)", "$(rootpath fixtures/crate_identity.rs)", "$(rootpath :generated_leaf_metadata)"],
        data = ["test/metadata_action_test.py", ":metadata_emitter", ":metadata_probe", "fixtures/crate_identity.rs", ":generated_leaf_metadata"],
    )
