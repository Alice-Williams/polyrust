"""Exact source exports/signatures and original dependency owner identities."""
from binary64_inventory import bundle
from java_fixture_native import exports, check_inventory_oracle
from floating_oracle import OPERATIONS, LITERALS


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    middle, = java[root]["dependencies"]
    leaf, = java[middle]["dependencies"]
    assert java[leaf]["dependencies"] == []
    order = [leaf, middle, root]
    bindings = {}
    for owner in order:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
    names = {leaf: ["identity", "negate"], middle: ["relay", "negate"],
             root: [*OPERATIONS, *LITERALS]}
    check_inventory_oracle(bindings, {(owner, "value", name) for owner, items in names.items() for name in items})
    functions = {d["id"]: d for api in java.values() for d in api["declarations"] if d["kind"] == "function"}
    native = {d["id"]: d for api in c.values() for d in api["functions"]}
    assert len(functions) == 15 and set(functions) == set(native)
    for owner, items in names.items():
        for name in items:
            declaration = functions[bindings[owner, "value", name][1]]
            assert declaration["parameters"] == ([] if owner == root and name in LITERALS else ["f64"])
            assert declaration["result"] == "f64" and declaration["externally_reachable"]
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
    marker, = [d for d in functions.values()
               if [s.strip() for s in d["documentation"]] == ["Trace G."]]
    assert not marker["externally_reachable"] and marker["parameters"] == ["f64"]
    fields = [d for d in java[root]["declarations"] if d["kind"] == "field"]
    assert len(fields) == 1 and fields[0]["scalar"] == "f64" and not fields[0]["externally_reachable"]
    return order, java, c, bindings, functions, native, marker["id"]
