"""Exact alias publication, tampered certificates, stale metadata and atomic failures."""
import json
import os
from pathlib import Path
import subprocess
import sys


def files(directory):
    return {path.relative_to(directory).as_posix(): path.read_bytes()
            for path in directory.rglob("*") if path.is_file()}


def main():
    args = [Path(value).resolve() for value in sys.argv[1:]]
    c, java, probe, emitter, cb, jb = args[:6]
    mutations = args[6:16]
    names = ["constants", "second", "middle", "root"]
    sources = dict(zip(names, args[16:20], strict=True))
    metadata = dict(zip(names, args[20:24], strict=True))
    dependencies = dict(constants=[], second=[], middle=["constants", "second"], root=["middle"])
    keys = {name: "proof.constant.export." + name for name in names}
    originals = {path: path.read_bytes() for path in [*sources.values(), *metadata.values()]}
    work = Path(os.environ["TEST_TMPDIR"]) / "constant-export-publication"
    work.mkdir()
    scratch = work / "scratch"
    scratch.mkdir()

    def records(root="root", order=None, paths=None, metas=None):
        paths, metas = paths or sources, metas or metadata
        used = set()

        def visit(name):
            if name in used:
                return
            used.add(name)
            for dep in dependencies[name]:
                visit(dep)

        visit(root)
        result = ["--root", keys[root]]
        for name in order or sorted(used):
            result += ["--crate", name, keys[name], paths[name], sources[name].name, metas[name]]
            for dep in dependencies[name]:
                result += ["--dependency", dep, keys[dep]]
        return result

    def run(adapter, tail, accepted=True):
        result = subprocess.run([str(value) for value in [adapter, *tail]], capture_output=True,
                                text=True, timeout=120, env=dict(os.environ, TMPDIR=str(scratch)))
        assert (result.returncode == 0) == accepted, (adapter, result.stdout, result.stderr)
        assert not list(scratch.iterdir())
        assert not list(work.glob(".polyrust-stage-*"))
        assert originals == {path: path.read_bytes() for path in originals}
        return result

    for label, adapter, generated in [("c", c, cb), ("java", java, jb)]:
        output = work / label
        run(adapter, ["--bundle", output, *records()])
        assert files(output) == files(generated)
        reordered = work / (label + "-reordered")
        run(adapter, ["--bundle", reordered, *records(order=list(reversed(names)))])
        assert files(reordered) == files(generated)
        alias = work / (label + "-alias-only")
        run(adapter, ["--bundle", alias, *records(root="middle")])
        manifests = [json.loads(path.read_text()) for path in alias.glob("*.api.json")]
        alias_manifest = next(value for value in manifests if value.get("constant_exports"))
        assert len(alias_manifest["constant_exports"]) == 12
        assert not alias_manifest.get("constant_imports")
        assert not alias_manifest.get("constants") and not alias_manifest.get("functions")
        if label == "java":
            assert not alias_manifest["declarations"]
        for existing in [False, True]:
            destination = work / f"{label}-missing-{existing}"
            if existing:
                destination.mkdir()
                (destination / "sentinel").write_bytes(b"preserved")
            result = run(adapter, ["--bundle", destination,
                                  *records(order=["second", "middle", "root"])], False)
            assert "dependency" in result.stderr
            assert destination.exists() == existing
            if existing:
                assert files(destination) == {"sentinel": b"preserved"}
    observed = work / "c-probe"
    result = run(probe, ["--bundle", observed, *records()])
    assert result.stdout.count("CONSTANT_EXPORT_MANIFEST\t7") == 2
    assert files(observed) == files(cb)

    for index, adapter in enumerate(mutations):
        for root in ["middle", "root"]:
            for existing in [False, True]:
                destination = work / f"mutant-{index}-{root}-{existing}"
                if existing:
                    destination.mkdir()
                    (destination / "sentinel").write_bytes(b"preserved")
                run(adapter, ["--bundle", destination, *records(root=root)], False)
                assert destination.exists() == existing
                if existing:
                    assert files(destination) == {"sentinel": b"preserved"}

    changed = work / "changed.rs"
    changed.write_text(sources["constants"].read_text().replace("7 * 9 - 1", "17"))
    assert changed.read_bytes() != sources["constants"].read_bytes()
    changed_sources = dict(sources, constants=changed)
    for label, adapter in [("c", c), ("java", java)]:
        destination = work / (label + "-stale")
        result = run(adapter, ["--bundle", destination, *records(paths=changed_sources)], False)
        assert "metadata" in result.stderr and not destination.exists()
    changed_metadata = dict(metadata)
    for name in names:
        changed_metadata[name] = work / ("libchanged_" + name + ".rmeta")
        run(emitter, records(root=name, paths=changed_sources, metas=changed_metadata))
        # The independent second producer legitimately remains unchanged.
        if name != "second":
            assert changed_metadata[name].read_bytes() != metadata[name].read_bytes()
    for label, adapter in [("c", c), ("java", java)]:
        destination = work / (label + "-fresh")
        run(adapter, ["--bundle", destination, *records(paths=changed_sources, metas=changed_metadata)])
        manifests = [json.loads(path.read_text()) for path in destination.glob("*.api.json")]
        aliases = [value for manifest in manifests for value in manifest.get("constant_exports", [])]
        assert any(value["value"] == "17" for value in aliases)
        assert all(value["value"] != "62" for value in aliases)
    print("Alias source/target inventory, seven C manifest mutations, ten authority mutants, missing/stale metadata and atomic failures")


if __name__ == "__main__":
    main()
