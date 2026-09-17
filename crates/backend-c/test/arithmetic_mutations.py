"""Test-only, compiling mutations of the fixed typed arithmetic fixture."""
import re
from short_circuit_mutations import definition

LEFT = "poly_fn_00000000000001f5_000000000000000a"
RIGHT = "poly_fn_00000000000001f5_000000000000000b"


def mutate(text, variant):
    index = {"swapped": 1, "grouping": 4, "fused": 5}.get(variant, 0)
    match, start, end = definition(text, f"poly_arithmetic_502_{index}")
    body = text[start:end]
    declarations = []
    for callee in [LEFT, RIGHT]:
        pattern = r"double\s+(\w+)\s*=\s*" + callee + r"\((\w+)\);"
        found = list(re.finditer(pattern, body))
        assert len(found) == 1, (variant, callee, body)
        declarations.append(found[0])
    left, right = [item.group(1) for item in declarations]
    if variant == "dropped":
        item = declarations[0]
        body = body[:item.start()] + f"double {left} = {item.group(2)};" + body[item.end():]
    elif variant == "duplicated":
        body = f"\n(void){LEFT}({declarations[0].group(2)});\n" + body
    elif variant == "reversed":
        a, b = declarations
        assert a.end() < b.start()
        body = body[:a.start()] + b.group(0) + body[a.end():b.start()] + a.group(0) + body[b.end():]
    else:
        old = re.search(r"return\s+(.*);", body)
        assert old is not None
        replacement = {
            "operator": f"{left} - {right}",
            "swapped": f"{right} - {left}",
            "zero_sign": f"({old.group(1)}) == 0.0 ? 0.0 : ({old.group(1)})",
            "grouping": f"{left} + ({right} * {right})",
            "fused": (f"({left} == 0x1.0000002p0 && {right} == 0x1.ffffffcp-1)"
                      f" ? fma({left}, {right}, -1.0) : ({old.group(1)})"),
        }[variant]
        body = body[:old.start()] + "return " + replacement + ";" + body[old.end():]
    changed = text[:start] + body + text[end:]
    return "#include <math.h>\n" + changed if variant == "fused" else changed
