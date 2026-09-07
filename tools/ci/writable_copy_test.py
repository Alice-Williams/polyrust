"""Exercise native-test copy commands against read-only Bazel-like inputs."""

from pathlib import Path
import os
import shlex
import stat
import subprocess
import sys
import tempfile


def copy_commands(root: Path) -> list[list[str]]:
    scripts = list(root.glob("crates/backend-*/test_generated*.sh"))
    scripts += list(root.glob("examples/**/native_test.sh"))
    assert len(scripts) == 17, (
        "the complete affected native-script inventory is required"
    )
    commands = []
    for script in scripts:
        copies = [
            shlex.split(line)
            for line in script.read_text().splitlines()
            if line.startswith("cp ")
        ]
        assert copies, f"{script}: missing copy setup"
        for command in copies:
            assert "--no-preserve=mode" in command, (
                f"{script}: readonly modes would leak"
            )
            commands.append([word for word in command[1:] if word.startswith("-")])
    return commands


def verify_copy(flags: list[str]) -> None:
    with tempfile.TemporaryDirectory(prefix="polyrust-copy-proof-") as temporary:
        root = Path(temporary)
        root.chmod(0o755)
        source = root / "source"
        source.mkdir()
        original = source / "generated.txt"
        original.write_text("generated input\n")
        original.chmod(0o444)
        source.chmod(0o555)
        link = root / "runfile"
        link.symlink_to(source, target_is_directory=True)
        destination = root / "editable"
        destination.mkdir(mode=0o777)
        destination.chmod(0o777)
        identity = (
            dict(user=65534, group=65534, extra_groups=[]) if os.geteuid() == 0 else {}
        )
        recursive = any(
            "R" in flag or "r" in flag[1:]
            for flag in flags
            if flag.startswith("-") and not flag.startswith("--")
        )
        input_path = (
            str(link if "-RL" in flags else source) + "/."
            if recursive
            else str(original)
        )
        subprocess.run(
            ["cp", *flags, input_path, str(destination)], check=True, **identity
        )
        copied = destination / original.name
        assert copied.stat().st_mode & stat.S_IWUSR, (
            "copy must be owner-writable even under root tests"
        )
        subprocess.run(
            [
                sys.executable,
                "-c",
                "from pathlib import Path; import sys; Path(sys.argv[1]).write_text('formatted\\n')",
                str(copied),
            ],
            check=True,
            **identity,
        )
        assert original.read_text() == "generated input\n", "Bazel input was mutated"
        assert stat.S_IMODE(original.stat().st_mode) == 0o444
        source.chmod(0o755)


if __name__ == "__main__":
    commands = copy_commands(Path(sys.argv[1]))
    for flags in {tuple(command) for command in commands}:
        verify_copy(list(flags))
    print(f"Verified {len(commands)} native-test copy sites with read-only inputs.")
