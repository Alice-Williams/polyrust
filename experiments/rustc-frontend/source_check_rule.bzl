"""Declared whole-graph source checking, independent of metadata generation."""

load(":metadata_output.bzl", "RustSourceCrateInfo")
load(":metadata_records.bzl", "crate_graph_arguments")

def _check_impl(ctx):
    info = ctx.attr.crate[RustSourceCrateInfo]
    arguments = crate_graph_arguments(ctx, info.crate_key, info.crates)
    inputs = []
    for key, record in info.crates.items():
        inputs.append(record.source)
        inputs.extend([item.file for item in record.inputs])
        if key != info.crate_key:
            inputs.append(record.metadata)
    output = ctx.actions.declare_file(ctx.label.name + ".checked")
    ctx.actions.run_shell(
        arguments = [ctx.executable._checker.path, output.path, arguments],
        command = '"$1" "$3" > "$2"',
        tools = [ctx.attr._checker[DefaultInfo].files_to_run],
        inputs = depset(inputs),
        outputs = [output],
        env = {"PATH": "/usr/bin:/bin"},
        mnemonic = "CheckedRustSources",
    )

    # A descriptive build report only. No compiler proof is serialized or loaded
    # from this file; target lowering must run in the same checked invocation.
    return [DefaultInfo(files = depset([output]), runfiles = ctx.runfiles(files = [output]))]

rust_source_check = rule(
    implementation = _check_impl,
    attrs = {
        "crate": attr.label(providers = [RustSourceCrateInfo], mandatory = True),
        "_checker": attr.label(default = "//experiments/rustc-frontend:source_checker", executable = True, cfg = "exec"),
    },
)
