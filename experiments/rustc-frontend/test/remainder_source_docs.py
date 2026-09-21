"""Complete normalized source documentation inventory and mutation controls."""
from collections import Counter
from copy import deepcopy
import re


def must_reject(check):
    try:
        check()
    except (AssertionError, KeyError):
        return
    raise AssertionError("inventory mutation escaped")


def inspect_docs(java_dir, c_dir, java, c, order, module, bindings, private):
    leaf, middle, root = order
    wanted = {
        leaf: ["Original operand producers, with a private implementation detail."],
        middle: ["Checked truncating remainder in its original public module."],
        root: ["Public forwarding without copying its dependency implementation."],
        module: ["Remainder operations preserve source module identity."],
        bindings[leaf, "value", "left"][1]: ["First operand retains its original producer."],
        bindings[leaf, "value", "right"][1]: ["Second operand retains its original producer."],
        bindings[module, "value", "remainder"][1]: ["Truncating quotient; first operand is evaluated before the second."],
        bindings[root, "value", "remainder"][1]: [],
        private: [],
    }

    def metadata(apis):
        records = [d for api in apis.values() for d in [*api["modules"], *api["declarations"]]]
        assert len(records) == len(wanted)
        assert {d["id"]: [line.strip() for line in d["documentation"]] for d in records} == wanted

    metadata(java)
    for owner in order:
        for group in ["modules", "declarations"]:
            for index, record in enumerate(java[owner][group]):
                changed = deepcopy(java)
                changed[owner][group][index]["documentation"] = ["changed"]
                must_reject(lambda: metadata(changed))
        documents = Counter(line for d in [*java[owner]["modules"], *java[owner]["declarations"]]
                            for line in wanted[d["id"]])
        java_text = (java_dir / java[owner]["source"]).read_text()
        c_text = (c_dir / c[owner]["header"]).read_text()

        def rendered(text, is_java):
            pattern = r"^\s*\*\s+(.+)$" if is_java else r"^/\*\s+(.+?)\s+\*/$"
            assert Counter(line.strip() for line in re.findall(pattern, text, re.MULTILINE)) == documents

        for text, is_java in [(java_text, True), (c_text, False)]:
            rendered(text, is_java)
            for document in documents:
                must_reject(lambda: rendered(text.replace(document, "changed"), is_java))
