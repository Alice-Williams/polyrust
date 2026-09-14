"""Independent tail-scope correspondence tests, retaining root-only coverage."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_scope_targets(name):
    """Declare scope proof and exact privacy contract.

    Args:
        name: Runtime proof target.
    """
    shared = ["src/inputs.rs", "src/source_capabilities/contracts.rs"] + native.glob([
        "src/owned_source/*.rs",
        "src/owned_linear/*.rs",
    ])
    sources = shared + native.glob(["test/owned_scopes/*.rs"])
    compiler_adapter(name = "owned_scope_probe", srcs = sources, crate_root = "test/owned_scopes/main.rs", rustc_cfgs = ["owned_scope_proof"])
    adapter_format_test(name = "owned_scope_format_test", srcs = sources + ["fixtures/owned_scopes.rs"])
    compiler_adapter_compile_fail_test(
        name = "owned_scope_private_test",
        srcs = shared + native.glob(["test/owned_linear/*.rs"]),
        crate_root = "test/owned_linear/main.rs",
        rustc_cfg = "owned_scope_private",
        expected_error = "error[E0451]",
        expected_errors = 1,
    )
    sh_test(
        name = name,
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/owned_scope_test.py)", "$(rootpath :owned_scope_probe)", "$(rootpath fixtures/owned_scopes.rs)"],
        data = ["test/owned_scope_test.py", ":owned_scope_probe", "fixtures/owned_scopes.rs"],
    )
