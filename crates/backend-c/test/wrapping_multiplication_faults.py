"""Disposable native mutations; never part of a generated production package."""
import re
import shutil


def prepare(root, stem):
    for variant in ["narrow", "saturating"]:
        directory = root / variant
        shutil.copytree(root / "valid", directory)
        producer = directory / (stem + ".c")
        text = producer.read_text()
        helpers = []
        for width in [32, 64]:
            pattern = rf"(\buint{width}_t \w+\s*=\s*)([^;\n]+);"
            if variant == "narrow":
                mask = (1 << (width // 2)) - 1
                sign = 1 << (width // 2 - 1)

                def mutate(match):
                    return (match[1] + f"((({match[2]}) & UINT{width}_C({mask})) "
                            f"^ UINT{width}_C({sign})) - UINT{width}_C({sign});")
            else:
                # These two locals are the actual typed fixture's parameter copies.
                operands = re.findall(rf"\bint{width}_t (\w+)\s*=", text)
                assert len(operands) == 2, (width, operands)

                def mutate(match):
                    return (match[1] + f"(uint{width}_t)fault_saturating{width}"
                            f"({operands[0]}, {operands[1]});")

                helpers.append(SATURATING.replace("WIDTH", str(width)))
            text, count = re.subn(pattern, mutate, text)
            assert count == 1, (width, "one original unsigned result")
        if helpers:
            # The original own-header include provides the exact-width types/macros.
            include = re.search(r'^#include "[^"\n]+"\n', text, flags=re.MULTILINE)
            assert include is not None
            text = text[:include.end()] + "".join(helpers) + text[include.end():]
        producer.write_text(text)


# Deliberately wrong Rust semantics, but defined C even at MIN * -1. Division
# guards establish signed representability before the final multiplication.
SATURATING = """
static intWIDTH_t fault_saturatingWIDTH(intWIDTH_t a, intWIDTH_t b) {
    if (a == 0 || b == 0) { return 0; }
    if (a > 0) {
        if (b > 0 && a > INTWIDTH_MAX / b) { return INTWIDTH_MAX; }
        if (b < 0 && b < INTWIDTH_MIN / a) { return INTWIDTH_MIN; }
    } else {
        if (b > 0 && a < INTWIDTH_MIN / b) { return INTWIDTH_MIN; }
        if (b < 0 && a < INTWIDTH_MAX / b) { return INTWIDTH_MAX; }
    }
    return a * b;
}
"""
