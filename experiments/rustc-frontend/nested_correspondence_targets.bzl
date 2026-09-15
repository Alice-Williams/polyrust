"""Complete nested source/MIR correspondence and query-only API contracts."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def nested_correspondence_targets(name):
    """Declare the nested body proof.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs", "test/owned_exit_consumer.rs", "test/owned_record/mapping.rs", "test/nested_record/mapping.rs", "test/nested_owned_probe/trace.rs"] + native.glob(["src/owned_source/*.rs", "src/owned_linear/**/*.rs", "test/nested_correspondence/*.rs"])
    compiler_adapter(name = "nested_correspondence_probe", srcs = sources, crate_root = "test/nested_correspondence/main.rs", rustc_cfg = "owned_nested_proof")
    adapter_format_test(name = "nested_correspondence_format_test", srcs = sources + ["fixtures/nested_correspondence.rs"])
    for case, error in [("private", "error[E0451]"), ("private_path", "error[E0451]"), ("private_event", "error[E0451]"), ("erased", "error[E0308]"), ("raw", "error[E0061]")]:
        compiler_adapter_compile_fail_test(name = "nested_correspondence_" + case + "_test", srcs = sources, crate_root = "test/nested_correspondence/main.rs", rustc_cfg = "nested_body_" + case, expected_error = error)
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/nested_correspondence_test.py)", "$(rootpath :nested_correspondence_probe)", "$(rootpath fixtures/nested_correspondence.rs)"], data = ["test/nested_correspondence_test.py", ":nested_correspondence_probe", "fixtures/nested_correspondence.rs"])
