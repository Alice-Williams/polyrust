"""Independent integer truth, certified Java Int fields, inlining-aware faults."""
from pathlib import Path
import re
import shutil
import subprocess
import sys

sys.path.insert(0, str(Path(sys.argv[-1]).resolve()))
from character_constant_oracle import BOUNDARIES, FAULTS, expected_values, faulty

TARGET_ONLY = (0xd800, 0xdfff, 0x110000, -1, -(1 << 31), (1 << 31) - 1)
VALUES = (*expected_values(), *TARGET_ONLY)
OWNER = "org.polyrust.generated.r000000000000035c.Generated"
FACADE = "org.polyrust.generated.r0000000000000925.Generated"
READER = "org.polyrust.generated.r0000000000000926.Generated"


def run(command):
    output = subprocess.run([str(arg) for arg in command], capture_output=True,
                            text=True, timeout=90)
    assert output.returncode == 0 and output.stderr == "", (command, output.stdout, output.stderr)
    return output.stdout


def source(root, owner):
    return root / "src/main/java" / (owner.replace(".", "/") + ".java")


def client(count):
    expressions = [f"{OWNER}.constant{index}" for index in range(count)]
    expressions += [f"{OWNER}.read{index}()" for index in range(count)]
    expressions += [f"{READER}.read{index}()" for index in range(count)]
    statements = "\n".join(f'System.out.printf("%08x%n", {value});' for value in expressions)
    return "public final class Consumer { public static void main(String[] args) {\n" + statements + "\n} }\n"


def observe(root, values, javac, java):
    classes = root / "classes"
    classes.mkdir()
    flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
             "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
    # Separate compilation, always rebuilding dependents after any producer edit.
    for owner in [OWNER, FACADE, READER]:
        run([javac, *flags, source(root, owner)])
    driver = root / "Consumer.java"
    driver.write_text(client(len(values)))
    run([javac, *flags, driver])
    expected = "".join(f"{value & 0xffffffff:08x}\n" for value in values) * 3
    for options in [[], ["-Xint"]]:
        assert run([java, *options, "-cp", classes, "Consumer"]) == expected, (root, options)


def mutate(root, fault):
    path = source(root, OWNER)
    original = path.read_text()
    def replace(match):
        return match[1] + str(faulty(BOUNDARIES[int(match[2])], fault)) + ";"
    changed, count = re.subn(r"(\bconstant(\d+)\s*=\s*)[^;]+;", replace, original)
    assert count == len(BOUNDARIES) and changed != original
    path.write_text(changed)


def readonly(root, javac):
    probe = root / "Readonly.java"
    probe.write_text("final class Readonly { void mutate() { " + OWNER + ".constant0 = 1; } }\n")
    result = subprocess.run([str(arg) for arg in [javac, "--release", "21", "-Xlint:all",
                            "-Werror", "-implicit:none", "-sourcepath", "", "-cp",
                            root / "classes", "-d", root / "classes", probe]],
                            capture_output=True, timeout=90)
    assert result.returncode != 0, "final field accepted assignment"
    assert b"final variable" in result.stderr, result.stderr


def main():
    assert len(VALUES) == 4133
    if sys.argv[1] in ("inputs", "controls"):
        print("\n".join(str(value) for value in (VALUES if sys.argv[1] == "inputs" else BOUNDARIES)))
        return
    root, javac, java = [Path(arg).resolve() for arg in sys.argv[1:4]]
    assert run([javac, "-version"]).startswith("javac 21.")
    for index, start in enumerate(range(0, len(VALUES), 64)):
        observe(root / str(index), VALUES[start:start + 64], javac, java)
    controls = root / "controls"
    originals = {path.relative_to(controls): path.read_bytes() for path in controls.rglob("*.java")}
    assert len(originals) == 3
    for fault in ["valid", *FAULTS]:
        directory = root / fault
        shutil.copytree(controls, directory)
        expected = list(BOUNDARIES) if fault == "valid" else [faulty(value, fault) for value in BOUNDARIES]
        assert (expected == list(BOUNDARIES)) == (fault == "valid")
        if fault != "valid":
            mutate(directory, fault)
        observe(directory, expected, javac, java)
    readonly(root / "valid", javac)
    assert originals == {path.relative_to(controls): path.read_bytes() for path in controls.rglob("*.java")}
    print("4133 certified Int fields, local readers and original-owner imports via aliases; "
          "strict separate Java21 compilation, normal/-Xint, readonly fields; "
          "four compiling faults detected after dependent recompilation")


if __name__ == "__main__":
    main()
