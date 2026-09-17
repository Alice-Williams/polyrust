"""Compiling faults of fixed test-only Java arithmetic packages."""
import re
from short_circuit_mutations import definition


def mutate(text, variant):
    index = {"swapped": 1, "grouping": 4, "fused": 5}.get(variant, 0)
    _, start, end = definition(text, f"arithmetic{index}")
    body = text[start:end]
    declarations = []
    for side in ["left", "right"]:
        pattern = r"final double\s+(\w+)\s*=\s*((?:\w+\.)*" + side + r"Identity\((\w+)\));"
        matches = list(re.finditer(pattern, body))
        assert len(matches) == 1, (variant, side, body)
        declarations.append(matches[0])
    left, right = [value.group(1) for value in declarations]
    if variant == "dropped":
        item = declarations[0]
        body = body[:item.start()] + f"final double {left} = {item.group(3)};" + body[item.end():]
    elif variant == "duplicated":
        body = "\n" + declarations[0].group(2) + ";\n" + body
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
                      f" ? java.lang.Math.fma({left}, {right}, -1.0) : ({old.group(1)})"),
        }[variant]
        body = body[:old.start()] + "return " + replacement + ";" + body[old.end():]
    return text[:start] + body + text[end:]
