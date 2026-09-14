"""Explicit return evidence and cleanup correspondence."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_return_targets(name):
    """Declare the separate explicit-return proof.

    Args:
        name: Runtime assertion target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob([
        "src/owned_source/*.rs",
        "src/owned_linear/**/*.rs",
        "test/owned_returns/*.rs",
    ])
    compiler_adapter(name = "owned_return_probe", srcs = sources, crate_root = "test/owned_returns/main.rs", rustc_cfgs = ["owned_return_proof"])
    for case in ["body", "exit"]:
        compiler_adapter_compile_fail_test(
            name = "owned_return_private_" + case + "_test",
            srcs = sources,
            crate_root = "test/owned_returns/main.rs",
            rustc_cfg = "owned_return_private_" + case,
            expected_error = "error[E0451]",
            expected_errors = 1,
        )
    adapter_format_test(name = "owned_return_format_test", srcs = sources + ["fixtures/owned_returns.rs"])
    sh_test(
        name = name,
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/owned_return_test.py)", "$(rootpath :owned_return_probe)", "$(rootpath fixtures/owned_returns.rs)"],
        data = ["test/owned_return_test.py", ":owned_return_probe", "fixtures/owned_returns.rs"],
    )
