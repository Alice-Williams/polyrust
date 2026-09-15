"""Exact public/private signatures and two-crate provenance."""
from i64_inventory import bundle
from java_fixture_native import exports, check_inventory_oracle
from eager_oracle import NAMES


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    leaf, = set(java) - {root}
    assert java[root]["dependencies"] == [leaf] and java[leaf]["dependencies"] == []
    assert not c[leaf]["imports"] and len(c[root]["imports"]) == 1
    bindings = {}
    for owner in java:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
    check_inventory_oracle(bindings, {(root, "value", name) for name in NAMES} | {(leaf, "value", "both")})
    functions = {d["id"]: d for owner in java.values() for d in owner["declarations"] if d["kind"] == "function"}
    native = {d["id"]: d for owner in c.values() for d in owner["functions"]}
    assert len(functions) == len(native) == 17 and set(functions) == set(native)
    for owner, names, count in [(root, NAMES, 3), (leaf, ["both"], 2)]:
        for name in names:
            kind, identity = bindings[owner, "value", name]
            assert kind == "declaration"
            d = functions[identity]
            assert d["parameters"] == ["bool"] * count and d["result"] == "bool"
            assert d["externally_reachable"] and native[identity]["linkage"] == "external"
    markers = {}
    for marker in "ABCD":
        found = [d for d in functions.values() if [s.strip() for s in d["documentation"]] == [f"Trace {marker}."]]
        assert len(found) == 1
        d = found[0]
        assert d["externally_reachable"] == (marker == "D")
        assert d["parameters"] == ["bool"] * (2 if marker == "D" else 1) and d["result"] == "bool"
        markers[marker] = d["id"]
    imported, = c[root]["imports"]
    assert imported["id"] == bindings[leaf, "value", "both"][1] == markers["D"]
    assert imported["owner"] == leaf and imported["header"] == c[leaf]["header"]
    assert imported["parameters"] == ["bool", "bool"] and imported["return"] == "bool"
    assert imported["symbol"] == native[imported["id"]]["symbol"]
    return root, leaf, java, c, bindings, functions, native, markers
