"""Actual checked Rust crates joined to separately certified C public APIs."""
import os
from pathlib import Path
import subprocess
import sys
import c_bundle_assertions
import c_bundle_determinism

emitter, adapter, inventory_contract, *mutations = [Path(value).resolve() for value in sys.argv[1:]]
work = Path(os.environ["TEST_TMPDIR"]) / "c-graph"
work.mkdir()
scratch = work / "scratch"
scratch.mkdir()
crates = {}
bundle_count = 0
expected_imports = {"leaf": 0, "middle": 2, "root": 2, "right": 1, "diamond": 2,
                    "aliases": 1, "arities": 2, "unused": 0, "versions": 2, "unary": 0, "integer_unary": 0}


def add(key, text, dependencies=(), name=None):
    source = work / (key + ".rs")
    source.write_text(text)
    item = dict(key=key, name=name or key, source=source,
                metadata=work / ("lib" + key + ".rmeta"), dependencies=dependencies)
    crates[key] = item
    return item


def records(root):
    pending = [root["key"]]
    seen = set()
    args = ["--root", root["key"]]
    while pending:
        key = pending.pop()
        if key in seen:
            continue
        seen.add(key)
        item = crates[key]
        args.extend(["--crate", item["name"], key, item["source"],
                     item["source"].name, item["metadata"]])
        for alias, dependency in item["dependencies"]:
            args.extend(["--dependency", alias, dependency])
            pending.append(dependency)
    return args, len(seen)


def execute(root, metadata=False):
    args, count = records(root)
    result = subprocess.run([str(value) for value in
        ([emitter] if metadata else [adapter, "--check-crates"]) + args],
        capture_output=True, text=True, timeout=60,
        env=dict(os.environ, TMPDIR=str(scratch)))
    assert not list(scratch.iterdir()), "C graph leaked compiler scratch files"
    assert not list(work.glob(".polyrust-metadata-*")), "metadata stage leaked"
    assert not list(work.glob("*.c")) and not list(work.glob("*.h")), "check mode published output"
    return result, count


def emit(root):
    result, _ = execute(root, metadata=True)
    assert result.returncode == 0, result.stderr
    assert root["metadata"].is_file()


def accept(root):
    global bundle_count
    before = {path: path.read_bytes() for path in work.glob("*.rmeta")}
    result, count = execute(root)
    assert result.returncode == 0, result.stderr
    assert result.stdout == f"certified {count} C crates; no output published\n"
    assert before == {path: path.read_bytes() for path in work.glob("*.rmeta")}
    arguments, _ = records(root)
    contract = subprocess.run([str(value) for value in [inventory_contract, "--check-crates", *arguments]],
        capture_output=True, text=True, timeout=60, env=dict(os.environ, TMPDIR=str(scratch)))
    assert contract.returncode == 0, contract.stderr
    assert contract.stdout == result.stdout and not list(scratch.iterdir())
    bundle_count += 1
    output = work / f"bundle-{bundle_count}"
    bundle = subprocess.run([str(value) for value in [adapter, "--bundle", output, *arguments]],
        capture_output=True, text=True, timeout=60, env=dict(os.environ, TMPDIR=str(scratch)))
    assert bundle.returncode == 0 and not bundle.stdout, bundle.stderr
    c_bundle_assertions.check(output, count, expected_imports[root["key"]])
    assert not list(scratch.iterdir()) and not list(work.glob(".polyrust-stage-*"))


def reject(root, diagnostic):
    before = {path: path.read_bytes() for path in work.glob("*.rmeta")}
    result, _ = execute(root)
    assert result.returncode != 0, result.stdout
    assert not result.stdout, "failed C graph reported a successful prefix"
    assert diagnostic in result.stderr, result.stderr
    assert before == {path: path.read_bytes() for path in work.glob("*.rmeta")}
    arguments, _ = records(root)
    for existing in [False, True]:
        output = work / ("rejected-" + root["key"] + str(existing))
        if existing:
            output.mkdir(exist_ok=True)
            (output / "sentinel").write_bytes(b"unchanged")
        result = subprocess.run([str(value) for value in [adapter, "--bundle", output, *arguments]],
            capture_output=True, text=True, timeout=60, env=dict(os.environ, TMPDIR=str(scratch)))
        assert result.returncode != 0 and not result.stdout and diagnostic in result.stderr, result.stderr
        if existing:
            assert {item.name: item.read_bytes() for item in output.iterdir()} == {"sentinel": b"unchanged"}
        else:
            assert not output.exists()
        assert not list(scratch.iterdir()) and not list(work.glob(".polyrust-stage-*"))


