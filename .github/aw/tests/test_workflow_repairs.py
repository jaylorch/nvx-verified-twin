"""Regression checks for the verified twin's agentic workflow configuration."""

import contextlib
import io
import json
import os
import re
import subprocess
import textwrap
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
WORKFLOWS = ROOT / ".github" / "workflows"
CATEGORIES = ("quality", "documentation", "deduplication", "reusability")


class WorkflowRepairsTests(unittest.TestCase):
    def test_dispatch_and_writer_use_the_same_existing_base(self):
        shared = (WORKFLOWS / "shared" / "code-improvement.md").read_text()
        updater = (WORKFLOWS / "documentation-updater.md").read_text()
        for source in (shared, updater):
            self.assertRegex(source, r"(?m)^    base-branch: main$")
            self.assertRegex(source, r"(?m)^      DEFAULT_BRANCH: main$")
        for category in CATEGORIES:
            source = (WORKFLOWS / f"code-{category}.md").read_text()
            self.assertIn("github.ref_name == 'main'", source)
            self.assertNotIn("github.ref_name == 'dev'", source)

    def test_incomplete_output_is_declared_without_disabling_failure_reports(self):
        for name in ("shared/code-improvement", "documentation-updater"):
            source = (WORKFLOWS / f"{name}.md").read_text()
            self.assertRegex(source, r"(?m)^  report-incomplete: \{\}$")
            self.assertNotIn("report-failure-as-issue: false", source)

    def run_design_validator(
        self, *, dirty=False, untracked=False, listing_error=False
    ):
        source = (WORKFLOWS / "documentation-updater.md").read_text()
        embedded = re.search(
            r"      cat > [^\n]+ <<'PY'\n(.*?)      PY\n", source, re.DOTALL
        )
        self.assertIsNotNone(embedded)
        code = compile(textwrap.dedent(embedded.group(1)), str(WORKFLOWS), "exec")
        commands = []

        def run(command, **kwargs):
            commands.append(command)
            self.assertTrue(kwargs["check"])
            if command == ["git", "rev-parse", "HEAD:openvmm"]:
                return subprocess.CompletedProcess(command, 0, stdout="a" * 40 + "\n")
            if command == ["git", "diff", "--exit-code", "HEAD", "--", "openvmm"]:
                if dirty:
                    raise subprocess.CalledProcessError(1, command)
                return subprocess.CompletedProcess(command, 0)
            if command == ["git", "rev-parse", "HEAD"]:
                return subprocess.CompletedProcess(command, 0, stdout="b" * 40 + "\n")
            if command == [
                "git",
                "ls-files",
                "--others",
                "--exclude-standard",
                "--",
                "openvmm",
            ]:
                if listing_error:
                    raise subprocess.CalledProcessError(128, command)
                return subprocess.CompletedProcess(
                    command,
                    0,
                    stdout="openvmm/untracked-source.rs\n" if untracked else "",
                )
            self.fail(f"Unexpected subprocess: {command}")

        with (
            patch.object(Path, "cwd", return_value=ROOT),
            patch.object(Path, "write_text") as write,
            patch("subprocess.run", side_effect=run),
            patch.dict(os.environ, {"DEFAULT_BRANCH": "main"}),
            patch("sys.argv", ["validate_design_docs.py"]),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            if dirty or listing_error:
                with self.assertRaises(subprocess.CalledProcessError):
                    exec(code, {})
                write.assert_not_called()
                return None
            if untracked:
                with self.assertRaisesRegex(SystemExit, "openvmm/untracked-source.rs"):
                    exec(code, {})
                write.assert_not_called()
                return None
            exec(code, {})
            write.assert_called_once()
            return json.loads(write.call_args.args[0])

    def test_vendored_tree_provenance_does_not_require_a_submodule_index_entry(self):
        context = self.run_design_validator()
        self.assertEqual(context["openvmm_tree"], "a" * 40)
        self.assertEqual(context["repository_sha"], "b" * 40)
        self.assertEqual(context["default_branch"], "main")
        self.assertNotIn("openvmm_sha", context)

    def test_modified_openvmm_source_blocks_context_generation(self):
        self.run_design_validator(dirty=True)

    def test_untracked_openvmm_source_blocks_context_generation(self):
        self.run_design_validator(untracked=True)

    def test_failed_untracked_listing_blocks_context_generation(self):
        self.run_design_validator(listing_error=True)


if __name__ == "__main__":
    unittest.main()
