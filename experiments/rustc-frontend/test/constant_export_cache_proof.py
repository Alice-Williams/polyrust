"""Manual Linux Bazel gate: mutate only an extracted Git archive, keep receipts."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import time

PREFIX = "//experiments/rustc-frontend:"
OWNERS = ["constants", "second", "middle", "root"]
BUNDLES = ["generated_" + language + "_constant_export_" + owner
           for language in ["c", "java"] for owner in ["middle", "root"]]
METADATA = ["constant_export_" + owner + "_metadata" for owner in OWNERS]
TARGETS = [PREFIX + name for name in BUNDLES + METADATA]
NATIVE = PREFIX + "constant_export_native_test"


def stream(text):
    """Bazel's JSON execution log is a stream of objects, not necessarily an array."""
    decoder = json.JSONDecoder()
    position = 0
    while position < len(text):
        if text[position].isspace():
            position += 1
            continue
        value, position = decoder.raw_decode(text, position)
        yield from value if isinstance(value, list) else [value]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot(work):
    generated = work / "bazel-bin/experiments/rustc-frontend"
    outputs = [generated / ("lib" + name + ".rmeta") for name in METADATA]
    for bundle in BUNDLES:
        outputs += sorted(path for path in (generated / (bundle + ".bundle")).rglob("*")
                          if path.is_file())
    return {path.relative_to(generated).as_posix(): digest(path) for path in outputs}


def main():
    archive = Path(sys.argv[1]).resolve(strict=True)
    receipts = Path(sys.argv[2]).resolve()
    receipts.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory(prefix="polyrust-alias-cache-proof-") as directory:
        work = Path(directory)
        with tarfile.open(archive) as source:
            source.extractall(work, filter="data")
        fixture = work / "experiments/rustc-frontend/fixtures/public_constant_data.rs"
        oracle = work / "experiments/rustc-frontend/test/constant_export_native.py"
        original, original_oracle = fixture.read_bytes(), oracle.read_bytes()
        assert original.count(b"7 * 9 - 1") == 1
        changed = original.replace(b"7 * 9 - 1", b"17")
        old_truth = b"-9007199254740993, 62, -17, 62, 42, 62]"
        new_truth = b"-9007199254740993, 17, -17, 17, 42, 17]"
        assert original_oracle.count(old_truth) == 1
        evidence = []

        def bazel(label, command, targets, accepted=True):
            started = time.monotonic()
            event, log = receipts / (label + ".jsonl"), receipts / (label + "-actions.json")
            # Only these phases need action evidence. A cold-baseline toolchain
            # execution log can otherwise retain hundreds of megabytes of inputs.
            action_flags = ["--execution_log_json_file=" + str(log)] if label in ["warm-build", "changed"] else []
            result = subprocess.run([
                "bazelisk", "--output_user_root=/tmp/polyrust-m34a10w-bazel", "--batch",
                command, *targets, "--noshow_progress", "--noverbose_failures",
                "--build_event_json_file=" + str(event), *action_flags,
            ], cwd=work, text=True, capture_output=True, timeout=900)
            (receipts / (label + ".stdout")).write_text(result.stdout)
            (receipts / (label + ".stderr")).write_text(result.stderr)
            assert (result.returncode == 0) == accepted, (label, result.stdout, result.stderr)
            events = [json.loads(line) for line in event.read_text().splitlines()]
            summaries = [item["testSummary"] for item in events if "testSummary" in item]
            actions = [dict(target=item.get("targetLabel"), mnemonic=item.get("mnemonic"),
                            cache_hit=item.get("cacheHit", False), runner=item.get("runner"))
                       for item in stream(log.read_text() if log.exists() else "")]
            entry = dict(label=label, seconds=round(time.monotonic() - started, 3),
                         exit_code=result.returncode, test_summaries=summaries, actions=actions)
            evidence.append(entry)
            (receipts / "summary.json").write_text(json.dumps(evidence, indent=2) + "\n")
            print(json.dumps({**entry, "actions": len(actions)}), flush=True)
            return summaries, actions

        def test(label, accepted=True, cached=None):
            summaries, actions = bazel(label, "test", [NATIVE], accepted)
            assert len(summaries) == 1
            assert summaries[0]["overallStatus"] == ("PASSED" if accepted else "FAILED")
            if cached is not None:
                assert (summaries[0].get("totalNumCached", 0) > 0) == cached, summaries
            return actions

        bazel("baseline", "build", TARGETS)
        baseline = snapshot(work)
        # A fresh workspace may legitimately fetch the baseline test from disk cache.
        summaries, _ = bazel("baseline-test", "test", [NATIVE])
        assert len(summaries) == 1 and summaries[0]["overallStatus"] == "PASSED"
        test("warm-test", cached=True)
        _, warm = bazel("warm-build", "build", TARGETS)
        assert not {item["target"] for item in warm} & set(TARGETS), warm
        assert snapshot(work) == baseline
        try:
            fixture.write_bytes(changed)
            _, actions = bazel("changed", "build", TARGETS)
            labels = {item["target"] for item in actions}
            affected = {PREFIX + name for name in BUNDLES + METADATA
                        if name != "constant_export_second_metadata"}
            assert affected <= labels, ("affected actions not invalidated", affected - labels, actions)
            assert PREFIX + "constant_export_second_metadata" not in labels, actions
            mutated = snapshot(work)
            assert baseline.keys() == mutated.keys()
            for owner in OWNERS:
                name = "libconstant_export_" + owner + "_metadata.rmeta"
                assert (baseline[name] != mutated[name]) == (owner != "second"), name
            for bundle in BUNDLES:
                assert any(baseline[name] != mutated[name] for name in baseline
                           if name.startswith(bundle + ".bundle/")), bundle
            (receipts / "output-hashes.json").write_text(
                json.dumps(dict(baseline=baseline, changed=mutated), indent=2) + "\n")
            test("changed-old-truth", accepted=False, cached=False)
            oracle.write_bytes(original_oracle.replace(old_truth, new_truth))
            test("changed-new-truth")
            assert snapshot(work) == mutated
        finally:
            fixture.write_bytes(original)
            oracle.write_bytes(original_oracle)
        bazel("restored", "build", TARGETS)
        assert snapshot(work) == baseline
        test("restored-test", cached=True)
        print("Alias cache proof passed: affected metadata/bundles invalidated; independent "
              "producer cached; old truth rejected; updated truth passed; original cache restored.")


if __name__ == "__main__":
    main()
