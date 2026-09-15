"""Mutations of fixed fixture copies; never a production rendering path."""
import re
from short_circuit_mutations import definition


def mutate(text, member, markers, width, java, fault, operator=None):
    match, start, end = definition(text, member)
    params = [re.search(r"\b(\w+)\s*$", p).group(1) for p in match[1].split(",")]
    assert len(params) == 2
    ty = ("int" if width == 32 else "long") if java else f"int{width}_t"
    left, right = markers[f"L{width}"], markers[f"R{width}"]
    if fault == "drop":
        unused = "" if java else f"(void){params[1]};"
        body = f"\n{unused}\nreturn {left}({params[0]});\n"
    else:
        assert fault == "reordered" and operator in ["&", "|", "^"]
        body = (f"\n{ty} right = {right}({params[1]});\n"
                f"{ty} left = {left}({params[0]});\nreturn left {operator} right;\n")
    return text[:start] + body + text[end:]
