"""Pinned nested ownership observations, not target admission."""
from pathlib import Path
import subprocess
import sys
import tempfile

probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
result = subprocess.run([str(probe), str(fixture)], capture_output=True,
                        text=True, check=False, timeout=60)
assert result.returncode == 0, result.stderr
assert "nested ownership observation inventory passed" in result.stdout
print(result.stdout)

original = fixture.read_text()
success = "nested ownership observation inventory passed"
with tempfile.TemporaryDirectory(prefix="nested-owned-") as directory:
    source = Path(directory) / "fixture.rs"
    for name, text, error in [
        ("move", "pub fn invalid(x: Outer) { let y = x.nested; let z = x.nested; }", "E0382"),
        ("field", "pub fn invalid(x: Outer) -> Box<bool> { x.nested.first }", "E0308"),
        ("uninitialized", "pub fn invalid() { let x: Inner; drop(x); }", "E0381"),
    ]:
        source.write_text(original + "\n" + text)
        result = subprocess.run([str(probe), str(source)], capture_output=True,
                                text=True, check=False, timeout=60)
        assert result.returncode != 0 and error in result.stderr, (name, result.stderr)
        assert success not in result.stdout
    for name, text in {
        "field": original.replace("let taken = outer.nested.first;", "let taken = outer.nested.second;", 1),
        "constructor": original.replace("let x = Box::new(a);", "let x = Box::new(b);", 1),
        "constructor_identity": original.replace("Box::new", "Box::from", 1),
        "nominal_identity": original.replace("let inner = Inner {", "let inner = AlternateInner {", 1).replace("let outer = Outer {", "let outer = AlternateOuter {", 1) + "\nstruct AlternateInner { first: Box<i32>, second: Box<i32> }\nstruct AlternateOuter { nested: AlternateInner, spare: Box<i32> }\n",
        "mutable_parameter": original.replace("pub fn first(a:", "pub fn first(mut a:", 1),
        "mutable_binding": original.replace("let x =", "let mut x =", 1),
        "generic_record": original.replace("struct Inner {", "struct Inner<T> {").replace("first: Box<i32>,", "first: Box<T>,", 1).replace("nested: Inner,", "nested: Inner<i32>,"),
        "tuple_record": original.replace("struct Outer {\n    nested: Inner,\n    spare: Box<i32>,\n}", "struct Outer(Inner, Box<i32>);").replace("Outer {\n        nested: inner,\n        spare: z,\n    }", "Outer(inner, z)").replace("Outer {\n        spare: z,\n        nested: inner,\n    }", "Outer(inner, z)").replace(".nested", ".0"),
        "inner_order": original.replace("first: x,\n        second: y,", "second: y,\n        first: x,", 1),
        "outer_order": original.replace("nested: inner,\n        spare: z,", "spare: z,\n        nested: inner,", 1),
        "read": original.replace("    *first\n", "    *second\n"),
        "transfer": original.replace("let moved = outer.nested;\n    let taken = moved.first;", "let moved = &outer.nested;\n    let taken = outer.nested.first;"),
        "inventory": original.replace("pub fn whole_inner(", "pub fn changed("),
        "empty": "// No functions\n",
    }.items():
        assert text != original, name
        source.write_text(text)
        result = subprocess.run([str(probe), str(source)], capture_output=True,
                                text=True, check=False, timeout=60)
        assert result.returncode != 0 and success not in result.stdout, name
        assert "panicked at" in result.stderr and "error[E" not in result.stderr, (name, result.stderr)
        if name == "tuple_record":
            assert "named-field record" in result.stderr, result.stderr
        for case, marker in [("transfer", "unique move from"), ("nominal_identity", "one nominal record pair"), ("mutable_parameter", "immutable parameter"), ("mutable_binding", "immutable binding")]:
            if name == case:
                assert marker in result.stderr, result.stderr
print("three invalid Rust and fourteen valid-source observation controls passed")
