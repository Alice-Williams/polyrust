"""One Boolean guard, two complete ownership paths, no path-to-body erasure."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_guard_targets(name):
    """Declare guarded owner correspondence proof targets.

    Args:
        name: Runtime proof target.
    """
    sources = ["test/owned_exit_consumer.rs", "src/inputs.rs", "src/source_capabilities/contracts.rs", "test/owned_returns/compatibility.rs"] + native.glob([
        "src/owned_source/*.rs",
        "src/owned_linear/**/*.rs",
        "test/owned_guarded/*.rs",
    ])
    compiler_adapter(name = "owned_guard_probe", srcs = sources, crate_root = "test/owned_guarded/main.rs", rustc_cfgs = ["owned_guard_proof"])
    for case in ["body", "guard", "path"]:
        compiler_adapter_compile_fail_test(name = "owned_guard_private_" + case + "_test", srcs = sources, crate_root = "test/owned_guarded/main.rs", rustc_cfg = "owned_guard_private_" + case, expected_error = "error[E0451]", expected_errors = 1)
    compiler_adapter_compile_fail_test(name = "owned_guard_not_whole_test", srcs = sources, crate_root = "test/owned_guarded/main.rs", rustc_cfg = "owned_guard_not_whole", expected_error = "error[E0308]", expected_errors = 1)
    adapter_format_test(name = "owned_guard_format_test", srcs = sources + ["fixtures/owned_guarded.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/owned_guard_test.py)", "$(rootpath :owned_guard_probe)", "$(rootpath fixtures/owned_guarded.rs)"], data = ["test/owned_guard_test.py", ":owned_guard_probe", "fixtures/owned_guarded.rs"])
