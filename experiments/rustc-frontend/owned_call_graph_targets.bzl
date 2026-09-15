"""Closed local ownership graph proof, independent of target emitters."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def owned_call_graph_targets(name):
    """Declare compiler-backed graph proof.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs", "test/local_calls/mapping.rs", "test/owned_exit_consumer.rs"] + native.glob(["src/owned_source/*.rs", "src/owned_linear/**/*.rs", "test/owned_call_graph/*.rs"])
    compiler_adapter(name = "owned_call_graph_probe", srcs = sources, crate_root = "test/owned_call_graph/main.rs", rustc_cfg = "owned_call_graph_proof")
    for case, error in [("private_graph", "error[E0451]"), ("private_body", "error[E0451]"), ("erased", "error[E0308]"), ("raw", "error[E0061]"), ("assembly", "error[E0624]")]:
        compiler_adapter_compile_fail_test(name = "owned_call_graph_" + case + "_test", srcs = sources, crate_root = "test/owned_call_graph/main.rs", rustc_cfg = "call_graph_" + case, expected_error = error)
    adapter_format_test(name = "owned_call_graph_format_test", srcs = sources + ["fixtures/owned_call_graph.rs", "fixtures/owned_calls.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/owned_call_graph_test.py)", "$(rootpath :owned_call_graph_probe)", "$(rootpath fixtures/owned_call_graph.rs)", "$(rootpath fixtures/owned_calls.rs)"], data = ["test/owned_call_graph_test.py", ":owned_call_graph_probe", "fixtures/owned_call_graph.rs", "fixtures/owned_calls.rs"])
