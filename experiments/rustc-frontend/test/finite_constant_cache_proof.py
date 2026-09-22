"""Manual Bazel invalidation gate; mutations stay inside a temporary Git archive."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

from constant_export_cache_proof import stream

PREFIX = "//experiments/rustc-frontend:"
OWNERS = ["constants", "second", "middle", "root"]
BUNDLES = ["generated_" + lang + "_finite_constant_" + owner
           for lang in ["c", "java"] for owner in ["middle", "root"]]
METADATA = ["finite_constant_" + owner + "_metadata" for owner in OWNERS]
TARGETS = [PREFIX + name for name in BUNDLES + METADATA]
NATIVE = PREFIX + "finite_constant_source_native_test"


def snapshot(work):
    generated = work / "bazel-bin/experiments/rustc-frontend"
    paths = [generated / ("lib" + name + ".rmeta") for name in METADATA]
    paths += [p for bundle in BUNDLES for p in (generated / (bundle + ".bundle")).rglob("*") if p.is_file()]
    return {p.relative_to(generated).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}


def main():
    archive = Path(sys.argv[1]).resolve(strict=True)
    receipts = Path(sys.argv[2]).resolve()
    receipts.mkdir(parents=True, exist_ok=False)
    evidence = []
    with tempfile.TemporaryDirectory(prefix="polyrust-finite-constant-cache-") as directory:
        work = Path(directory)
        with tarfile.open(archive) as source:
            source.extractall(work, filter="data")
        fixture = work / "experiments/rustc-frontend/fixtures/finite_constant_data.rs"
        oracle = work / "experiments/rustc-frontend/test/finite_constant_source_truth.py"
        original, original_oracle = fixture.read_bytes(), oracle.read_bytes()
        old_source = b"pub const POSITIVE_ZERO: f64 = 0.0;"
        old_truth = b'"POSITIVE_ZERO": 0,'
        assert original.count(old_source) == original_oracle.count(old_truth) == 1

        def bazel(label, command, targets, accepted=True, actions=False, cached=None):
            event, action_log = receipts / (label + ".jsonl"), receipts / (label + "-actions.json")
            result = subprocess.run([
                "bazelisk", "--batch", "--output_user_root=/tmp/polyrust-m34a10w-bazel",
                command, *targets, "--jobs=2", "--lockfile_mode=off", "--noshow_progress",
                "--noverbose_failures", "--build_event_json_file=" + str(event),
                *(["--execution_log_json_file=" + str(action_log)] if actions else []),
            ], cwd=work, capture_output=True, text=True, timeout=1200)
            (receipts / (label + ".stdout")).write_text(result.stdout)
            (receipts / (label + ".stderr")).write_text(result.stderr)
            assert (result.returncode == 0) == accepted, (label, result.stdout, result.stderr)
            events = [json.loads(line) for line in event.read_text().splitlines()]
            summaries = [entry["testSummary"] for entry in events if "testSummary" in entry]
            records = [dict(target=item.get("targetLabel"), mnemonic=item.get("mnemonic"),
                            cache_hit=item.get("cacheHit", False), runner=item.get("runner"))
                       for item in stream(action_log.read_text() if actions else "")]
            if command == "test":
                assert len(summaries) == 1 and summaries[0]["overallStatus"] == ("PASSED" if accepted else "FAILED")
                if cached is not None:
                    assert (summaries[0].get("totalNumCached", 0) > 0) == cached, summaries
            entry = dict(label=label, exit_code=result.returncode, tests=summaries, actions=records)
            evidence.append(entry)
            (receipts / "summary.json").write_text(json.dumps(evidence, indent=2) + "\n")
            print(json.dumps(dict(label=label, exit_code=result.returncode, actions=len(records))), flush=True)
            return records

        bazel("baseline", "build", TARGETS)
        baseline = snapshot(work)
        assert len([name for name in baseline if name.endswith(".rmeta")]) == 4
        bazel("baseline-test", "test", [NATIVE])
        bazel("warm-test", "test", [NATIVE], cached=True)
        warm = bazel("warm-build", "build", TARGETS, actions=True)
        assert not {item["target"] for item in warm} & set(TARGETS)
        for label, expression, bits in [("finite-value", b"1.0", 0x3ff0000000000000),
                                        ("zero-sign", b"-0.0", 0x8000000000000000)]:
            try:
                fixture.write_bytes(original.replace(old_source, b"pub const POSITIVE_ZERO: f64 = " + expression + b";"))
                actions = bazel(label, "build", TARGETS, actions=True)
                labels = {item["target"] for item in actions}
                independent = PREFIX + "finite_constant_second_metadata"
                assert set(TARGETS) - {independent} <= labels, (label, labels)
                assert independent not in labels
                changed = snapshot(work)
                assert changed.keys() == baseline.keys()
                for owner in OWNERS:
                    name = "libfinite_constant_" + owner + "_metadata.rmeta"
                    assert (changed[name] != baseline[name]) == (owner != "second"), (label, name)
                for bundle in BUNDLES:
                    assert any(changed[name] != baseline[name] for name in baseline if name.startswith(bundle + ".bundle/"))
                (receipts / (label + "-hashes.json")).write_text(json.dumps(dict(baseline=baseline, changed=changed), indent=2) + "\n")
                bazel(label + "-old-truth", "test", [NATIVE], accepted=False, cached=False)
                oracle.write_bytes(original_oracle.replace(old_truth, f'"POSITIVE_ZERO": 0x{bits:016x},'.encode()))
                bazel(label + "-new-truth", "test", [NATIVE])
                assert snapshot(work) == changed
            finally:
                fixture.write_bytes(original)
                oracle.write_bytes(original_oracle)
            bazel(label + "-restored", "build", TARGETS)
            assert snapshot(work) == baseline
            bazel(label + "-restored-test", "test", [NATIVE], cached=True)
        print("Finite value and zero sign: affected metadata/bundles invalidated, independent producer retained, "
              "old truth rejected, new truth observed natively, restored cached baseline reused.")


if __name__ == "__main__":
    main()
