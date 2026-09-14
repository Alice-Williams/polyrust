"""Exact public object ownership and generated unresolved foreign call inventory."""
import json
from pathlib import Path
import subprocess
import sys

members_file, source_name, object_file, optimization = sys.argv[1:]
member = next(item for item in json.loads(Path(members_file).read_text()) if item["implementation"] == source_name)
defined = subprocess.run(["nm", "--defined-only", object_file], capture_output=True, text=True, check=True).stdout
public = {line.split()[-1] for line in defined.splitlines() if len(line.split()) >= 3 and line.split()[-2] == "T"}
public_ids = {binding["target"] for module in member["modules"]
              for binding in module["bindings"] if binding["kind"] == "function"}
functions = {item["id"]: item for item in member["functions"]}
assert public_ids <= functions.keys()
assert {item["id"] for item in functions.values() if item["linkage"] == "external"} == public_ids
expected = {functions[identity]["symbol"] for identity in public_ids}
assert public == expected, (object_file, public, expected)
if optimization == "0":
    # O2 may legitimately clone/remove private helpers; O0 retains the exact
    # owned function inventory under the matrix's explicit no-inline flags.
    owned = {line.split()[-1] for line in defined.splitlines()
             if len(line.split()) >= 3 and line.split()[-2] in {"T", "t"}
             and line.split()[-1].startswith("poly_fn_")}
    expected_owned = {item["symbol"] for item in functions.values()}
    assert owned == expected_owned, (object_file, owned, expected_owned)
undefined = subprocess.run(["nm", "-u", object_file], capture_output=True, text=True, check=True).stdout
generated = {line.split()[-1] for line in undefined.splitlines() if line.split() and line.split()[-1].startswith("poly_")}
expected = {item["symbol"] for item in member["imports"]}
assert generated == expected, (object_file, generated, expected)
