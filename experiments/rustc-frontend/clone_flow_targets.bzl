"""Two independent owner chains around a standard shared Box clone."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def clone_flow_targets(name):
    """Declare clone correspondence tests.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs", "test/owned_exit_consumer.rs", "test/box_clone/mapping.rs"] + native.glob(["src/owned_source/*.rs", "src/owned_linear/**/*.rs", "test/clone_flow/**/*.rs"])
    compiler_adapter(name = "clone_flow_probe", srcs = sources, crate_root = "test/clone_flow/main.rs", rustc_cfgs = ["clone_flow_proof"])
    for case, error in [("private", "error[E0451]"), ("erased", "error[E0308]"), ("raw", "error[E0061]")]:
        compiler_adapter_compile_fail_test(name = "clone_flow_" + case + "_test", srcs = sources, crate_root = "test/clone_flow/main.rs", rustc_cfg = "clone_flow_" + case, expected_error = error)
    adapter_format_test(name = "clone_flow_format_test", srcs = sources + ["fixtures/clone_flow.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/clone_flow_test.py)", "$(rootpath :clone_flow_probe)", "$(rootpath fixtures/clone_flow.rs)"], data = ["test/clone_flow_test.py", ":clone_flow_probe", "fixtures/clone_flow.rs"])
