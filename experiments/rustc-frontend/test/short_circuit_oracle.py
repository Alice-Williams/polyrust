"""Finite bool truth and call traces, independent of manifests and generators."""

NAMES = ["and", "or", "nested", "left_nested", "negated", "argument", "later_argument",
         "local", "shadow", "field", "shared", "comparison", "conditional", "constants"]


def expected():
    values, traces = [], []
    for a in [False, True]:
        for b in [False, True]:
            for c in [False, True]:
                trace = []

                def call(marker, value):
                    trace.append(marker)
                    return value

                first = lambda: call("A", a)
                second = lambda: call("B", b)
                third = lambda: call("C", c)
                operations = [
                    lambda: first() and second(),
                    lambda: first() or second(),
                    lambda: first() and (second() or third()),
                    lambda: (first() or second()) and third(),
                    lambda: not (first() and not second()) or third(),
                    lambda: (first() and second()) == third(),
                    lambda: first() == (second() and third()),
                    lambda: (first() and second()) or third(),
                    lambda: not (first() and second()) and third(),
                    lambda: (a and b) or c,
                    lambda: (a and b) or c,
                    lambda: (a == b) and (b != c),
                    lambda: third() if first() and second() else False,
                    lambda: third(),
                ]
                assert len(operations) == len(NAMES)
                for operation in operations:
                    trace.clear()
                    values.append(str(int(operation())))
                    traces.append("".join(trace))
    return "".join(value + "\n" for value in values), "".join(trace + "\n" for trace in traces)
