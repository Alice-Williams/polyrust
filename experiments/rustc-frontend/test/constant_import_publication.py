"""Actual compiler/AST proofs, original-owner failures and atomic publication."""
import json
import os
from pathlib import Path
import subprocess
import sys


def files(root):
    return {p.relative_to(root).as_posix(): p.read_bytes()
            for p in root.rglob("*") if p.is_file()}


def main():
    args = [Path(p).resolve() for p in sys.argv[1:]]
    c, java, cp, jp, emitter, cb, jb = args[:7]
    mutations = args[7:17]
    data, bridge, source = args[17:20]
    metadata = dict(zip(["constants", "left", "right", "root"], args[20:], strict=True))
    sources = dict(constants=data, left=bridge, right=bridge, root=source)
    deps = dict(constants=[], left=["constants"], right=["constants"],
                root=["constants", "left", "right"])
    keys = {owner: "proof.constant.import." + owner for owner in metadata}
    originals = {p: p.read_bytes() for p in [data, bridge, source, *metadata.values()]}
    work = Path(os.environ["TEST_TMPDIR"]) / "constant-import-publication"
    work.mkdir()
    scratch = work / "scratch"
    scratch.mkdir()

    def records(root="root", order=None, paths=None, metas=None):
        paths, metas = paths or sources, metas or metadata
        used = set()

        def visit(owner):
            if owner in used:
                return
            used.add(owner)
            for dependency in deps[owner]:
                visit(dependency)

        visit(root)
        selected = order or sorted(used)
        result = ["--root", keys[root]]
        for owner in selected:
            result += ["--crate", owner, keys[owner], paths[owner],
                       sources[owner].name, metas[owner]]
            for dependency in deps[owner]:
                result += ["--dependency", dependency, keys[dependency]]
        return result

    def run(adapter, tail, success):
        result = subprocess.run([str(v) for v in [adapter, *tail]], capture_output=True,
                                text=True, timeout=120, env=dict(os.environ, TMPDIR=str(scratch)))
        assert (result.returncode == 0) == success, (adapter, result.stdout, result.stderr)
        assert not list(scratch.iterdir())
        assert not list(work.glob(".polyrust-stage-*"))
        assert originals == {p: p.read_bytes() for p in originals}
        return result

    for label, adapter, probe, generated in [("c", c, cp, cb), ("java", java, jp, jb)]:
        output = work / label
        production = run(adapter, ["--bundle", output, *records()], True)
        assert "CONSTANT_IMPORT_READ" not in production.stdout
        assert files(output) == files(generated)
        observed = work / (label + "-probe")
        checked = run(probe, ["--bundle", observed, *records()], True)
        assert checked.stdout.count("CONSTANT_IMPORT_READ\t") == 12
        if label == "java":
            assert "CONSTANT_IMPORT_GRAPH\tjava\t1" in checked.stdout
        assert files(observed) == files(output), "AST probes changed production bytes"
        reordered = work / (label + "-order")
        run(adapter, ["--bundle", reordered, *records(order=["root", "right", "left", "constants"])], True)
        assert files(reordered) == files(output)
        for kind in ["empty", "directory", "file", "symlink"]:
            destination = work / (label + "-" + kind)
            if kind in ["empty", "directory"]:
                destination.mkdir()
                if kind == "directory":
                    (destination / "sentinel").write_bytes(b"untouched\x00\xff")
            elif kind == "file":
                destination.write_bytes(b"untouched\x00\xff")
            else:
                destination.symlink_to(output, target_is_directory=True)
            before = destination.lstat()
            run(adapter, ["--bundle", destination, *records()], False)
            assert (destination.lstat().st_ino, destination.lstat().st_mode) == (before.st_ino, before.st_mode)
            if kind == "directory":
                assert files(destination) == {"sentinel": b"untouched\x00\xff"}
            elif kind == "empty":
                assert not list(destination.iterdir())
            elif kind == "file":
                assert destination.read_bytes() == b"untouched\x00\xff"
            else:
                assert destination.is_symlink()
            assert files(output) == files(generated)

    for index, adapter in enumerate(mutations):
        for existing in [False, True]:
            output = work / f"mutation-{index}-{existing}"
            if existing:
                output.mkdir()
                (output / "sentinel").write_bytes(b"preserved")
            result = run(adapter, ["--bundle", output,
                         *records(root="left" if index % 5 == 3 else "root")], False)
            if index % 5 != 3:
                assert "identity/type/value differs" in result.stderr, result.stderr
            else:
                assert any(text in result.stderr for text in [
                    "exact member certificate", "conflict", "owner", "authorit"
                ]), result.stderr
            assert output.exists() == existing
            if existing:
                assert files(output) == {"sentinel": b"preserved"}

    altered = work / "altered.rs"
    altered.write_text(data.read_text().replace("7 * 9 - 1", "17"))
    assert altered.read_bytes() != data.read_bytes()
    changed_sources = dict(sources, constants=altered)
    for label, adapter in [("c", c), ("java", java)]:
        for mode in ["missing", "stale"]:
            tail = records(order=["left", "right", "root"]) if mode == "missing" else records(paths=changed_sources)
            for existing in [False, True]:
                output = work / f"{label}-{mode}-{existing}"
                if existing:
                    output.mkdir()
                    (output / "sentinel").write_bytes(b"preserved")
                result = run(adapter, ["--bundle", output, *tail], False)
                assert ("dependency" if mode == "missing" else "metadata") in result.stderr, result.stderr
                assert output.exists() == existing
                if existing:
                    assert files(output) == {"sentinel": b"preserved"}

    # Regenerated metadata must propagate producer changes through every consumer.
    changed_metadata = dict(metadata)
    for owner in ["constants", "left", "right", "root"]:
        changed_metadata[owner] = work / ("libchanged_" + owner + ".rmeta")
        run(emitter, records(root=owner, paths=changed_sources, metas=changed_metadata), True)
        assert changed_metadata[owner].read_bytes() != metadata[owner].read_bytes()
    for label, adapter in [("c", c), ("java", java)]:
        changed = work / (label + "-fresh-metadata")
        run(adapter, ["--bundle", changed, *records(paths=changed_sources, metas=changed_metadata)], True)
        original = files(cb if label == "c" else jb)
        assert files(changed) != original
        manifests = [json.loads(p.read_text()) for p in changed.rglob("*.json") if p.name != "bundle.json"]
        imported = [entry for m in manifests for entry in m.get("constant_imports", [])]
        assert any(entry["value"] == "17" for entry in imported)
        assert all(entry["value"] != "62" for entry in imported)
    print("Compiler/AST imports, missing/stale/replaced owners, exact bindings, mutation propagation and atomic publication")


if __name__ == "__main__":
    main()
