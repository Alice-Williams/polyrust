"""Exact source exports/signatures and original dependency owner identities."""
from truncation_bundle import bundle, link_options
from java_fixture_native import exports, check_inventory_oracle
from truncation_oracle import OPERATIONS, LITERALS


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    middle, = java[root]["dependencies"]
    leaf, = java[middle]["dependencies"]
    assert java[leaf]["dependencies"] == []
    order = [leaf, middle, root]
    for owner in order:
        assert link_options(c[owner]) == ['-lm']
    assert '#include <math.h>' not in (c_dir / c[middle]['implementation']).read_text()
    bindings = {}
    for owner in order:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
    names = {leaf: ["trunc", "integral"], middle: ["relay", "integral"],
             root: [*OPERATIONS, *LITERALS]}
    check_inventory_oracle(bindings, {(owner, "value", name) for owner, items in names.items() for name in items})
    functions = {d["id"]: d for api in java.values() for d in api["declarations"] if d["kind"] == "function"}
    native = {d["id"]: d for api in c.values() for d in api["functions"]}
    assert len(functions) == 15 and set(functions) == set(native)
    for owner, items in names.items():
        for name in items:
            declaration = functions[bindings[owner, "value", name][1]]
            assert declaration["parameters"] == ([] if owner == root and name in LITERALS else ["f64"])
            assert declaration["result"] == "f64"
            assert declaration["externally_reachable"]
    for identity, declaration in functions.items():
        assert native[identity]["parameters"] == declaration["parameters"]
        assert native[identity]["return"] == declaration["result"]
        assert native[identity]["linkage"] == ("external" if declaration["externally_reachable"] else "internal")
    for owner in order:
        for imported in c[owner]["imports"]:
            declaration = functions[imported["id"]]
            assert imported["parameters"] == declaration["parameters"] and imported["return"] == declaration["result"]
            assert imported["header"] == c[imported["owner"]]["header"]
            assert imported["symbol"] == native[imported["id"]]["symbol"]
    expected_imports = {bindings[leaf, "value", name][1] for name in ["trunc", "integral"]}
    assert {entry["id"] for entry in c[middle]["imports"]} == expected_imports
    marker, = [d for d in functions.values()
               if [s.strip() for s in d["documentation"]] == ["Trace A."]]
    assert not marker["externally_reachable"] and marker["parameters"] == ["f64"]
    fields = [d for d in java[root]["declarations"] if d["kind"] == "field"]
    assert len(fields) == 1 and fields[0]["scalar"] == "f64" and not fields[0]["externally_reachable"]
    return order, java, c, bindings, functions, native, marker["id"]
