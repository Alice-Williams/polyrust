"""Manual Linux cache gate; mutate only a temporary extracted checkpoint."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

from constant_export_cache_proof import stream

PREFIX = "//experiments/rustc-frontend:"
METADATA = ["character_" + owner + "_metadata" for owner in ["leaf", "middle", "root"]]
BUNDLES = ["generated_" + language + "_character_" + owner
           for language in ["c", "java"] for owner in ["middle", "root"]]
TARGETS = [PREFIX + name for name in METADATA + BUNDLES]
NATIVE = PREFIX + "character_source_native_test"


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
    with tempfile.TemporaryDirectory(prefix="polyrust-character-cache-") as temporary:
        work = Path(temporary)
        with tarfile.open(archive) as source:
            source.extractall(work, filter="data")
        fixture = work / "experiments/rustc-frontend/fixtures/character_leaf.rs"
        oracle = work / "experiments/rustc-frontend/test/character_source_truth.py"
        original, original_truth = fixture.read_bytes(), oracle.read_bytes()
        old = b"pub fn maximum() -> char {\n        '\\u{10ffff}'\n    }"
        new = b"pub fn maximum() -> char {\n        '\\u{10fffe}'\n    }"
        old_truth = b'LITERAL_PREFIX = b"".join(struct.pack("<I", value) for value in BOUNDARIES)'
        new_truth = b'LITERAL_PREFIX = b"".join(struct.pack("<I", value) for value in (*BOUNDARIES[:-1], 0x10fffe))'
        assert original.count(old) == original_truth.count(old_truth) == 1

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
            evidence.append(dict(label=label, exit_code=result.returncode, tests=summaries, actions=records))
            (receipts / "summary.json").write_text(json.dumps(evidence, indent=2) + "\n")
            print(json.dumps(dict(label=label, exit_code=result.returncode, actions=len(records))), flush=True)
            return records

        bazel("baseline", "build", TARGETS)
        baseline = snapshot(work)
        assert len([name for name in baseline if name.endswith(".rmeta")]) == 3
        bazel("baseline-test", "test", [NATIVE])
        bazel("warm-test", "test", [NATIVE], cached=True)
        warm = bazel("warm-build", "build", TARGETS, actions=True)
        assert not {item["target"] for item in warm} & set(TARGETS)
        try:
            fixture.write_bytes(original.replace(old, new))
            actions = bazel("changed-character", "build", TARGETS, actions=True)
            labels = {item["target"] for item in actions}
            assert set(TARGETS) <= labels, labels
            assert PREFIX + "adapter" not in labels and PREFIX + "java_graph_adapter" not in labels
            changed = snapshot(work)
            assert changed.keys() == baseline.keys()
            for name in METADATA:
                assert changed["lib" + name + ".rmeta"] != baseline["lib" + name + ".rmeta"]
            for bundle in BUNDLES:
                assert any(changed[name] != baseline[name] for name in baseline if name.startswith(bundle + ".bundle/"))
            (receipts / "hashes.json").write_text(json.dumps(dict(baseline=baseline, changed=changed), indent=2) + "\n")
            bazel("old-truth", "test", [NATIVE], accepted=False, cached=False)
            oracle.write_bytes(original_truth.replace(old_truth, new_truth))
            bazel("new-truth", "test", [NATIVE])
            assert snapshot(work) == changed
        finally:
            fixture.write_bytes(original)
            oracle.write_bytes(original_truth)
        bazel("restored", "build", TARGETS)
        assert snapshot(work) == baseline
        bazel("restored-test", "test", [NATIVE], cached=True)
        print("Changed source character invalidated all affected metadata/packages; "
              "old truth rejected, updated original/C/Java observations passed; "
              "restored bytes and cached passing test were reused.")


if __name__ == "__main__":
    main()
