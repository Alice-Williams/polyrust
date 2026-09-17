"""Test-copy faults preserve compilation, without production mutation hooks."""
import re
from short_circuit_mutations import definition


def mutate(text, member, marker, java, variant):
    match, start, end = definition(text, member)
    parameter = re.search(r"\b(\w+)\s*$", match.group(1)).group(1)
    if variant == "wrong_comparison":
        body = f"\nreturn {parameter} == {parameter};\n"
    elif variant == "always_false":
        false = "false" if java else "0"
        discard = "" if java else f"(void){parameter};\n"
        body = f"\n{discard}return {false};\n"
    elif variant == "dropped":
        body = f"\nreturn {parameter} != {parameter};\n"
    else:
        assert variant == "duplicated"
        prefix = "" if java else "(void)"
        body = f"\n{prefix}{marker}({parameter});\n" + text[start:end]
    return text[:start] + body + text[end:]
