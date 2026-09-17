"""Compiling faults of fixed generated fixture copies, never production output."""
import re
from short_circuit_mutations import definition


def mutate(text, member, left, right, java, variant):
    match, start, end = definition(text, member)
    parameters = [re.search(r"\b(\w+)\s*$", value).group(1)
                  for value in match.group(1).split(",")]
    assert len(parameters) == 2
    a, b = parameters
    body = text[start:end]
    if variant in ["dropped", "duplicated", "reversed"]:
        callee = right if variant == "reversed" else left
        pattern = r"(?:\b\w+\.)*\b" + re.escape(callee.split(".")[-1]) + r"\((\w+)\)"
        found = list(re.finditer(pattern, body))
        assert len(found) == 1, (variant, callee, body)
        call = found[0]
        if variant == "dropped":
            body = body[:call.start()] + call.group(1) + body[call.end():]
        elif variant == "duplicated":
            line = body.rfind("\n", 0, call.start()) + 1
            prefix = "" if java else "(void)"
            body = body[:line] + prefix + call.group(0) + ";\n" + body[line:]
        else:
            body = body[:call.start()] + "poly_fault_right" + body[call.end():]
            body = f"\ndouble poly_fault_right = {right}({b});\n" + body
    else:
        returns = list(re.finditer(r"return\s+([^;]+);", body))
        assert len(returns) == 1, (variant, body)
        old = returns[0]
        fused = "java.lang.Math.fma" if java else "fma"
        replacement = {
            "operator": f"{a} - {b}",
            "swapped": f"{b} - {a}",
            "zero_sign": f"({old.group(1)}) == 0.0 ? 0.0 : ({old.group(1)})",
            "grouping": f"{a} + ({b} * {b})",
            "fused": (f"({a} == 0x1.0000002p0 && {b} == 0x1.ffffffcp-1)"
                      f" ? {fused}({a}, {b}, -1.0) : ({old.group(1)})"),
            "division_grouping": f"({a} / {b}) / {a}",
        }[variant]
        body = body[:old.start()] + "return " + replacement + ";" + body[old.end():]
    if not java:
        # Retain pure discarded materializations under strict unused-local lint.
        locals_ = re.findall(r"\bdouble\s+(\w+)\s*=", body)
        position = body.index("return ")
        body = body[:position] + "".join(f"(void){local};\n" for local in locals_) + body[position:]
    changed = text[:start] + body + text[end:]
    return "#include <math.h>\n" + changed if variant == "fused" and not java else changed
