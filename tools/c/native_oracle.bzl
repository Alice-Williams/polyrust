"""Declared pinned Zig SDK for runtime compiler-oracle tests, never generation."""

def _native_oracle_impl(ctx):
    sdk = ctx.toolchains["@zig_sdk//toolchain/zig:toolchain_type"].ziginfo
    compiler = sdk.zig.short_path
    if compiler.startswith("../"):
        compiler = compiler[3:]
    else:
        compiler = ctx.workspace_name + "/" + compiler
    launcher = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(
        launcher,
        """#!/usr/bin/env bash
set -euo pipefail
export ZIG_GLOBAL_CACHE_DIR="${TEST_TMPDIR:?}/zig-global-cache"
export ZIG_LOCAL_CACHE_DIR="${TEST_TMPDIR:?}/zig-local-cache"
exec "${TEST_SRCDIR:?}/%s" cc -target x86_64-linux-gnu "$@"
""" % compiler,
        is_executable = True,
    )
    return [DefaultInfo(
        executable = launcher,
        runfiles = ctx.runfiles(files = [sdk.zig] + sdk.data),
    )]

zig_native_oracle = rule(
    implementation = _native_oracle_impl,
    executable = True,
    toolchains = ["@zig_sdk//toolchain/zig:toolchain_type"],
)
