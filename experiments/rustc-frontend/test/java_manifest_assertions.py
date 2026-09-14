"""Exact JSON content/completeness against independent typed compiler probe rows."""
from collections import defaultdict
from copy import deepcopy


def check(manifests, rows):
    source_rows = [row for row in rows if row[0] == "SOURCE"]
    module_rows = [row for row in rows if row[0] == "MODULE"]
    target_rows = [row for row in rows if row[0] == "TARGET"]
    targets = {}
    for row in target_rows:
        target_path = dict(package=bytes.fromhex(row[3]).decode(), owners=row[4].split(",") if row[4] else [], member=row[5])
        targets[row[1]] = (dict(kind="field", owner=target_path, member=row[6]) if row[2] == "field"
                          else dict(kind="declaration", path=target_path))
    assert len(targets) == len(target_rows) and set(targets) == {row[1] for row in source_rows}
    source_docs = defaultdict(list)
    module_docs = defaultdict(list)
    bindings = defaultdict(list)
    for row in rows:
        if row[0] in {"SOURCEDOC", "MODULEDOC"}:
            destination = source_docs if row[0] == "SOURCEDOC" else module_docs
            destination[row[1]].append(bytes.fromhex(row[2]).decode())
        elif row[0] == "BINDING":
            bindings[row[1]].append(dict(namespace=row[2], name=bytes.fromhex(row[3]).decode(),
                                         target=dict(kind=row[4], id=row[5])))
    assert len({row[1] for row in source_rows}) == len(source_rows)
    assert len({row[1] for row in module_rows}) == len(module_rows)
    observed_sources = set()
    observed_modules = set()
    for manifest in manifests:
        crate_id = manifest["root"].split(":")[0]
        expected_sources = {row[1]: row for row in source_rows if row[1].split(":")[0] == crate_id}
        expected_modules = {row[1]: row for row in module_rows if row[1].split(":")[0] == crate_id}
        declarations = manifest["declarations"]
        assert [item["id"] for item in declarations] == sorted(expected_sources), "private/public declaration inventory differs"
        for item in declarations:
            row = expected_sources[item["id"]]
            observed_sources.add(item["id"])
            expected = dict(id=row[1], kind=row[2], module=row[3],
                visibility=dict(kind="public") if row[4] == "public" else dict(kind="restricted_to", module=row[4]),
                externally_reachable=row[5] == "true",
                location=dict(file=bytes.fromhex(row[6]).decode(), line=int(row[7]), column=int(row[8])),
                documentation=source_docs[row[1]])
            target = item["target"]
            if row[2] == "field":
                assert set(target) == {"kind", "owner", "member"} and target["kind"] == "field"
                target_path = target["owner"]
                suffix = "." + target["member"]
                expected.update(owner=row[11], scalar=row[10])
            else:
                assert set(target) == {"kind", "path"} and target["kind"] == "declaration"
                target_path = target["path"]
                suffix = ""
                if row[2] == "function":
                    expected.update(parameters=row[10].split(",") if row[10] else [], result=row[11])
            assert set(target_path) == {"package", "owners", "member"}
            assert ".".join([target_path["package"], *target_path["owners"], target_path["member"]]) + suffix == row[9]
            expected["target"] = targets[row[1]]
            assert item == expected, (item, expected)
        assert [item["id"] for item in manifest["modules"]] == sorted(expected_modules)
        for item in manifest["modules"]:
            row = expected_modules[item["id"]]
            observed_modules.add(item["id"])
            expected = dict(id=row[1], parent=None if row[2] == "-" else row[2],
                location=dict(file=bytes.fromhex(row[3]).decode(), line=int(row[4]), column=int(row[5])),
                documentation=module_docs[row[1]], bindings=bindings[row[1]])
            assert item == expected, (item, expected)
    assert observed_sources == {row[1] for row in source_rows}
    assert observed_modules == {row[1] for row in module_rows}


def prove_rejections(manifests, rows):
    """The oracle itself must detect omissions and independent metadata mutations."""
    def rejects(edit):
        changed = deepcopy(manifests)
        edit(changed)
        try:
            check(changed, rows)
        except AssertionError:
            return
        raise AssertionError("independent manifest oracle accepted mutation")

    for kind in ["function", "record", "field"]:
        owner, position = next((owner, position) for owner, manifest in enumerate(manifests)
            for position, item in enumerate(manifest["declarations"])
            if item["kind"] == kind and not item["externally_reachable"])
        rejects(lambda changed: changed[owner]["declarations"].pop(position))
        for key, value in [("documentation", ["altered"]), ("module", "wrong"),
                           ("visibility", dict(kind="public")), ("target", dict(kind="declaration", path=dict(package="wrong", owners=[], member="wrong")))]:
            rejects(lambda changed: changed[owner]["declarations"][position].__setitem__(key, value))
    rejects(lambda changed: changed[0]["declarations"].append(changed[0]["declarations"][0]))
    rejects(lambda changed: changed[0]["modules"].pop())
    rejects(lambda changed: changed[0]["modules"][0].__setitem__("documentation", ["altered"]))
    owner, position = next((owner, position) for owner, manifest in enumerate(manifests)
        for position, module in enumerate(manifest["modules"]) if module["bindings"])
    rejects(lambda changed: changed[owner]["modules"][position]["bindings"].pop())
