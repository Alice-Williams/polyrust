"""Real source facts and atomic publication under type/identity corruption."""
import os
from pathlib import Path
import subprocess
import sys


def files(root):
    return {p.relative_to(root).as_posix(): p.read_bytes() for p in root.rglob("*") if p.is_file()}


def main():
    args = [Path(p).resolve() for p in sys.argv[1:]]
    c, java, cp, jp, cb, jb = args[:6]
    sources = dict(zip(["leaf", "middle", "root"], args[6:9], strict=True))
    metadata = dict(zip(sources, args[9:12], strict=True))
    assert len(args) == 12
    keys = {owner: "proof.character_" + owner + ".v1" for owner in sources}
    records = ["--root", keys["root"]]
    for owner in sources:
        records += ["--crate", "character_" + owner, keys[owner], sources[owner],
                    sources[owner].name, metadata[owner]]
        if owner != "leaf":
            dep = "leaf" if owner == "middle" else "middle"
            records += ["--dependency", "character_" + dep, keys[dep]]
    original = {p: p.read_bytes() for p in [*sources.values(), *metadata.values()]}
    work = Path(os.environ["TEST_TMPDIR"]) / "character-source-ast"
    work.mkdir()
    scratch = work / "scratch"
    scratch.mkdir()
    for language, production, probe, generated in [("c", c, cp, cb), ("java", java, jp, jb)]:
        for label, adapter in [("production", production), ("probe", probe)]:
            output = work / (language + label)
            result = subprocess.run([str(v) for v in [adapter, "--bundle", output, *records]],
                                    capture_output=True, text=True, timeout=180,
                                    env=dict(os.environ, TMPDIR=str(scratch)))
            assert result.returncode == 0, result.stderr
            assert files(output) == files(generated)
            if label == "probe":
                observed = [line.split("\t")[2:] for line in result.stderr.splitlines()
                            if line.startswith("CHAR_SOURCE_TYPES\t")]
                assert sorted(observed) == sorted([["32", "4"], ["6", "0"], ["6", "0"]]), observed
        faults = ["function_kind", "field_kind", "field_owner", "target_field_owner", "declaration", "owner"]
        if language == "c":
            faults += ["call_argument", "call_result"]
        else:
            faults += ["target_function_kind"]
        for fault in faults:
            for existing in [False, True]:
                output = work / f"{language}-{fault}-{existing}"
                if existing:
                    output.mkdir()
                    (output / "sentinel").write_bytes(b"preserved\x00\xff")
                before = files(output) if existing else {}
                result = subprocess.run([str(v) for v in [probe, "--bundle", output, *records]],
                                        capture_output=True, text=True, timeout=180,
                                        env=dict(os.environ, TMPDIR=str(scratch), POLYRUST_CHARACTER_FAULT=fault))
                expected = ("C source field facts disagree with target declaration" if language == "c"
                            else "Java source field type facts disagree with target representation") if fault == "target_field_owner" else "source type facts differ from original compiler declarations"
                if fault == "call_argument":
                    expected = "C original source call argument type mismatch"
                elif fault == "call_result":
                    expected = "C original source call arity or result type mismatch"
                elif fault == "target_function_kind":
                    expected = "foreign original Rust signature differs from Java source type facts"
                assert result.returncode != 0 and expected in result.stderr, result.stderr
                assert "error[E" not in result.stderr
                assert output.exists() == existing and files(output) == before
                assert not list(scratch.iterdir()) and not list(work.glob(".polyrust-stage-*"))
    assert original == {p: p.read_bytes() for p in original}
    foreign = work / "character_middle.rs"
    changed_records = [foreign if value == sources["middle"] else value for value in records]
    for member in ["identity", "compare"]:
        diagnostic = ("foreign public binding requires an ordinary public module constant" if member == "identity"
                      else "foreign or unsupported module binding")
        foreign.write_text(sources["middle"].read_text() + f"\npub use character_leaf::{member} as foreign_alias;\n")
        for language, adapter in [("c", c), ("java", java)]:
            for existing in [False, True]:
                output = work / f"foreign-{member}-{language}-{existing}"
                if existing:
                    output.mkdir()
                    (output / "sentinel").write_bytes(b"preserved")
                before = files(output)
                result = subprocess.run([str(v) for v in [adapter, "--bundle", output, *changed_records]],
                                        capture_output=True, text=True, timeout=180,
                                        env=dict(os.environ, TMPDIR=str(scratch)))
                assert result.returncode != 0 and diagnostic in result.stderr, result.stderr
                assert "error[E" not in result.stderr
                assert files(output) == before and output.exists() == existing
                assert not list(scratch.iterdir()) and not list(work.glob(".polyrust-stage-*"))
    print("44 original signatures and four source fields per target; read-only probes preserve output; "
          "30 atomic source/target kind/field/declaration/owner/call/import corruptions and eight foreign re-exports rejected")


if __name__ == "__main__":
    main()
