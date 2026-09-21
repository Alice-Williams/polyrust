#!/usr/bin/env python3
"""Boundary, deletion-safety and process-lifecycle tests; tiny synthetic caches."""

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import cache_budget
import storage_guard


class CacheBudgetTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve() / "cache"
        self.root.mkdir()

    def entry(self, number, age=1, area="cas"):
        digest = f"{number:064x}"
        path = self.root / area / digest[:2] / digest
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"cached artifact")
        os.utime(path, (1000 - age, 1000 - age))
        return path

    def test_evicts_oldest_to_budget_and_keeps_test_action_results(self):
        old = self.entry(1, age=100)
        newer = self.entry(2, area="ac")
        size = cache_budget.inventory(self.root)[0][2]
        total, removed = cache_budget.prune(self.root, budget=size, now=1000)
        self.assertEqual((total, removed), (size, size))
        self.assertFalse(old.exists())
        self.assertTrue(newer.exists())

    def test_age_eviction_even_when_under_size_budget(self):
        old = self.entry(1, age=100)
        recent = self.entry(2)
        cache_budget.prune(self.root, max_age=10, now=1000)
        self.assertFalse(old.exists())
        self.assertTrue(recent.exists())

    def test_no_deletion_needed(self):
        recent = self.entry(1)
        self.assertEqual(cache_budget.prune(self.root, now=1000)[1], 0)
        self.assertTrue(recent.exists())

    def test_unexpected_file_aborts_before_any_deletion(self):
        old = self.entry(1)
        (self.root / "source.rs").write_text("user source")
        with self.assertRaisesRegex(RuntimeError, "unexpected"):
            cache_budget.prune(self.root, budget=0)
        self.assertTrue(old.exists())

    def test_symlink_directory_aborts_before_any_deletion(self):
        old = self.entry(1)
        (self.root / "outside").symlink_to(self.root.parent, target_is_directory=True)
        with self.assertRaisesRegex(RuntimeError, "symlink"):
            cache_budget.prune(self.root, budget=0)
        self.assertTrue(old.exists())

    def test_symlink_root_is_rejected(self):
        link = self.root.parent / "linked"
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(RuntimeError, "unsafe"):
            cache_budget.prune(link)

    def test_temporary_files_are_not_deleted(self):
        temporary = self.root / "tmp" / "in-progress"
        temporary.parent.mkdir()
        temporary.write_bytes(b"keep")
        with self.assertRaisesRegex(RuntimeError, "temporary"):
            cache_budget.prune(self.root, budget=0)
        self.assertTrue(temporary.exists())


def space(free, used=0):
    return SimpleNamespace(
        f_bavail=free, f_frsize=1, f_blocks=free + used, f_bfree=free
    )


class StorageGuardTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def test_host_reserve_boundary(self):
        storage_guard.check_space(space(150_000_000_000), space(1, 0))
        with self.assertRaisesRegex(RuntimeError, "150 GB"):
            storage_guard.check_space(space(149_999_999_999), space(1, 0))

    def test_docker_budget_boundary(self):
        with self.assertRaisesRegex(RuntimeError, "200 GB"):
            storage_guard.check_space(space(200_000_000_000), space(1, 200_000_000_000))

    def test_ci_bypasses_machine_specific_guard(self):
        with patch.dict(os.environ, {"GITHUB_ACTIONS": "true"}):
            self.assertFalse(storage_guard.local_container())

    def test_explicit_server_and_alternate_cache_are_rejected(self):
        for argument in ("--nobatch", "--batch=false", "--disk_cache=/tmp/other"):
            with self.assertRaises(RuntimeError):
                storage_guard.run_guarded(["unused", argument])

    def test_budget_failure_prevents_launch(self):
        with (
            patch.object(storage_guard, "CACHE_ROOT", self.root / "cache"),
            patch.object(storage_guard, "LOCK", self.root / "lock"),
            patch.object(storage_guard, "maintain"),
            patch.object(
                storage_guard, "check_current_space", side_effect=RuntimeError("full")
            ),
            patch.object(subprocess, "Popen") as launch,
        ):
            with self.assertRaisesRegex(RuntimeError, "full"):
                storage_guard.run_guarded(["unused"])
            launch.assert_not_called()

    def test_running_child_stopped_at_budget_and_post_cleanup_runs(self):
        with (
            patch.object(storage_guard, "CACHE_ROOT", self.root / "cache"),
            patch.object(storage_guard, "LOCK", self.root / "lock"),
            patch.object(storage_guard, "maintain") as maintenance,
            patch.object(
                storage_guard,
                "check_current_space",
                side_effect=[None, RuntimeError("full")],
            ),
            patch.object(subprocess, "Popen") as launch,
            patch.object(storage_guard, "stop") as stop,
        ):
            launch.return_value.poll.return_value = None
            with self.assertRaisesRegex(RuntimeError, "full"):
                storage_guard.run_guarded(["bazel", "test", "//..."])
            self.assertEqual(
                launch.call_args.args[0], ["bazel", "--batch", "test", "//..."]
            )
            stop.assert_called_once_with(launch.return_value)
            self.assertEqual(maintenance.call_count, 2)

    def test_build_exit_code_preserved_and_cleanup_runs(self):
        for code in (0, 7):
            with (
                patch.object(storage_guard, "CACHE_ROOT", self.root / "cache"),
                patch.object(storage_guard, "LOCK", self.root / "lock"),
                patch.object(storage_guard, "maintain") as maintenance,
                patch.object(storage_guard, "check_current_space"),
                patch.object(subprocess, "Popen") as launch,
            ):
                launch.return_value.poll.return_value = code
                launch.return_value.returncode = code
                self.assertEqual(storage_guard.run_guarded(["bazel", "test"]), code)
                self.assertEqual(maintenance.call_count, 2)

    def test_actual_child_process_is_reaped(self):
        process = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(60)"],
            start_new_session=True,
        )
        self.addCleanup(storage_guard.stop, process)
        storage_guard.stop(process)
        self.assertIsNotNone(process.poll())


if __name__ == "__main__":
    unittest.main()
