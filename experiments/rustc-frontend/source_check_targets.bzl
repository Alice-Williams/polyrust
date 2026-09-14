"""Independently cached checked-source driver and compiler authentication tests."""

load("@rules_rust//rust:defs.bzl", "rust_clippy_test", "rust_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")
load(":source_check_rule.bzl", "rust_source_check")

def source_check_targets(name):
    """Declare the source-only compiler checker and its integration gate.

    Args:
        name: Real source/metadata authentication test target.
    """
    sources = [
        "src/source_check_main.rs",
        "src/source_check.rs",
        "src/source_check/agreement.rs",
        "src/source_check/dependencies.rs",
        "src/source_check/inventory.rs",
        "src/compiler_dependencies.rs",
        "src/inputs.rs",
        "src/metadata_cli.rs",
        "src/metadata_dependencies.rs",
        "src/metadata_stage.rs",
    ]
    compiler_adapter(
        name = "source_checker",
        srcs = sources,
        crate_root = "src/source_check_main.rs",
        deps = [":compiler_configuration"],
    )
    compiler_adapter(
        name = "source_callback_contract",
        srcs = sources + ["test/source_callback.rs"],
        crate_root = "src/source_check_main.rs",
        rustc_cfg = "source_callback_contract",
        deps = [":compiler_configuration"],
    )
    rust_test(
        name = "source_inventory_test",
        srcs = ["test/source_inventory.rs", "src/source_check/inventory.rs"],
        crate_root = "test/source_inventory.rs",
        edition = "2024",
        deps = [":compiler_configuration"],
    )
    rust_clippy_test(
        name = "source_inventory_clippy_test",
        targets = [":source_inventory_test"],
    )
    rust_source_check(
        name = "checked_root_sources",
        crate = ":generated_root_metadata",
    )
    rust_source_check(
        name = "checked_alias_sources",
        crate = ":generated_alias_metadata",
    )
    sh_test(
        name = name,
        srcs = ["test/metadata_action_test.sh"],
        args = ["$(rootpath test/source_check_test.py)", "$(rootpath :metadata_emitter)", "$(rootpath :source_checker)", "$(rootpath :source_callback_contract)", "$(rootpath :checked_root_sources)", "$(rootpath :checked_alias_sources)"],
        data = ["test/source_check_test.py", ":metadata_emitter", ":source_checker", ":source_callback_contract", ":checked_root_sources", ":checked_alias_sources"],
    )
