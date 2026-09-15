"""Require exact operation/slot inventories and reject invalid Rust first."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "scalar-record Box capability proof passed"
IDENTITIES = "scalar-record identities, closed kinds and three executable slots passed"


def run(probe, source, dependency=None):
    args = [str(probe), str(source)]
    if dependency is not None:
        args += ["--input", str(dependency)]
    return subprocess.run(args, capture_output=True, text=True, check=False, timeout=60)


def main():
    probe, fixture, limit = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture, limit)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and IDENTITIES in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="scalar-box-") as directory:
        work = Path(directory)
        limits = work / limit.name
        limits.write_text(limit.read_text())
        for name, source, error in [
            ("moved", "struct R { value: i32 }\nfn bad(a: i32) { let r = R { value: a }; let x = Box::new(r); let y = Box::new(r); }", "E0382"),
            ("borrowed", "struct R { value: i32 }\nfn bad(a: i32) { let mut r = R { value: a }; let x = &r; let y = &mut r; y.value = a; let b = Box::new(R { value: x.value }); }", "E0502"),
            ("allocator", "struct R { value: i32 }\nfn bad(a: i32) { let b = Box::new_in(R { value: a }, std::alloc::System); }", "E0658"),
        ]:
            path = work / (name + ".rs")
            path.write_text(source)
            result = run(probe, path)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout and IDENTITIES not in result.stdout
        for name, source in {
            "representation": original.replace("pub struct Record", "#[repr(C)]\npub struct Record", 1),
            "order": original.replace("number: i32,\n    enabled: bool,", "enabled: bool,\n    number: i32,", 1),
            "inventory": original.replace("pub fn mixed(", "pub fn changed_mixed(", 1),
            "empty": "// No empty capability proof.\n",
        }.items():
            assert source != original, name
            path = work / (name + ".rs")
            path.write_text(source)
            result = run(probe, path, None if name == "empty" else limits)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("scalar Box source controls and nonempty inventory passed")


if __name__ == "__main__":
    main()
