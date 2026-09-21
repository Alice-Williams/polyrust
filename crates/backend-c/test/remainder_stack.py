"""Actual fmod/import-chain stack bounds, guarded arenas and sensitivity controls."""
from remainder_native import ROOT, ZIG, FLAGS, STEMS, LEFT, RIGHT, PAIRS, result, run
from pathlib import Path
import os
import subprocess


def main():
    cases = ",\n".join(f"{{UINT64_C({a}),UINT64_C({b}),UINT64_C({result(a,b)})}}" for a, b in PAIRS)
    template = Path(__file__).with_name("remainder_stack_probe.c").read_text()
    source = ROOT / "probe.c"
    source.write_text(template.replace("PROBE_CASES", cases))
    members = [LEFT, RIGHT, "poly_remainder_602_0", "poly_remainder_603_0"]
    bounds = {name: int(bound) for name, bound in (line.split() for line in (ROOT / "stack_bounds.txt").read_text().splitlines())}
    assert set(bounds) == set(members)
    measurements = []
    for index, compiler, sanitizer in [(0, "gcc-14", None), (1, ZIG, None),
                                       (2, "gcc-14", "address"), (3, "gcc-14", "undefined")]:
        for optimization in ["0", "2"]:
            directory = ROOT / f"stack{index}{optimization}"
            directory.mkdir()
            flags = [*FLAGS, "-O" + optimization, "-fno-inline", "-fno-optimize-sibling-calls",
                     "-fstack-usage", "-pthread", "-I", ROOT]
            sanitize = [] if sanitizer is None else ["-fsanitize=" + sanitizer, "-fno-sanitize-recover=all",
                                                      "-fno-pie", "-no-pie"]
            objects, frames = [], {}
            for stem in [*STEMS, "probe"]:
                obj, report = directory / (stem + ".o"), directory / (stem + ".su")
                stack_report = ["-Xclang", "-stack-usage-file", "-Xclang", report] if index == 1 else []
                run([compiler, *flags, *sanitize, *stack_report, "-c", ROOT / (stem + ".c"), "-o", obj])
                objects.append(obj)
                if stem != "probe":
                    for line in report.read_text().splitlines():
                        fields = line.split("\t")
                        member = fields[0].split(":")[-1]
                        if member in members:
                            assert fields[2] in ("static", "dynamic,bounded") and member not in frames
                            frames[member] = int(fields[1])
                        else:
                            assert member in ("_sub_I_00099_0", "_sub_I_00099_1", "_sub_D_00099_0", "_sub_D_00099_1"), member
            assert set(frames) == set(members)
            assert all(frames[member] <= bounds[member] for member in members)
            # Fresh whole-worker measurement includes generated frames and real libm.
            # Each generated frame is compared with the exported certificate bound above.
            binary = directory / "probe"
            run([compiler, *objects, *sanitize, "-pthread", "-lm", "-o", binary])

            def invoke(size, allowance, guard=None):
                command = [str(binary), str(size), str(allowance)]
                if guard:
                    command.append(guard)
                env = {**os.environ, "ASAN_OPTIONS": "detect_leaks=1:halt_on_error=1",
                       "UBSAN_OPTIONS": "halt_on_error=1"}
                return subprocess.run(command, capture_output=True, text=True, timeout=90, env=env)

            for size in [65536, 262144]:
                output = invoke(size, 65536)
                assert output.returncode == 0, (index, optimization, size, output.stderr)
                used = int(output.stdout.strip())
                assert 1 < used <= 65536
                measurements.append(used)
                print(f"fmod stack {index} O{optimization} arena={size}: watermark={used}, frames={frames}")
            assert invoke(262144, 1).returncode == 3
            for guard in ["lower", "upper"]:
                assert invoke(262144, 65536, guard).returncode != 0, guard
    print(f"Fresh fmod stack watermark {min(measurements)}..{max(measurements)} bytes; "
          "GCC/Zig O0/O2 and GCC ASan/UBSan; both guard faults and one-byte allowance rejected")


if __name__ == "__main__":
    main()
