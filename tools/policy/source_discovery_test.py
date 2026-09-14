"""Prove policy harnesses reject empty and partially failed source discovery."""

import os
from pathlib import Path
import subprocess
import tempfile


def main() -> None:
    root = Path(os.environ["RUNFILES_DIR"]) / os.environ["TEST_WORKSPACE"]
    with tempfile.TemporaryDirectory(dir=os.environ["TEST_TMPDIR"]) as temporary:
        work = Path(temporary)
        commands = work / "commands"
        commands.mkdir()
        python_marker = work / "policy-was-invoked"
        fake_python = commands / "python3"
        fake_python.write_text(
            '#!/bin/sh\nprintf "%s\\0" "$@" > "$POLICY_MARKER"\nexit 0\n', encoding="utf-8"
        )
        fake_python.chmod(0o755)
        env = {
            **os.environ,
            "PATH": f"{commands}:{os.environ['PATH']}",
            "POLICY_MARKER": str(python_marker),
            "TEST_TMPDIR": str(work),
        }
        for harness in ["template_policy_test.sh", "typed_generation_source_policy_test.sh"]:
            for output, exit_code, expected_success in [
                (True, 1, False),
                (False, 0, False),
                (True, 0, True),
            ]:
                if python_marker.exists():
                    python_marker.unlink()
                fake_find = commands / "find"
                fake_find.write_text(
                    "#!/bin/sh\n"
                    + ('printf "%s\\0" "$2/example.rs"\n' if output else "")
                    + f"exit {exit_code}\n",
                    encoding="utf-8",
                )
                fake_find.chmod(0o755)
                result = subprocess.run(
                    ["bash", str(root / "tools/policy" / harness)],
                    env=env,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                assert (result.returncode == 0) == expected_success, (
                    harness, output, exit_code, result.stdout, result.stderr
                )
                assert python_marker.exists() == expected_success, (
                    "policy must not run over an incomplete discovery result", harness
                )
                if expected_success:
                    arguments = python_marker.read_bytes().split(b"\0")
                    expected = (
                        "crates/codegen/src/linking/example.rs"
                        if harness == "typed_generation_source_policy_test.sh"
                        else "crates/codegen/src/import_rendering.rs"
                    )
                    assert any(argument.endswith(expected.encode()) for argument in arguments), (
                        "new shared modules must actually reach the verifier", harness, arguments
                    )
        # A successful Java inventory cannot conceal an empty/failed second
        # discovery of the shared linker. Both must finish before verification.
        for partial, exit_code in [(False, 0), (True, 1)]:
            python_marker.unlink(missing_ok=True)
            fake_find.write_text(
                '#!/bin/sh\ncase "$2" in\n*/codegen/src/linking)\n'
                + ('printf "%s\\0" "$2/partial.rs"\n' if partial else "")
                + f'exit {exit_code};;\nesac\nprintf "%s\\0" "$2/example.rs"\n',
                encoding="utf-8",
            )
            result = subprocess.run(
                ["bash", str(root / "tools/policy/typed_generation_source_policy_test.sh")],
                env=env, capture_output=True, text=True, check=False,
            )
            assert result.returncode != 0, (partial, exit_code, result.stdout, result.stderr)
            assert not python_marker.exists(), "partial shared inventory must not reach policy"
    print("policy source discovery fails closed")


if __name__ == "__main__":
    main()