leaf_text = """fn hidden(v: i32) -> i32 { v }
/// Public scalar API with a private local helper.
pub fn identity(v: i32) -> i32 { hidden(v) }
pub fn echo(v: i32) -> i32 { v }
pub fn invert(v: bool) -> bool { if v { false } else { true } }
pub fn zero() -> i32 { 0 }
pub fn choose(flag: bool, left: i32, right: i32) -> i32 { if flag { left } else { right } }
"""
leaf = add("leaf", leaf_text)
emit(leaf)
accept(leaf)
middle = add("middle",
    "pub fn identity(v: i32) -> i32 { renamed::identity(v) }\n"
    "pub fn invert(v: bool) -> bool { renamed::invert(v) }\n",
    [("renamed", "leaf")])
emit(middle)
accept(middle)
root = add("root",
    "pub fn identity(v: i32) -> i32 { dep::identity(v) }\n"
    "pub fn invert(v: bool) -> bool { dep::invert(v) }\n",
    [("dep", "middle")])
accept(root)

right = add("right", "pub fn identity(v: i32) -> i32 { dep::identity(v) }\n",
            [("dep", "leaf")])
emit(right)
diamond = add("diamond",
    "pub fn identity(v: i32) -> i32 { left::identity(right::identity(v)) }\n",
    [("left", "middle"), ("right", "right")])
accept(diamond)
c_bundle_determinism.check(adapter, work, scratch, records(diamond)[0], 4, 2)

aliases = add("aliases",
    "pub fn identity(v: i32) -> i32 { left::identity(right::identity(v)) }\n",
    [("left", "leaf"), ("right", "leaf")])
accept(aliases)
arities = add("arities",
    "pub fn identity(v: i32) -> i32 { dep::choose(false, dep::zero(), v) }\n",
    [("dep", "leaf")])
accept(arities)
unused = add("unused", "pub fn identity(v: i32) -> i32 { v }\n", [("dep", "leaf")])
accept(unused)

version = add("version", leaf_text, name="leaf")
emit(version)
versions = add("versions",
    "pub fn identity(v: i32) -> i32 { left::identity(right::identity(v)) }\n",
    [("left", "leaf"), ("right", "version")])
accept(versions)
for binary, diagnostic in zip(mutations, [
    "wrong compiler crate",
    "foreign compiler identity/signature differs from C certificate",
    "foreign compiler identity/signature differs from C certificate",
], strict=True):
    arguments, _ = records(versions)
    result = subprocess.run([str(value) for value in [binary, "--check-crates", *arguments]],
        capture_output=True, text=True, timeout=60, env=dict(os.environ, TMPDIR=str(scratch)))
    assert result.returncode != 0 and diagnostic in result.stderr, result.stderr
    assert not result.stdout and not list(scratch.iterdir())

# Compiler privacy and missing target semantics remain closed.
private = add("private", "pub fn identity(v: i32) -> i32 { dep::hidden(v) }\n",
              [("dep", "leaf")])
reject(private, "is private")
external = add("external", "pub fn identity(_v: i32) -> i32 { std::process::id() as i32 }\n")
reject(external, "no source-authenticated owning C package")
diverging = add("diverging", "pub fn identity(_v: i32) -> i32 { std::process::abort() }\n")
reject(diverging, "direct-call compiler adjustments are not implemented")
reexport = add("reexport", "pub use dep::identity;\n", [("dep", "leaf")])
reject(reexport, "public package API mapping is not implemented")
generic = add("generic", "pub fn identity(v: i32) -> i32 { core::convert::identity(v) }\n")
reject(generic, "generic or mismatched direct callee identity")
unary = add("unary", "pub fn invert(v: bool) -> bool { !v }\n")
accept(unary)
integer_unary = add("integer_unary", "pub fn invert(v: i32) -> i32 { !v }\n")
accept(integer_unary)
arithmetic_unary = add("arithmetic_unary", "pub fn invert(v: i32) -> i32 { -v }\n")
reject(arithmetic_unary, "only negative integer literals")
conditional_argument = add("conditional_argument",
    "pub fn identity(v: i32) -> i32 { dep::identity(if v < 0 { 0 } else { v }) }\n",
    [("dep", "leaf")])
reject(conditional_argument, "C expression mapping is not implemented")
wide = add("wide", "pub fn identity(v: u64) -> u64 { v }\n")
emit(wide)
wide_unused = add("wide_unused", "pub fn identity(v: i32) -> i32 { v }\n",
                  [("dep", "wide")])
reject(wide_unused, "direct-call signatures support only i32, i64 and bool")

# A newly certified leaf cannot authenticate a consumer loading stale metadata.
leaf["source"].write_text("/// Changed docs.\n" + leaf_text)
reject(root, "source/metadata identity or content mismatch")
leaf["source"].write_text(leaf_text)
old_leaf, old_version = leaf["metadata"], version["metadata"]
leaf["metadata"], version["metadata"] = old_version, old_leaf
reject(versions, "source/metadata identity or content mismatch")
leaf["metadata"], version["metadata"] = old_leaf, old_version
accept(versions)
print("C graph: local/private helpers, bool/i32 foreign calls, transitive/diamond/repeated aliases, versions and closed rejection passed")
