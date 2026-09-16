"""Real checked compiler export identities; target publication remains disabled."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    emitter, probe, mismatch = [Path(value).resolve() for value in sys.argv[1:]]
    work = Path(os.environ["TEST_TMPDIR"]) / "foreign-exports"
    work.mkdir()
    scratch = work / "scratch"
    scratch.mkdir()
    sources, metadata, dependencies = {}, {}, {}

    def run(tool, arguments, accepted=True, diagnostic=None):
        result = subprocess.run([str(value) for value in [tool, *arguments]],
                                capture_output=True, text=True, timeout=120,
                                env=dict(os.environ, TMPDIR=str(scratch)))
        assert (result.returncode == 0) == accepted, (arguments, result.stdout, result.stderr)
        if diagnostic:
            assert diagnostic in result.stderr, result.stderr
        assert not list(scratch.iterdir())
        return result.stdout

    def records(root):
        result, seen = ["--root", "alias." + root], set()

        def visit(owner):
            if owner in seen:
                return
            seen.add(owner)
            result.extend(["--crate", owner, "alias." + owner, sources[owner],
                           sources[owner].name, metadata[owner]])
            for dep in dependencies[owner]:
                result.extend(["--dependency", dep, "alias." + dep])
            for dep in dependencies[owner]:
                visit(dep)

        visit(root)
        return result

    def owner(name, body, deps=()):
        sources[name] = work / (name + ".rs")
        sources[name].write_text(body + "\n")
        metadata[name] = work / ("lib" + name + ".rmeta")
        dependencies[name] = deps
        run(emitter, records(name))

    constants = "\n".join(f"pub const C{i}:i32={i};" for i in range(2049))
    owner("first", constants + """
pub const TEXT: &str = "classification, not type admission";
pub fn function() -> i32 { 1 }
pub static STATIC: i32 = 1;
pub struct Object { pub value: i32 }
impl Object { pub const ASSOCIATED: i32 = 1; }
pub mod nested { pub const VALUE: i32 = 7; }
const PRIVATE: i32 = 1;
pub(crate) const RESTRICTED: i32 = 2;
""")
    owner("second", constants)
    owner("left", "pub use first::C0 as RENAMED;", ["first"])
    owner("right", "pub use first::C0 as RENAMED;", ["first"])
    original = {path: path.read_bytes() for path in [*sources.values(), *metadata.values()]}
    externs = ["-Ldependency=" + str(work)]
    for name, path in metadata.items():
        externs += ["--extern", name + "=" + str(path)]

    def case(name, body, error=None):
        path = work / (name + ".rs")
        path.write_text(body + "\n")
        output = run(probe, [path, *externs], error is None, error)
        assert original == {path: path.read_bytes() for path in original}
        return output

    def rows(output, prefix):
        return [line.split() for line in output.splitlines() if line.startswith(prefix + " ")]

    def alias(output, name):
        found = [row[-1] for row in rows(output, "BIND") if row[2] == name]
        assert len(found) == 1, (name, output)
        return found[0]

    direct = case("direct", "pub use first::C0;")
    assert not rows(direct, "OWNED")
    assert len(rows(direct, "FOREIGN")) == 1
    defining = alias(direct, "C0")
    assert rows(direct, "FOREIGN")[0][1:] == ["first::C0", defining]
    assert "PRODUCTION_REJECTED" in direct
    diamond = case("diamond", """
pub use left::RENAMED as LEFT;
pub use right::RENAMED as RIGHT;
pub use first::C0 as DIRECT;
pub mod aliases { pub use left::RENAMED as NESTED; pub use crate::aliases as cycle; }
pub use aliases as alternate;
""")
    assert len(rows(diamond, "FOREIGN")) == 1 and not rows(diamond, "OWNED")
    assert all(alias(diamond, name) == defining for name in ["LEFT", "RIGHT", "DIRECT", "NESTED"])
    assert "MODULES 2" in diamond and "PRODUCTION_REJECTED" in diamond
    mixed = case("mixed", """
pub const LOCAL: bool = true;
pub fn value() -> i32 { first::C0 }
pub use first::nested::VALUE as EXPORTED;
""")
    assert len(rows(mixed, "OWNED")) == 2 and len(rows(mixed, "FOREIGN")) == 1
    assert rows(mixed, "FOREIGN")[0][1] == "first::nested::VALUE"
    text = case("text", "pub use first::TEXT;")
    assert rows(text, "FOREIGN")[0][1] == "first::TEXT", "classification must not pretend scalar admission"
    distinct = case("distinct", "pub use first::C0 as A; pub use second::C0 as B;")
    assert len(rows(distinct, "FOREIGN")) == 2 and alias(distinct, "A") != alias(distinct, "B")
    local = case("local", "pub const LOCAL:i32=1;")
    assert len(rows(local, "OWNED")) == 1 and "PRODUCTION_REJECTED" not in local

    for name, body, diagnostic in [
        ("local_mismatch", "pub const A:i32=1; pub const B:i32=2;",
         "local binding differs from its checked compiler definition"),
        ("foreign_mismatch", "pub use first::C0 as A; pub use first::C1 as B;",
         "foreign binding compiler identity disagrees"),
    ]:
        path = work / (name + ".rs")
        path.write_text(body + "\n")
        run(mismatch, [path, *externs], False, diagnostic)

    for name, body, error in [
        ("function", "pub use first::function;", "ordinary public module constant"),
        ("static", "pub use first::STATIC;", "ordinary public module constant"),
        ("module", "pub use first::nested;", "foreign or unsupported module"),
        ("type", "pub use first::Object;", "mapping is not implemented"),
        ("private", "pub use first::PRIVATE;", "private"),
        ("restricted", "pub use first::RESTRICTED;", "private"),
        ("associated", "pub use first::Object::ASSOCIATED;", "unresolved import"),
    ]:
        case(name, body, error)

    def many(count, owned=0):
        return "\n".join(
            f"pub use " + (f"first::C{i}" if i < 2048 else f"second::C{i - 2048}") +
            f" as V{i};" for i in range(count)
        ) + "\n" + "\n".join(f"pub const L{i}:i32={i};" for i in range(owned))

    boundary = case("at_limit", many(4096))
    assert len(rows(boundary, "FOREIGN")) == 4096 and not rows(boundary, "OWNED")
    mixed_boundary = case("mixed_limit", many(4095, 1))
    assert len(rows(mixed_boundary, "FOREIGN")) == 4095 and len(rows(mixed_boundary, "OWNED")) == 1
    for name, body in [("over_limit", many(4097)), ("mixed_over", many(4096, 1))]:
        case(name, body, "public declaration inventory size budget exceeded")
    print("Original foreign constant identities, aliases/diamonds, typed separation, "
          "strict production rejection, unsupported kinds and combined 4096 boundary")


if __name__ == "__main__":
    main()
