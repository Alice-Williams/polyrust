"""Declared directory artifact from the checked compiler package CLI."""

def _generate_impl(ctx):
    output = ctx.actions.declare_directory(ctx.label.name + ".package")
    arguments = [ctx.executable._adapter.path, ctx.file.source.path, output.path]
    if bool(ctx.attr.crate_name) != bool(ctx.attr.crate_key):
        fail("explicit package identity requires crate_name and crate_key together")
    if ctx.attr.crate_name:
        arguments.extend(["--crate-name", ctx.attr.crate_name, "--crate-key", ctx.attr.crate_key])
    for dependency in ctx.files.inputs:
        arguments.extend(["--input", dependency.path])
    ctx.actions.run_shell(
        command = """
set -eu
adapter="$1"
source="$2"
output="$3"
shift 3
# Bazel pre-creates tree artifacts; the CLI intentionally accepts only new paths.
stage=$(mktemp -d .polyrust-package.XXXXXXXX)
"$adapter" "$source" "$stage/package" --package "$@"
test -d "$output"
test -z "$(find "$output" -mindepth 1 -maxdepth 1 -print -quit)"
mv "$stage/package/"* "$output/"
rmdir "$stage/package" "$stage"
""",
        tools = [ctx.attr._adapter[DefaultInfo].files_to_run],
        inputs = [ctx.file.source] + ctx.files.inputs,
        outputs = [output],
        arguments = arguments,
        env = {"PATH": "/usr/bin:/bin"},
        mnemonic = "RustPublicPackage",
    )
    return [DefaultInfo(files = depset([output]), runfiles = ctx.runfiles(files = [output]))]

rust_public_package = rule(
    implementation = _generate_impl,
    attrs = {
        "source": attr.label(allow_single_file = [".rs"], mandatory = True),
        "inputs": attr.label_list(allow_files = True),
        "crate_name": attr.string(),
        "crate_key": attr.string(),
        "_adapter": attr.label(default = "//experiments/rustc-frontend:adapter", executable = True, cfg = "exec"),
    },
)
