"""Authenticated canonical instance fan-in without target source admission."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def instance_graph_targets(name):
    """Declare the graph probe, private-boundary checks and integration proof.

    Args:
        name: Behavioral graph test target.
    """
    sources = [
        "src/instance_graph_probe.rs",
        "test/instance_graph/interner.rs",
        "test/instance_graph/controls.rs",
        "test/instance_graph/audit.rs",
        "test/instance_graph/error_audit.rs",
        "test/instance_graph/error_corrupt.rs",
        "test/instance_graph/corrupt.rs",
        "test/scalar_result_identity/shape.rs",
        "test/scalar_result_identity/error_shape.rs",
        "src/compiler_dependencies.rs",
        "src/inputs.rs",
        "src/metadata_cli.rs",
        "src/metadata_dependencies.rs",
        "src/metadata_stage.rs",
        "src/source_check.rs",
        "src/source_check/agreement.rs",
        "src/source_check/dependencies.rs",
        "src/source_check/inventory.rs",
    ]
    deps = [":compiler_configuration", "//crates/codegen:portable_codegen"]
    for suffix, configs in [
        ("probe", ["instance_graph_probe"]),
        ("controls", ["instance_graph_probe", "instance_graph_controls"]),
        ("late_conflict", ["instance_graph_probe", "instance_graph_late_conflict"]),
        ("bad_facts", ["instance_graph_probe", "instance_graph_bad_facts"]),
        ("bad_error_facts", ["instance_graph_probe", "instance_graph_bad_error_facts"]),
        ("error_shape_fault", ["instance_graph_probe", "instance_graph_error_shape_fault"]),
        ("payload_free", ["instance_graph_probe", "instance_graph_payload_free"]),
        ("payload_free_unit", ["instance_graph_probe", "instance_graph_payload_free", "instance_graph_payload_free_unit"]),
    ]:
        compiler_adapter(
            name = "instance_graph_" + suffix,
            srcs = sources,
            crate_root = "src/instance_graph_probe.rs",
            rustc_cfgs = configs,
            deps = deps,
        )
    for suffix, error in [("forge", "error[E0624]"), ("frozen_forge", "error[E0616]")]:
        compiler_adapter_compile_fail_test(
            name = "instance_graph_" + suffix + "_test",
            srcs = sources,
            crate_root = "src/instance_graph_probe.rs",
            rustc_cfgs = ["instance_graph_probe", "instance_graph_" + suffix],
            expected_error = error,
            deps = deps,
        )
    adapter_format_test(name = "instance_graph_format_test", srcs = sources)
    sh_test(
        name = name,
        srcs = ["test/metadata_action_test.sh"],
        args = [
            "$(rootpath test/instance_graph_test.py)",
            "$(rootpath :metadata_emitter)",
            "$(rootpath :instance_graph_probe)",
            "$(rootpath :instance_graph_controls)",
            "$(rootpath :instance_graph_late_conflict)",
            "$(rootpath :instance_graph_bad_facts)",
            "$(rootpath :instance_graph_payload_free)",
            "$(rootpath :instance_graph_payload_free_unit)",
            "$(rootpath :instance_graph_bad_error_facts)",
            "$(rootpath :instance_graph_error_shape_fault)",
        ],
        data = [
            "test/instance_graph_test.py",
            ":metadata_emitter",
            ":instance_graph_probe",
            ":instance_graph_controls",
            ":instance_graph_late_conflict",
            ":instance_graph_bad_facts",
            ":instance_graph_payload_free",
            ":instance_graph_payload_free_unit",
            ":instance_graph_bad_error_facts",
            ":instance_graph_error_shape_fault",
        ],
    )
