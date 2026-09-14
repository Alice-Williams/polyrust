"""Handwritten C consumer protocol resolved through descriptive public bindings."""
import json
from pathlib import Path
import re
import sys

bundle, work = map(Path, sys.argv[1:])
index = json.loads((bundle / "bundle.json").read_text())
assert len(index["members"]) == 4
members = [json.loads((bundle / item["manifest"]).read_text()) for item in index["members"]]


def bindings(manifest):
    assert len(manifest["modules"]) == 1
    module = manifest["modules"][0]
    assert set(module) == {"id", "bindings"} and module["id"] == manifest["root"]
    rows = module["bindings"]
    assert len({item["name"] for item in rows}) == len(rows)
    for item in rows:
        assert set(item) == {"namespace", "name", "kind", "target"}
        assert item["namespace"] == "value" and item["kind"] == "function"
    functions = {item["id"]: item for item in manifest["functions"]}
    assert len(functions) == len(manifest["functions"])
    return {item["name"]: functions[item["target"]]["symbol"] for item in module["bindings"] if item["kind"] == "function"}


root = next(member for member in members if member["root"] == index["root"])
leaf = next(member for member in members if set(bindings(member)) == {"score", "alias", "flip", "zero", "choose"})
left = next(member for member in members if set(bindings(member)) == {"score", "flip", "zero"})
right = next(member for member in members if set(bindings(member)) == {"score", "flip"})
r, a, b, c = map(bindings, [root, leaf, left, right])
assert set(r) == {"score", "alias", "alternate", "flip", "zero", "choose"}
assert r["score"] == r["alias"] and a["score"] == a["alias"]
assert len({r["score"], a["score"], b["score"], c["score"]}) == 4
assert [len(member["functions"]) for member in [leaf, left, right, root]] == [6, 3, 2, 5]
assert [len(member["imports"]) for member in [leaf, left, right, root]] == [0, 4, 3, 5]

def expected_import(owner, name, result, parameters):
    module = next(item for item in owner["modules"] if item["id"] == owner["root"])
    target = next(item["target"] for item in module["bindings"] if item["name"] == name)
    function = next(item for item in owner["functions"] if item["id"] == target)
    return {"id": target, "owner": owner["root"], "header": owner["header"],
            "symbol": function["symbol"], "return": result, "parameters": parameters}


# Explicit source-fixture signatures and call edges, not copied from imports.
leaf_score = expected_import(leaf, "alias", "i32", ["i32"])
leaf_choose = expected_import(leaf, "choose", "i32", ["bool", "i32", "i32"])
leaf_flip = expected_import(leaf, "flip", "bool", ["bool"])
leaf_zero = expected_import(leaf, "zero", "i32", [])
for member, expected in [
    (leaf, []),
    (left, [leaf_score, leaf_choose, leaf_flip, leaf_zero]),
    (right, [leaf_choose, leaf_flip, leaf_zero]),
    (root, [
        expected_import(left, "score", "i32", ["i32"]),
        expected_import(left, "flip", "bool", ["bool"]),
        expected_import(left, "zero", "i32", []),
        expected_import(right, "score", "i32", ["i32"]),
        expected_import(right, "flip", "bool", ["bool"]),
    ]),
]:
    assert member["imports"] == sorted(expected, key=lambda item: item["id"])

leaf_header = (bundle / leaf["header"]).read_text()
leaf_source = (bundle / leaf["implementation"]).read_text()
for member in members:
    assert "struct " not in (bundle / member["header"]).read_text()
def check_docs(member, public, private):
    header = (bundle / member["header"]).read_text()
    source = (bundle / member["implementation"]).read_text()
    all_texts = [(bundle / item[field]).read_text()
                 for item in members for field in ["header", "implementation"]]
    for text in public + private:
        assert sum(value.count(text) for value in all_texts) == 1, text
    for text in public:
        assert header.count(text) == 1 and text not in source, text
    for text in private:
        assert source.count(text) == 1 and text not in header, text


check_docs(leaf, [
    "Shared leaf crate documentation.",
    "Public leaf score behind two aliases.",
    "Leaf boolean operation.",
    "Leaf zero-argument operation.",
    "Leaf mixed-parameter operation.",
], [
    "Private concrete storage stays inside the leaf implementation.",
    "Private stored scalar.",
    "Private leaf helper.",
    "Private ancestor of a publicly aliased operation.",
    "A private boolean helper exercises same-crate scalar calls in both branches.",
])
check_docs(left, ["Left consumer of the shared leaf."], [])
check_docs(right, ["Right consumer of the same shared leaf."], [])
check_docs(root, [
    "Root crate diamond documentation.",
    "Nested foreign calls retain independent owning implementations.",
], [])

private_count = 0
for member in members:
    public_ids = {binding["target"] for module in member["modules"]
                  for binding in module["bindings"] if binding["kind"] == "function"}
    for function in member["functions"]:
        assert (function["linkage"] == "external") == (function["id"] in public_ids)
        if function["id"] not in public_ids:
            # Both deliberately private fixture helpers take one scalar.
            # Owned-function manifests are descriptive, without signatures.
            arguments = "0"
            (work / f"private-function-{private_count}.c").write_text(
                f'#include "{member["header"]}"\nint main(void) {{ return {function["symbol"]}({arguments}); }}\n')
            private_count += 1
assert private_count == 2
record = re.search(r'\bstruct\s+(poly_\w+)\s*\{', leaf_source)
assert record, leaf_source
(work / "private-record.c").write_text(f'#include "{leaf["header"]}"\nint main(void) {{ struct {record[1]} value; return (int)sizeof(value); }}\n')

calls = [f'{a["score"]}(value)', f'{a["alias"]}(value)', f'{b["score"]}(value)', f'{c["score"]}(value)',
         f'{r["score"]}(value)', f'{r["alias"]}(value)', f'{r["alternate"]}(value)',
         f'{a["flip"]}(0)', f'{a["flip"]}(1)', f'{c["flip"]}(0)', f'{c["flip"]}(1)',
         f'{r["flip"]}(0)', f'{r["flip"]}(1)', f'{r["zero"]}()',
         f'{r["choose"]}(1, value, 0)', f'{r["choose"]}(0, value, 0)']
formats = ['"%" PRId32'] * 7 + ['"%d"'] * 6 + ['"%" PRId32'] * 3
body = ('int main(void) {\n    int32_t value = 0;\n'
        '    while (scanf("%" SCNd32, &value) == 1) {\n'
        '        printf(' + ' " " '.join(formats) + ' "\\n",\n            ' + ', '.join(calls) + ');\n'
        '    }\n    return 0;\n}\n')
headers = [f'#include "{member["header"]}"\n' for member in members]
standard = '#include <inttypes.h>\n#include <stdio.h>\n'
for order, preamble in enumerate([
    ''.join(headers) + headers[0] + standard,
    standard + ''.join(reversed(headers)) + headers[-1],
    '#include <inttypes.h>\n' + ''.join(headers[:2]) + '#include <stdio.h>\n' + ''.join(headers[2:]) + ''.join(headers),
]):
    (work / f"consumer{order}.c").write_text(preamble + body)
(work / "members.json").write_text(json.dumps(members))
