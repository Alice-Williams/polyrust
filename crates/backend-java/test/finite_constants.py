"""Independent bits, strict separate Java21 compilation, real constant mutations."""
from pathlib import Path
import re
import shutil
import subprocess
import sys

sys.path.insert(0, str(Path(sys.argv[-1]).resolve()))
from finite_constant_oracle import expected_values
from finite_constant_faults import faulty

VALUES = expected_values()
CONTROLS = [0, 1 << 63, 1, 0x0010000000000000, 0x3ff0000000000001,
            0x3fb999999999999a, 0x7fefffffffffffff, 0xffefffffffffffff]
OWNER = "org.polyrust.generated.r000000000000035c.Generated"
READER = "org.polyrust.generated.r0000000000000924.Generated"


def run(command):
    output = subprocess.run([str(arg) for arg in command], capture_output=True,
                            text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    assert output.stderr == "", (command, output.stderr)
    return output.stdout


def source(root, owner):
    return root / "src/main/java" / (owner.replace(".", "/") + ".java")


def client(count, readers):
    expressions = [f"{OWNER}.constant{index}" for index in range(count)]
    if readers:
        expressions += [f"{OWNER}.read{index}()" for index in range(count)]
        expressions += [f"{READER}.read{index}()" for index in range(count)]
    statements = "\n".join('System.out.printf("%016x%n", Double.doubleToRawLongBits(' + value + '));'
                           for value in expressions)
    return "public final class Consumer { public static void main(String[] args) {\n" + statements + "\n} }\n"


def literal(bits):
    sign = "-" if bits >> 63 else ""
    exponent, fraction = (bits >> 52) & 2047, bits & ((1 << 52) - 1)
    if exponent == 2047:
        assert fraction == 0
        return sign + "Double.POSITIVE_INFINITY"
    return f"{sign}0x{int(exponent != 0):x}.{fraction:013x}p{(exponent or 1) - 1023:+d}"


def mutate(root, variant):
    path = source(root, OWNER)
    original = path.read_text()
    def replace(match):
        return match[1] + literal(faulty(CONTROLS[int(match[2])], variant)) + ";"
    changed, count = re.subn(r"(\bconstant(\d+)\s*=\s*)[^;]+;", replace, original)
    assert count == len(CONTROLS) and changed != original
    path.write_text(changed)


def observe(root, values, javac, java, readers=False):
    classes = root / "classes"
    classes.mkdir()
    flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
             "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
    run([javac, *flags, source(root, OWNER)])
    # Recompile all dependents AFTER each producer mutation: javac inlines fields.
    if readers:
        run([javac, *flags, source(root, READER)])
    driver = root / "Consumer.java"
    driver.write_text(client(len(values), readers))
    run([javac, *flags, driver])
    expected = "".join(f"{value:016x}\n" for value in values) * (3 if readers else 1)
    for options in [[], ["-Xint"]]:
        assert run([java, *options, "-cp", classes, "Consumer"]) == expected, (root, options)


def main():
    if sys.argv[1] == "inputs":
        print("\n".join(f"{value:016x}" for value in VALUES))
        return
    root, javac, java = [Path(arg).resolve() for arg in sys.argv[1:4]]
    assert run([javac, "-version"]).startswith("javac 21.")
    for index, start in enumerate(range(0, len(VALUES), 256)):
        observe(root / str(index), VALUES[start:start + 256], javac, java)
    controls = root / "controls"
    originals = {path.relative_to(controls): path.read_bytes() for path in controls.rglob("*.java")}
    assert len(originals) == 2
    for variant in ["valid", "zero_sign", "f32", "wrong_value"]:
        directory = root / variant
        shutil.copytree(controls, directory)
        expected = CONTROLS if variant == "valid" else [faulty(value, variant) for value in CONTROLS]
        assert (expected == CONTROLS) == (variant == "valid")
        if variant != "valid":
            mutate(directory, variant)
        observe(directory, expected, javac, java, readers=True)
    assert originals == {path.relative_to(controls): path.read_bytes() for path in controls.rglob("*.java")}
    print(f"{len(VALUES)} certified finite fields; strict separate Java21 compilation, normal/-Xint; "
          "owned/imported-reader controls; three compiling faults detected after dependent recompilation")


if __name__ == "__main__":
    main()
