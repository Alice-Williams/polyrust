"""Independent exact Python-integer oracle; never round through floats."""
VALUES = [-(1 << 63), -(1 << 63) + 1, -(1 << 53) - 1, -(1 << 53),
          -(1 << 32), -(1 << 31) - 1, -(1 << 31), -(1 << 31) + 1, -1,
          0, 1, (1 << 31) - 2, (1 << 31) - 1, 1 << 31, (1 << 32) - 1,
          1 << 32, (1 << 53) - 1, 1 << 53, (1 << 53) + 1,
          (1 << 63) - 2, (1 << 63) - 1]
WIDE = ["identity", "choose", "minimum", "maximum", "positive", "negative", "local",
        "record", "shared", "shared_record", "imported", "mixed"]
BOOL = ["equal", "not_equal", "less", "less_equal", "greater", "greater_equal", "lazy"]


def expected():
    output = []
    for a in VALUES:
        for b in VALUES:
            for flag in [False, True]:
                chosen = b if flag else a
                wide = [a, b if (a < b) == flag else a, -(1 << 63), (1 << 63) - 1, (1 << 53) + 1,
                        -(1 << 53) - 1, a, chosen, a, chosen, chosen,
                        a if flag or a == b else b]
                boolean = [a == b, a != b, a < b, a <= b, a > b, a >= b,
                           (a < b and flag) or a == b]
                output.extend(str(value) for value in wide)
                output.extend(str(int(value)) for value in boolean)
                output.append(str(a))
    assert len(output) == 21 * 21 * 2 * 20
    return "\n".join(output) + "\n"


def expected_traces():
    traces = []
    for a in VALUES:
        for b in VALUES:
            for flag in [False, True]:
                traces.extend("LR" if name == "choose" else "" for name in WIDE)
                traces.extend(["LRC"] * 6)
                traces.append("LR" if a < b and flag else "LRLR")
                traces.append("")  # Public alias of the uninstrumented identity.
    assert len(traces) == 17640
    return "\n".join(traces) + "\n"
