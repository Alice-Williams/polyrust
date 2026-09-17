"""Bounded source discovery: exact constant cap and traversal rejection."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    emitter, probe = [Path(value).resolve() for value in sys.argv[1:]]
    work = Path(os.environ["TEST_TMPDIR"]) / "constant-import-limits"
    work.mkdir()
    scratch = work / "scratch"
    scratch.mkdir()
    sources, metadata = {}, {}
    for owner in ["first", "second"]:
        sources[owner] = work / (owner + ".rs")
        sources[owner].write_text("\n".join(
            f"pub const C{i}: i32 = {i};" for i in range(2049)) + "\n")
        metadata[owner] = work / ("lib" + owner + ".rmeta")

    def record(owner):
        return ["--crate", owner, "limits." + owner, sources[owner],
                sources[owner].name, metadata[owner]]

    def run(adapter, tail, accepted, diagnostic=None):
        result = subprocess.run([str(value) for value in [adapter, *tail]],
                                capture_output=True, text=True, timeout=240,
                                env=dict(os.environ, TMPDIR=str(scratch)))
        assert (result.returncode == 0) == accepted, (adapter, result.stderr)
        if diagnostic:
            assert diagnostic in result.stderr, result.stderr
        assert not list(scratch.iterdir())
        return result

    for owner in sources:
        run(emitter, ["--root", "limits." + owner, *record(owner)], True)
    for case, count, diagnostic in [
        ("at_limit", 4096, None),
        ("over_limit", 4097, "source constant import inventory budget exceeded"),
        ("deep", 0, "budget"),
        ("many", 0, "budget"),
    ]:
        sources["root"] = work / (case + ".rs")
        metadata["root"] = work / ("lib" + case + ".rmeta")
        if count:
            body = "\n".join(f"let _v{i} = " +
                (f"first::C{i}" if i < 2048 else f"second::C{i - 2048}") + ";"
                for i in range(count)) + "\nfirst::C0"
        elif case == "deep":
            body = "{" * 140 + "first::C0" + "}" * 140
        else:
            body = "first::C0;\n" * 100_001 + "0"
        sources["root"].write_text("pub fn value() -> i32 {\n" + body + "\n}\n")
        graph = ["--root", "limits.root", *record("first"), *record("second"),
                 *record("root"), "--dependency", "first", "limits.first",
                 "--dependency", "second", "limits.second"]
        run(emitter, graph, True)
        result = run(probe, [sources["root"], metadata["first"], metadata["second"]],
                     diagnostic is None, diagnostic)
        if diagnostic is None:
            assert result.stdout.strip() == "CONSTANTS 4096", result.stdout
    for case, aliases, reads, expected in [
        ("overlap", 1, 1, 2),
        ("union_at_limit", 2048, 2048, 4096),
        ("union_over_limit", 2048, 2049, None),
    ]:
        sources["root"] = work / (case + ".rs")
        metadata["root"] = work / ("lib" + case + ".rmeta")
        exports = "\n".join(f"pub use first::C{i} as A{i};" for i in range(aliases))
        body = "\n".join(f"let _v{i} = second::C{i};" for i in range(reads))
        # first::C0 is an actual body read as well as two public aliases.
        sources["root"].write_text(exports + "\npub use first::C0 as REPEATED;\n"
            + "pub fn value() -> i32 {\n" + body + "\nfirst::C0\n}\n")
        graph = ["--root", "limits.root", *record("first"), *record("second"),
                 *record("root"), "--dependency", "first", "limits.first",
                 "--dependency", "second", "limits.second"]
        run(emitter, graph, True)
        result = run(probe, [sources["root"], metadata["first"], metadata["second"]],
                     expected is not None,
                     None if expected is not None else "constant import union exceeds 4096 defining declarations")
        if expected is not None:
            assert result.stdout.strip() == f"CONSTANTS {expected}", result.stdout
    print("Deduplicated export/read union accepts 4096 and rejects 4097; "
          "4096 foreign constants discovered (independent target AST limits still apply); 4097 and excessive source traversal/depth rejected")


if __name__ == "__main__":
    main()
