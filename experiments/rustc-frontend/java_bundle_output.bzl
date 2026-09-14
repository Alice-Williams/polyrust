"""Production Java bundle action over the declared Rust source/metadata graph."""

load(":metadata_output.bzl", "RustSourceCrateInfo")
load(":metadata_records.bzl", "crate_graph_arguments")

def _bundle_impl(ctx):
    info = ctx.attr.crate[RustSourceCrateInfo]
    records = crate_graph_arguments(ctx, info.crate_key, info.crates)
    inputs = []
    for key, record in info.crates.items():
        inputs.append(record.source)
        inputs.extend([item.file for item in record.inputs])
        if key != info.crate_key:
            inputs.append(record.metadata)
    output = ctx.actions.declare_directory(ctx.label.name + ".bundle")
    ctx.actions.run_shell(
        arguments = [ctx.executable._adapter.path, output.path, records],
        command = """
set -eu
# Bazel precreates the declared empty tree. Remove only that empty directory.
test -d "$2"
rmdir "$2"
"$1" --bundle "$2" "$3"
""",
        tools = [ctx.attr._adapter[DefaultInfo].files_to_run],
        inputs = depset(inputs),
        outputs = [output],
        env = {"PATH": "/usr/bin:/bin"},
        mnemonic = "RustJavaBundle",
    )
    return [DefaultInfo(files = depset([output]), runfiles = ctx.runfiles(files = [output]))]

rust_java_bundle = rule(
    implementation = _bundle_impl,
    attrs = {
        "crate": attr.label(providers = [RustSourceCrateInfo], mandatory = True),
        "_adapter": attr.label(default = "//experiments/rustc-frontend:java_graph_adapter", executable = True, cfg = "exec"),
    },
)
