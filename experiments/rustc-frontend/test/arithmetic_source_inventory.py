"""Original modules, public/private signatures, imports and exact bundle payloads."""
from binary64_inventory import bundle
from java_fixture_native import exports, check_inventory_oracle
from arithmetic_source_oracle import OPERATIONS


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
        assert "system_libraries" not in c[owner]
        for field in ["header", "implementation"]:
            text = (c_dir / c[owner][field]).read_text()
            assert "math.h" not in text and "#include \"runtime" not in text.lower()
        assert "Runtime" not in (java_dir / java[owner]["source"]).read_text()
    kind, module = bindings[middle, "type", "operations"]
    assert kind == "module"
    expected = {(leaf, "value", name) for name in ["left", "right"]}
    expected.add((middle, "type", "operations"))
    expected.update((owner, "value", name) for owner in [module, root] for name in OPERATIONS)
    check_inventory_oracle(bindings, expected)
    functions = {d["id"]: d for api in java.values() for d in api["declarations"] if d["kind"] == "function"}
    native = {d["id"]: d for api in c.values() for d in api["functions"]}
    assert len(functions) == 17 and set(functions) == set(native)
    public_ids = {value[1] for value in bindings.values() if value[0] == "declaration"}
    for identity, declaration in functions.items():
        assert declaration["result"] == "f64"
        assert declaration["parameters"] == ["f64"] * (1 if identity in {
            d["id"] for d in java[leaf]["declarations"] if d["kind"] == "function"
        } else 2)
        assert declaration["externally_reachable"] == (identity in public_ids)
        assert native[identity]["parameters"] == declaration["parameters"]
        assert native[identity]["return"] == declaration["result"]
        assert native[identity]["linkage"] == ("external" if identity in public_ids else "internal")
    private, = set(functions) - public_ids
    assert native[private]["symbol"] not in (c_dir / c[leaf]["header"]).read_text()
    for owner, dependency, names in [(middle, leaf, ["left", "right"]), (root, module, OPERATIONS)]:
        wanted = {bindings[dependency, "value", name][1] for name in names}
        assert {item["id"] for item in c[owner]["imports"]} == wanted
        for imported in c[owner]["imports"]:
            declaration = functions[imported["id"]]
            assert imported["parameters"] == declaration["parameters"] and imported["return"] == declaration["result"]
            assert imported["header"] == c[imported["owner"]]["header"]
            assert imported["symbol"] == native[imported["id"]]["symbol"]
    grouped = functions[bindings[module, "value", "grouped"][1]]
    assert any("addition rounds before multiplication" in text for text in grouped["documentation"])
    return order, module, java, c, bindings, functions, native
