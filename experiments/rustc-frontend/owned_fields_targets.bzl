"""Record aggregate/partial-move correspondence proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_fields_targets(name):
    """Declare the partial-record proof.

    Args:
        name: Runtime proof target.
    """
    sources = ["test/owned_exit_consumer.rs", "src/inputs.rs", "src/source_capabilities/contracts.rs", "test/owned_record/mapping.rs", "test/owned_returns/compatibility.rs"] + native.glob([
        "src/owned_source/*.rs",
        "src/owned_linear/**/*.rs",
        "test/owned_fields/*.rs",
    ])
    compiler_adapter(name = "owned_fields_probe", srcs = sources, crate_root = "test/owned_fields/main.rs", rustc_cfg = "owned_fields_proof")
    adapter_format_test(name = "owned_fields_format_test", srcs = sources + ["fixtures/owned_fields.rs"])
    for case, error in [("private_body", "error[E0451]"), ("private_chain", "error[E0451]"), ("erased", "error[E0308]"), ("raw", "error[E0061]")]:
        compiler_adapter_compile_fail_test(name = "owned_fields_" + case + "_test", srcs = sources, crate_root = "test/owned_fields/main.rs", rustc_cfg = "fields_" + case, expected_error = error)
    sh_test(
        name = name,
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/owned_fields_test.py)", "$(rootpath :owned_fields_probe)", "$(rootpath fixtures/owned_fields.rs)"],
        data = ["test/owned_fields_test.py", ":owned_fields_probe", "fixtures/owned_fields.rs"],
    )
