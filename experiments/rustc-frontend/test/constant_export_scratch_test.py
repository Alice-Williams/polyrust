"""Run the mutation-copy contract unprivileged even in the root dev container."""
import ast
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile


PROBE = r"""
import os
from pathlib import Path
import runpy
import shutil
import sys

assert os.geteuid() != 0, "permission proof must run unprivileged"
source, work, helper = map(Path, sys.argv[1:])
raw = work / "raw"
shutil.copytree(source, raw)
try:
    (raw / "bundle.json").write_text("incorrect overwrite")
except PermissionError:
    pass
else:
    raise AssertionError("negative control did not reproduce read-only failure")
copy = runpy.run_path(str(helper))["writable_copy"]
target = work / "editable"
copy(source, target)
(target / "bundle.json").write_text("mutated schema")
(target / "nested" / "Generated.java").write_text("mutated producer")
(target / "nested" / "new.txt").write_text("new compilation output")
(target / "classes").mkdir()
assert (target / "tool").stat().st_mode & 0o111 == 0o111
try:
    copy(source, target)
except FileExistsError:
    pass
else:
    raise AssertionError("copy must not overwrite an existing destination")
"""

calls = [node for node in ast.walk(ast.parse(Path(sys.argv[2]).read_text()))
         if isinstance(node, ast.Call)]
assert sum(isinstance(node.func, ast.Name) and node.func.id == "writable_copy"
           for node in calls) == 3, "schema, producer and example copies must use the helper"
assert not any(isinstance(node.func, ast.Attribute) and node.func.attr == "copytree"
               for node in calls), "native proof bypassed writable-copy contract"

with tempfile.TemporaryDirectory(prefix="polyrust-alias-permissions-") as temporary:
    root = Path(temporary)
    root.chmod(0o755)
    helper = root / "copy_helper.py"
    shutil.copyfile(Path(sys.argv[1]), helper)
    helper.chmod(0o644)
    source = root / "source"
    nested = source / "nested"
    nested.mkdir(parents=True)
    for path in [source / "bundle.json", nested / "Generated.java", source / "tool"]:
        path.write_text("immutable input\n")
        path.chmod(0o555 if path.name == "tool" else 0o444)
    nested.chmod(0o555)
    source.chmod(0o555)
    work = root / "scratch"
    work.mkdir()
    work.chmod(0o777)
    original_modes = {path: stat.S_IMODE(path.stat().st_mode)
                      for path in [source, *source.rglob("*")]}
    identity = dict(user=65534, group=65534, extra_groups=[]) if os.geteuid() == 0 else {}
    subprocess.run([sys.executable, "-c", PROBE, str(source), str(work),
                    str(helper)], check=True, **identity)
    for path in [source / "bundle.json", nested / "Generated.java", source / "tool"]:
        assert path.read_text() == "immutable input\n"
        assert not path.stat().st_mode & stat.S_IWUSR
    assert {path: stat.S_IMODE(path.stat().st_mode) for path in original_modes} == original_modes
    for path, mode in original_modes.items():
        copied = work / "editable" / path.relative_to(source)
        expected = mode | stat.S_IWUSR | (stat.S_IXUSR if path.is_dir() else 0)
        assert stat.S_IMODE(copied.stat().st_mode) == expected
    assert (work / "editable/bundle.json").read_text() == "mutated schema"
    assert (work / "editable/nested/Generated.java").read_text() == "mutated producer"
    # TemporaryDirectory cleanup needs writable directories on an ordinary runner.
    for path in [source, nested, work / "raw", work / "raw/nested"]:
        path.chmod(0o755)
print("Unprivileged readonly failure reproduced; scratch-only mutation passes.")
