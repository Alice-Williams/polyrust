"""Compiler-authenticated constant facts and atomic import corruption controls."""
import os
from pathlib import Path
import subprocess
import sys


def files(root):
    return {p.relative_to(root).as_posix(): p.read_bytes() for p in root.rglob("*") if p.is_file()}


def main():
    args = [Path(p).resolve() for p in sys.argv[1:]]
    cp, jp, cb, jb = args[:4]
    sources = dict(zip(["constants", "second", "middle", "root"], args[4:8], strict=True))
    metadata = dict(zip(sources, args[8:12], strict=True))
    mutants = args[12:]
    assert len(mutants) == 10
    deps = dict(constants=[], second=[], middle=["constants", "second"], root=["middle"])
    keys = {owner: "proof.character.constant." + owner for owner in sources}
    records = ["--root", keys["root"]]
    for owner in sources:
        records += ["--crate", owner, keys[owner], sources[owner], sources[owner].name, metadata[owner]]
        for dep in deps[owner]:
            records += ["--dependency", dep, keys[dep]]
    originals = {p: p.read_bytes() for p in [*sources.values(), *metadata.values()]}
    work = Path(os.environ["TEST_TMPDIR"]) / "character-constant-source-ast"
    work.mkdir()
    scratch = work / "scratch"
    scratch.mkdir()

    def run(adapter, output, fault=None):
        env = dict(os.environ, TMPDIR=str(scratch))
        if fault is not None:
            env["POLYRUST_CHARACTER_CONSTANT_FAULT"] = fault
        result = subprocess.run([str(v) for v in [adapter, "--bundle", output, *records]],
                                capture_output=True, text=True, timeout=180, env=env)
        assert not list(scratch.iterdir()) and not list(work.glob(".polyrust-stage-*"))
        assert originals == {p: p.read_bytes() for p in originals}
        return result

    for language, probe, generated in [("c", cp, cb), ("java", jp, jb)]:
        output = work / (language + "-valid")
        result = run(probe, output)
        assert result.returncode == 0, result.stderr
        assert files(output) == files(generated)
        for fault in ["kind", "value", "declaration", "owner", "missing", "extra",
                      "target_kind", "target_value"]:
            for existing in [False, True]:
                output = work / f"{language}-{fault}-{existing}"
                if existing:
                    output.mkdir()
                    (output / "sentinel").write_bytes(b"preserved\x00\xff")
                before = files(output)
                result = run(probe, output, fault)
                if not fault.startswith("target_"):
                    expected = "source type facts differ from original compiler declarations"
                elif language == "java" and fault == "target_kind":
                    expected = "original Rust value differs from Java source constant facts"
                else:
                    expected = "source constant"
                assert result.returncode != 0 and expected in result.stderr, (fault, result.stderr)
                assert "panicked at" not in result.stderr and "error[E" not in result.stderr
                assert output.exists() == existing and files(output) == before
    for index, adapter in enumerate(mutants):
        for existing in [False, True]:
            output = work / f"import-{index}-{existing}"
            if existing:
                output.mkdir()
                (output / "sentinel").write_bytes(b"preserved\x00\xff")
            before = files(output)
            result = run(adapter, output)
            expected = (["exact member certificate", "conflict", "owner", "authorit"]
                        if index % 5 == 3 else ["identity/type/value differs"])
            if index < 5:
                expected.append("C declaration is already registered")
            assert result.returncode != 0 and any(x in result.stderr for x in expected), (index, result.stderr)
            assert "panicked at" not in result.stderr and "error[E" not in result.stderr
            assert output.exists() == existing and files(output) == before
    print("32 atomic original/target source-fact faults; 20 import certificate faults; "
          "same-valued Java Char/I32 forgery rejected; positive probes preserve exact output")


if __name__ == "__main__":
    main()
