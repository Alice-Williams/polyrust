"""Real compiler sessions reconcile complete original Result facts in a fan-in."""
import itertools
import os
from pathlib import Path
import subprocess
import sys


def main():
    (emitter, probe, controls, late_conflict, bad_facts, payload_free,
     payload_free_unit, bad_error_facts, error_shape_fault) = [str(Path(arg).resolve()) for arg in sys.argv[1:]]
    work = Path(os.environ["TEST_TMPDIR"]) / "instance-graph"
    work.mkdir()
    scratch = work / "scratch"
    scratch.mkdir()
    standard = "Result<i32, core::num::TryFromIntError>"

    def crate(name, source, dependencies=(), key=None):
        path = work / f"{name}.rs"
        path.write_text(source)
        return dict(name=name, path=path, key=key or name + ".key",
                    metadata=work / f"lib{name}.rmeta", dependencies=dependencies)

    def records(root, crates):
        args = ["--root", root["key"]]
        for item in crates:
            args.extend(["--crate", item["name"], item["key"], item["path"],
                         item["path"].name, item["metadata"]])
            for alias, key in item["dependencies"]:
                args.extend(["--dependency", alias, key])
        return args

    def run(binary, root, crates):
        return subprocess.run([str(arg) for arg in [binary, *records(root, crates)]],
                              capture_output=True, text=True, timeout=90,
                              env=dict(os.environ, TMPDIR=str(scratch)))

    def emit(item):
        # The publisher deliberately refuses overwrite. These are this test's
        # own scratch metadata files, recreated after a source/key mutation.
        assert item["metadata"].parent == work
        item["metadata"].unlink(missing_ok=True)
        result = run(emitter, item, [item])
        assert result.returncode == 0, result.stderr

    def snapshot():
        inventory = {}
        for path in work.rglob("*"):
            assert not path.is_symlink(), "unexpected observation output symlink"
            assert path.is_file() or path.is_dir(), "unexpected special output"
            inventory[str(path.relative_to(work))] = None if path.is_dir() else path.read_bytes()
        return inventory

    def accept(binary, root, crates, uses):
        before = snapshot()
        result = run(binary, root, crates)
        assert result.returncode == 0, result.stderr
        lines = result.stdout.splitlines()
        assert len(lines) == 2 and lines[0].startswith("INSTANCE "), result.stdout
        assert lines[1] == f"CHECKED {len(crates)} CRATES {uses} USES; no target output published"
        assert snapshot() == before, "observation changed a declared source/metadata input"
        assert not list(scratch.iterdir()), "observation leaked a scratch stage"
        return lines[0]

    def reject(binary, root, crates, error):
        before = snapshot()
        result = run(binary, root, crates)
        assert result.returncode != 0 and error in result.stderr, (error, result)
        assert not result.stdout, "failed graph exposed a successful prefix"
        assert snapshot() == before, "failed graph changed declared input"
        assert not list(scratch.iterdir()), "failed graph leaked a scratch stage"

    left = crate("left", f"pub fn identity(v: {standard}) -> {standard} {{ v }}")
    right = crate("right", f"""
        type Alias = core::result::Result<i32, <i32 as core::convert::TryFrom<i64>>::Error>;
        pub fn identity(v: Alias) -> Alias {{ match v {{ Ok(x) => Ok(x), Err(e) => Err(e) }} }}
    """)
    root_source = f"pub fn combine(v: {standard}) -> {standard} {{ left::identity(right::identity(v)) }}"
    root = crate("root", root_source, [("left", left["key"]), ("right", right["key"])])
    emit(left)
    emit(right)
    expected = accept(probe, root, [root, left, right], 6)
    assert accept(controls, root, [root, left, right], 6) == expected
    for order in itertools.permutations([root, left, right]):
        assert accept(probe, root, order, 6) == expected
    reject(late_conflict, root, [root, left, right], "instance graph original facts conflict")
    # Pinned Rust 1.98 has a one-byte error kind. Observation succeeds but the
    # former payload-free target assumption must fail. A unit-layout control
    # proves that rejection is conditional, not an always-failing guard.
    reject(payload_free, root, [root, left, right], "standard narrowing error must have zero-sized payload")
    assert accept(payload_free_unit, root, [root, left, right], 6) == expected
    for role in ["CoreRoot", "Result", "Error", "Ok", "Err", "OkPayload", "ErrPayload"]:
        os.environ["POLYRUST_INSTANCE_FAULT"] = role
        reject(bad_facts, root, [root, left, right], f"compiler instance role mismatch: {role}")
    del os.environ["POLYRUST_INSTANCE_FAULT"]
    for role in ["WrapperField", "Kind", "Empty", "InvalidDigit", "PosOverflow",
                 "NegOverflow", "Zero", "NotAPowerOfTwo"]:
        os.environ["POLYRUST_ERROR_STATE_FAULT"] = role
        reject(bad_error_facts, root, [root, left, right], f"compiler error-state role mismatch: {role}")
    del os.environ["POLYRUST_ERROR_STATE_FAULT"]
    for role, error in [
        ("WrapperFields", "standard error wrapper inventory changed"),
        ("VariantCount", "standard error kind inventory changed"),
        ("PayloadCount", "standard error kind variant mapping changed"),
        ("Discriminant", "standard error kind variant mapping changed"),
        ("Layout", "standard error state layout changed"),
    ]:
        os.environ["POLYRUST_ERROR_SHAPE_FAULT"] = role
        reject(error_shape_fault, root, [root, left, right], error)
    del os.environ["POLYRUST_ERROR_SHAPE_FAULT"]

    # Change actual deterministic dependency-first order, not just CLI order.
    reversed_left = dict(left, key="z.left.key")
    reversed_right = dict(right, key="a.right.key")
    reversed_root = dict(root, dependencies=[("left", reversed_left["key"]),
                                             ("right", reversed_right["key"])])
    emit(reversed_left)
    emit(reversed_right)
    assert accept(probe, reversed_root, [reversed_left, reversed_right, reversed_root], 6) == expected
    emit(left)
    emit(right)

    unused = crate("unrelated", "pub const NUMBER: i32 = 7;")
    emit(unused)
    expanded = dict(root, dependencies=[*root["dependencies"], ("unrelated", unused["key"])])
    assert accept(probe, expanded, [expanded, unused, right, left], 6) == expected

    original = right["path"].read_text()
    for source, error in [
        (f"pub fn identity(v: {standard.replace('i32', 'i64')}) -> {standard.replace('i32', 'i64')} {{ v }}", "result success payload"),
        ("pub enum Result<T, E> { Ok(T), Err(E) } "
         + f"pub fn identity(v: {standard}) -> {standard} {{ v }}", "expected standard scalar Result"),
        ("pub struct TryFromIntError; pub fn identity(v: Result<i32, TryFromIntError>) -> Result<i32, TryFromIntError> { v }", "result error payload"),
    ]:
        right["path"].write_text(source)
        emit(right)
        reject(probe, root, [root, left, right], error)
    right["path"].write_text(original)
    emit(right)
    assert accept(probe, root, [root, right, left], 6) == expected

    missing = dict(root, dependencies=[("left", left["key"]), ("right", "missing.key")])
    reject(probe, missing, [missing, left], "dependency is not declared")
    cyclic_left = dict(left, dependencies=[("root", root["key"])])
    reject(probe, root, [root, cyclic_left, right], "dependency cycle")

    # An actual later source failure also discards previously checked producers.
    root["path"].write_text(root_source.replace("left::identity", "left::absent"))
    reject(probe, root, [root, left, right], "cannot find function")
    root["path"].write_text(root_source)
    assert accept(probe, root, [root, left, right], 6) == expected
    print("Canonical instance: original fan-in authority, aliases, traversal permutations, "
          "unrelated owner, late conflicts, bounds, missing/cyclic graphs and restoration passed")


if __name__ == "__main__":
    main()
