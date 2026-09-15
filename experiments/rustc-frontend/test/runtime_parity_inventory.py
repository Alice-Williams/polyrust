"""Lexical inventory drift guard, not a parser, admission proof or parity oracle."""
import copy
import json
from pathlib import Path
import re
import sys

FIELDS = {"java_capabilities", "c_intrinsics", "java_helpers", "c_sections"}


def observed(java, helpers, c, header, source):
    slots = java.split("pub type JavaCapabilitySlots = capability_slots!(", 1)[1].split(");", 1)[0]
    registrations = re.findall(r"implemented CheckedJavaMapping<(\w+)>", slots)
    assert registrations and len(registrations) == len(set(registrations))
    built = re.findall(r"\.support\((Java\w+)\)", java.split("pub(crate) fn java_capabilities()", 1)[1])
    assert len(built) == len(set(built)) and set(built) == set(registrations)
    helper_enum = helpers.split("pub enum JavaRuntimeHelper {", 1)[1].split("}", 1)[0]
    helper_names = re.findall(r"^\s*(\w+),\s*$", helper_enum, re.MULTILINE)
    helper_all = re.search(r"pub const ALL:\s*\[Self;\s*\d+\]\s*=\s*\[(.*?)\];", helpers, re.DOTALL)
    assert helper_all is not None
    registered_helpers = re.findall(r"Self::(\w+)", helper_all.group(1))
    assert helper_names and len(helper_names) == len(set(helper_names))
    assert len(registered_helpers) == len(set(registered_helpers))
    assert set(registered_helpers) == set(helper_names), "helper variants and executable ALL registrations differ"
    validate = c.split("fn validate_expression(", 1)[1]
    allowlist = validate.split("if !matches!(", 1)[1].split(") {", 1)[0]
    intrinsics = re.findall(r"Intrinsic::(\w+)", allowlist)
    assert intrinsics and len(intrinsics) == len(set(intrinsics))
    sections = set()
    for template in [header, source]:
        entries = re.findall(r"^/\* POLYRUST-BEGIN ([\w.-]+) \*/$", template, re.MULTILINE)
        assert entries and len(entries) == len(set(entries))
        sections.update(entries)
    return dict(java_capabilities=set(registrations), c_intrinsics=set(intrinsics),
                java_helpers=set(helper_names), c_sections=sections)


def verify(inventory, actual):
    assert inventory["schema_version"] == 1
    groups = inventory["groups"]
    assert set(groups) == {f"M35-03A-{i:02}" for i in range(2, 7)}
    for group in groups.values():
        assert set(group) == FIELDS
    for field in FIELDS:
        names = [name for group in groups.values() for name in group[field]]
        assert len(names) == len(set(names)), (field, "duplicate disposition")
        assert set(names) == actual[field], (field, set(names) ^ actual[field])
    status = inventory["new_system"]
    partial = status["shared_partial_features"]
    assert len(partial) == len(set(partial)) and set(partial) <= actual["java_capabilities"]
    # No group has complete replacement proof yet. Change this boundary only
    # together with feature-specific evidence and a reviewed inventory schema.
    assert status["full_features"] == []
    assert status["scope"] and len(inventory["c_limits"]) == 4


def main():
    manifest, java, helpers, c, header, source = [Path(p) for p in sys.argv[1:]]
    inventory = json.loads(manifest.read_text())
    sources = [path.read_text() for path in [java, helpers, c, header, source]]
    actual = observed(*sources)
    verify(inventory, actual)
    for field in sorted(FIELDS):
        for mutation in ["missing", "extra", "duplicate"]:
            changed = copy.deepcopy(inventory)
            group = next(g for g in changed["groups"].values() if g[field])
            if mutation == "missing":
                group[field].pop()
            elif mutation == "extra":
                group[field].append("UnclassifiedFutureFeature")
            else:
                group[field].append(group[field][0])
            try:
                verify(changed, actual)
            except AssertionError:
                pass
            else:
                raise AssertionError((field, mutation, "inventory corruption accepted"))
    print("Legacy parity inventory:", {name: len(values) for name, values in actual.items()})
    print("Twelve missing/extra/duplicate inventory controls rejected; no full parity claimed")
    for mutation in ["missing", "duplicate"]:
        changed = list(sources)
        if mutation == "missing":
            changed[1] = changed[1].replace("[Self; 9]", "[Self; 8]", 1).replace("Self::Bytes,", "", 1)
        else:
            changed[1] = changed[1].replace("Self::Bytes,", "Self::Core,", 1)
        assert changed != sources
        try:
            observed(*changed)
        except AssertionError:
            pass
        else:
            raise AssertionError((mutation, "helper registration drift accepted"))
    print("Two executable helper-catalogue registration mutations rejected")


if __name__ == "__main__":
    main()
