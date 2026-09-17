"""Exact Java21 arithmetic and independent call-order fault observations."""
from pathlib import Path
import subprocess
import sys

root, javac, java, oracle = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(oracle))
from arithmetic_oracle import MAGNITUDE, Operation as Op, rational, encode, result, observed
from arithmetic_cases import PAIRS
from short_circuit_mutations import instrument
from arithmetic_mutations import mutate

FMA_PAIR = (0x3ff0000000000000 + (1 << 25), 0x3ff0000000000000 - (1 << 26))
PAIRS = PAIRS + [FMA_PAIR]
NEGATIVE_ONE = 0xbff0000000000000


def expected(variant):
    values = []
    for left, right in PAIRS:
        basic = [result(left, right, operation) for operation in Op]
        wanted = basic + [result(basic[0], right, Op.MULTIPLY),
                          result(basic[2], NEGATIVE_ONE, Op.ADD)]
        if variant == "operator":
            wanted[0] = result(left, right, Op.SUBTRACT)
        elif variant == "swapped":
            wanted[1] = result(right, left, Op.SUBTRACT)
        elif variant == "zero_sign" and wanted[0] & MAGNITUDE == 0:
            wanted[0] = 0
        elif variant == "grouping":
            wanted[4] = result(left, result(right, right, Op.MULTIPLY), Op.ADD)
        elif variant == "fused" and (left, right) == FMA_PAIR:
            wanted[5] = encode(rational(left) * rational(right) - 1)
        values.extend(observed(bits) for bits in wanted * 2)
    return "\n".join(values) + "\n"


def trace(variant):
    if variant == "plain":
        return ""
    first = {"dropped": "B", "duplicated": "AAB", "reversed": "BA"}.get(variant, "AB")
    return ((first + "AB" * 5) * 2) * len(PAIRS)


def run(command, inputs=None):
    output = subprocess.run([str(arg) for arg in command], input=inputs,
                            capture_output=True, text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    return output


consumer = """public final class Consumer {
private Consumer() {}
private static void observe(double value) {
    if (Double.isNaN(value)) { System.out.println("nan"); }
    else { System.out.printf(java.util.Locale.ROOT, "%016x%n", Double.doubleToRawLongBits(value)); }
}
public static void main(String[] args) throws java.io.IOException {
    java.io.BufferedReader reader = new java.io.BufferedReader(
        new java.io.InputStreamReader(System.in, java.nio.charset.StandardCharsets.UTF_8));
    String line;
    while ((line = reader.readLine()) != null) {
        String[] parts = line.split(" ");
        double left = Double.longBitsToDouble(Long.parseUnsignedLong(parts[0], 16));
        double right = Double.longBitsToDouble(Long.parseUnsignedLong(parts[1], 16));
""" + "".join(f"observe(org.polyrust.generated.r{owner:016x}.Generated.arithmetic{index}(left, right));\n"
              for owner in [502, 503] for index in range(6)) + "} } }\n"


def main():
    assert run([javac, "-version"]).stdout.startswith("javac 21.")
    truth = expected("plain")
    inputs = "".join(f"{left:016x} {right:016x}\n" for left, right in PAIRS)
    for variant in ["plain", "traced", "operator", "swapped", "zero_sign",
                    "grouping", "fused", "dropped", "duplicated", "reversed"]:
        directory = root / variant
        classes = directory / "classes"
        classes.mkdir(parents=True)
        for index in range(3):
            originals = list((root / f"owner{index}").rglob("*.java"))
            assert len(originals) == 1
            original = originals[0]
            text = original.read_text()
            assert "Math." not in text and "Runtime" not in text
            if variant != "plain" and index == 0:
                text = instrument(text, {"A": "leftIdentity", "B": "rightIdentity"}, True)
            if variant not in ("plain", "traced") and index == 1:
                text = mutate(text, variant)
            path = directory / f"owner{index}" / original.relative_to(root / f"owner{index}")
            path.parent.mkdir(parents=True)
            path.write_text(text)
            compile_source(path, classes)
        client = directory / "Consumer.java"
        client.write_text(consumer)
        compile_source(client, classes)
        wanted = expected(variant)
        if variant in ("operator", "swapped", "zero_sign", "grouping", "fused"):
            assert wanted != truth, variant
        else:
            assert wanted == truth
        if variant in ("dropped", "duplicated", "reversed"):
            assert trace(variant) != trace("traced")
        output = run([java, "-cp", classes, "Consumer"], inputs)
        if output.stdout != wanted:
            actual, target = output.stdout.splitlines(), wanted.splitlines()
            mismatch = next((i for i, pair in enumerate(zip(actual, target)) if pair[0] != pair[1]),
                            min(len(actual), len(target)))
            raise AssertionError((variant, mismatch, PAIRS[mismatch // 12],
                                  actual[mismatch:mismatch + 1], target[mismatch:mismatch + 1]))
        assert output.stderr == trace(variant), (variant, "trace")
    print(f"{len(PAIRS) * 12} exact Java21 results/run; separate original/importing packages; "
          "eight compiling value/trace faults")


def compile_source(source, classes):
    run([javac, "--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes, source])


if __name__ == "__main__":
    main()
