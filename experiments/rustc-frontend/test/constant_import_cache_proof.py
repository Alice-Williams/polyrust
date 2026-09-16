"""Manual Bazel integration gate: mutate only a freshly extracted Git archive."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import time

PREFIX = "//experiments/rustc-frontend:"
TARGETS = [PREFIX + name for name in [
    "generated_c_constant_import", "generated_java_constant_import",
    "constant_import_constants_metadata", "constant_import_left_metadata",
    "constant_import_right_metadata", "constant_import_root_metadata",
]]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot(work):
    generated = work / "bazel-bin/experiments/rustc-frontend"
    return {p.relative_to(generated).as_posix(): digest(p)
            for p in generated.rglob("*") if p.is_file() and
            (p.suffix == ".rmeta" and p.name.startswith("libconstant_import_")
             or any(parent.name in ["generated_c_constant_import.bundle",
                                    "generated_java_constant_import.bundle"] for parent in p.parents))}


def main():
    archive = Path(sys.argv[1]).resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="polyrust-constant-cache-proof-") as directory:
        work = Path(directory)
        with tarfile.open(archive) as source:
            source.extractall(work, filter="data")
        fixture = work / "experiments/rustc-frontend/fixtures/public_constant_data.rs"
        original = fixture.read_bytes()
        changed = original.replace(b"7 * 9 - 1", b"17")
        assert original != changed
        evidence = []

        def bazel(label, command, targets, accepted):
            started = time.monotonic()
            event = work / (label + ".jsonl")
            result = subprocess.run([
                "bazelisk", "--output_user_root=/tmp/polyrust-m34a10w-bazel", "--batch",
                command, *targets, "--noshow_progress", "--noverbose_failures",
                "--build_event_json_file=" + str(event),
            ], cwd=work, text=True, capture_output=True, timeout=900)
            assert (result.returncode == 0) == accepted, (label, result.stdout, result.stderr)
            events = [json.loads(line) for line in event.read_text().splitlines()]
            summary = [e["testSummary"] for e in events if "testSummary" in e]
            evidence.append(dict(label=label, seconds=round(time.monotonic() - started, 3),
                                 exit_code=result.returncode, test_summaries=summary))
            print(json.dumps(evidence[-1]), flush=True)
            return summary

        bazel("baseline", "build", TARGETS, True)
        baseline = snapshot(work)
        assert len([name for name in baseline if name.endswith(".rmeta")]) == 4
        bazel("baseline-test", "test", [PREFIX + "constant_import_native_test"], True)
        try:
            fixture.write_bytes(changed)
            bazel("changed", "build", TARGETS, True)
            mutated = snapshot(work)
            assert baseline.keys() == mutated.keys()
            for name in baseline:
                if name.endswith(".rmeta"):
                    assert baseline[name] != mutated[name], ("metadata was not invalidated", name)
            for bundle in ["generated_c_constant_import.bundle", "generated_java_constant_import.bundle"]:
                assert any(baseline[name] != mutated[name] for name in baseline if name.startswith(bundle + "/"))
            summaries = bazel("changed-test", "test", [PREFIX + "constant_import_native_test"], False)
            assert len(summaries) == 1 and summaries[0]["overallStatus"] == "FAILED"
        finally:
            fixture.write_bytes(original)
        bazel("restored", "build", TARGETS, True)
        assert snapshot(work) == baseline
        summaries = bazel("restored-test", "test", [PREFIX + "constant_import_native_test"], True)
        assert len(summaries) == 1 and summaries[0]["overallStatus"] == "PASSED"
        assert summaries[0].get("totalNumCached", 0) > 0, summaries
        print("Producer-source changes invalidate all four metadata artifacts and both generated bundles; "
              "the independent oracle rejects the mutation; restoring source reuses passing test cache.")


if __name__ == "__main__":
    main()
