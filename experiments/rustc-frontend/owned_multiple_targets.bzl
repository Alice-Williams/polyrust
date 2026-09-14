"""Disjoint parameter-anchored owner-chain proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_multiple_targets(name):
    """Declare the separate multi-owner experiment.

    Args:
        name: Runtime assertion target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob([
        "src/owned_source/*.rs",
        "src/owned_linear/**/*.rs",
        "test/owned_multiple/*.rs",
    ])
    compiler_adapter(name = "owned_multiple_probe", srcs = sources, crate_root = "test/owned_multiple/main.rs", rustc_cfgs = ["owned_multiple_proof"])
    for case in ["body", "chain"]:
        compiler_adapter_compile_fail_test(
            name = "owned_multiple_private_" + case + "_test",
            srcs = sources,
            crate_root = "test/owned_multiple/main.rs",
            rustc_cfg = "owned_multiple_private_" + case,
            expected_error = "error[E0451]",
            expected_errors = 1,
        )
    adapter_format_test(name = "owned_multiple_format_test", srcs = sources + ["fixtures/owned_multiple.rs"])
    sh_test(
        name = name,
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/owned_multiple_test.py)", "$(rootpath :owned_multiple_probe)", "$(rootpath fixtures/owned_multiple.rs)"],
        data = ["test/owned_multiple_test.py", ":owned_multiple_probe", "fixtures/owned_multiple.rs"],
    )
