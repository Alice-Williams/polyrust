"""Actual compiler-to-Java graph admission; production check mode emits nothing."""
import os
from pathlib import Path
import subprocess
import sys

emitter, adapter, *mutations = [Path(value).resolve() for value in sys.argv[1:]]
work = Path(os.environ["TEST_TMPDIR"]) / "java-graph-check"
work.mkdir()
scratch = work / "scratch"
scratch.mkdir()
crates = {}


def add(key, text, dependencies=(), name=None):
    source = work / f"{key}.rs"
    source.write_text(text)
    item = dict(key=key, name=name or key, source=source,
                metadata=work / f"lib{key}.rmeta", dependencies=dependencies)
    crates[key] = item
    return item


def records(root, reverse=False):
    pending = [root["key"]]
    seen = {}
    while pending:
        key = pending.pop()
        if key in seen:
            continue
        seen[key] = crates[key]
        pending.extend(dependency for _, dependency in crates[key]["dependencies"])
    values = list(seen.values())
    if reverse:
        values.reverse()
    args = ["--root", root["key"]]
    for item in values:
        args.extend(["--crate", item["name"], item["key"], item["source"],
                     item["source"].name, item["metadata"]])
        for alias, dependency in item["dependencies"]:
            args.extend(["--dependency", alias, dependency])
    return args, len(seen)


def execute(root, metadata=False, reverse=False):
    args, count = records(root, reverse)
    before = {path: path.read_bytes() for path in work.glob("*.rmeta")}
    result = subprocess.run([str(value) for value in
        ([emitter] if metadata else [adapter, "--check-crates"]) + args],
        capture_output=True, text=True, timeout=60,
        env=dict(os.environ, TMPDIR=str(scratch)))
    assert not list(scratch.iterdir()), "compiler scratch leaked"
    assert not list(work.glob(".polyrust-metadata-*")), "metadata staging leaked"
    assert not list(work.rglob("*.java")), "check mode published Java"
    if not metadata:
        assert before == {path: path.read_bytes() for path in work.glob("*.rmeta")}
    return result, count


def emit(root):
    result, _ = execute(root, metadata=True)
    assert result.returncode == 0, result.stderr
    assert root["metadata"].is_file()


def accept(root):
    for reverse in [False, True]:
        result, count = execute(root, reverse=reverse)
        assert result.returncode == 0, result.stderr
        assert result.stdout == f"certified {count} Java crates; no output published\n"


def reject(root, reason):
    result, _ = execute(root)
    assert result.returncode != 0 and not result.stdout, result.stdout
    assert reason in result.stderr, result.stderr


leaf_text = """fn hidden(v: i32) -> i32 { v }
/// Leaf public identity.
pub fn identity(v: i32) -> i32 { hidden(v) }
pub fn zero() -> i32 { 0 }
pub fn invert(flag: bool) -> bool { if flag { false } else { true } }
pub fn choose(a: i32, b: bool, c: i32, d: bool) -> i32 {
    if b { if d { a } else { c } } else { c }
}
"""
leaf = add("leaf", leaf_text)
emit(leaf)
accept(leaf)
left = add("left", "pub fn identity(v: i32) -> i32 { renamed::identity(v) }",
           [("renamed", "leaf")])
emit(left)
accept(left)
right = add("right", "pub fn identity(v: i32) -> i32 { leaf::identity(v) }",
            [("leaf", "leaf")])
emit(right)
diamond = add("diamond",
              "pub fn identity(v: i32) -> i32 { left::identity(right::identity(v)) }",
              [("left", "left"), ("right", "right")])
accept(diamond)
aliases = add("aliases",
              "pub fn identity(v: i32) -> i32 { a::identity(b::identity(v)) }",
              [("a", "leaf"), ("b", "leaf")])
accept(aliases)
unused = add("unused", "pub fn identity(v: i32) -> i32 { v }", [("dep", "leaf")])
accept(unused)
arities = add("arities", """pub fn identity(v: i32) -> i32 {
    dep::choose(v, dep::invert(false), dep::zero(), true)
}""", [("dep", "leaf")])
accept(arities)
version = add("version", leaf_text, name="leaf")
emit(version)
versions = add("versions",
               "pub fn identity(v: i32) -> i32 { a::identity(b::identity(v)) }",
               [("a", "leaf"), ("b", "version")])
accept(versions)

# Both files remain valid metadata with the same crate name and API. Exact
# file-to-defining-key agreement must still reject a coordinated owner swap.
leaf["metadata"], version["metadata"] = version["metadata"], leaf["metadata"]
reject(versions, "source/metadata identity or content mismatch")
leaf["metadata"], version["metadata"] = version["metadata"], leaf["metadata"]
crates["changed_key"] = dict(leaf, key="changed_key")
changed_key = add("changed_key_consumer", "pub fn identity(v: i32) -> i32 { dep::identity(v) }",
                  [("dep", "changed_key")])
reject(changed_key, "source/metadata identity or content mismatch")

for binary, reason in zip(mutations, [
    "wrong compiler crate",
    "foreign compiler identity/signature differs from Java certificate",
    "foreign compiler identity/signature differs from Java certificate",
], strict=True):
    arguments, _ = records(versions)
    result = subprocess.run([str(value) for value in [binary, "--check-crates", *arguments]],
        capture_output=True, text=True, timeout=60,
        env=dict(os.environ, TMPDIR=str(scratch)))
    assert result.returncode != 0 and not result.stdout, result.stdout
    assert reason in result.stderr, result.stderr
    assert not list(scratch.iterdir()) and not list(work.rglob("*.java"))

private = add("private", "pub fn identity(v: i32) -> i32 { dep::hidden(v) }",
              [("dep", "leaf")])
reject(private, "private")
external = add("external", "pub fn identity(v: i32) -> i32 { let n = std::process::id(); v }")
reject(external, "no source-authenticated owning Java package")
for altered in [leaf_text.replace("Leaf public identity.", "Changed documentation."),
                leaf_text.replace("pub fn zero() -> i32 { 0 }", "pub fn zero() -> i32 { 1 }")]:
    leaf["source"].write_text(altered)
    reject(left, "source/metadata identity or content mismatch")
    reject(unused, "source/metadata identity or content mismatch")
    leaf["source"].write_text(leaf_text)
accept(diamond)
print("Java compiler graph admission, aliases, signatures and source agreement passed")
