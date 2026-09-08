"""Expose the selected pinned Rust compiler and its runtime to contract tests."""

def _rust_compiler_files_impl(ctx):
    toolchain = ctx.toolchains["@rules_rust//rust:toolchain_type"]
    return [DefaultInfo(files = toolchain.all_files, runfiles = ctx.runfiles(transitive_files = toolchain.all_files))]

rust_compiler_files = rule(
    implementation = _rust_compiler_files_impl,
    toolchains = ["@rules_rust//rust:toolchain_type"],
)
