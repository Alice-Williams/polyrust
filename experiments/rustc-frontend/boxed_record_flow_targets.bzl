"""Boxed scalar-record producer/read/drop correspondence proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def boxed_record_flow_targets(name):
    """Declare the boxed payload correspondence and privacy contracts.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs", "test/owned_returns/compatibility.rs", "test/scalar_box/mapping.rs", "test/owned_record/mapping.rs"] + native.glob(["src/owned_source/*.rs", "src/owned_linear/**/*.rs", "test/boxed_record_flow/*.rs"])
    compiler_adapter(name = "boxed_record_flow_probe", srcs = sources, crate_root = "test/boxed_record_flow/main.rs", rustc_cfg = "boxed_record_flow_proof")
    for case, error in [("private_body", "error[E0451]"), ("private_field", "error[E0451]"), ("erased", "error[E0308]"), ("raw", "error[E0061]")]:
        compiler_adapter_compile_fail_test(name = "boxed_record_flow_" + case + "_test", srcs = sources, crate_root = "test/boxed_record_flow/main.rs", rustc_cfg = "boxed_flow_" + case, expected_error = error)
    adapter_format_test(name = "boxed_record_flow_format_test", srcs = sources + ["fixtures/boxed_record_flow.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/boxed_record_flow_test.py)", "$(rootpath :boxed_record_flow_probe)", "$(rootpath fixtures/boxed_record_flow.rs)"], data = ["test/boxed_record_flow_test.py", ":boxed_record_flow_probe", "fixtures/boxed_record_flow.rs"])
