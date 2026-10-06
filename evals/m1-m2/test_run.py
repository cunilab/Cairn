#!/usr/bin/env python3
"""Focused checks for evaluation validity metadata; no agents or services required."""

import importlib.util
import contextlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
import runpy
import subprocess


MODULE = Path(__file__).with_name("run.py")


def load_runner():
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / "cases.json").write_text("[]")
        os.environ.update({"CAIRN_M2_SOURCES": str(root), "CAIRN_M2_OUT": str(root),
                           "CAIRN_M2_CREDENTIALS": str(root / "credentials.json"),
                           "CAIRN_M2_CASES": str(root / "cases.json")})
        spec = importlib.util.spec_from_file_location("m2_run", MODULE)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module


RUN = load_runner()


class RunnerEvidenceTests(unittest.TestCase):
    def test_hook_capture_forwards_bytes_and_exit_status(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "fake-cairn"
            binary.write_text('#!/usr/bin/env python3\nimport sys\n'
                              'sys.stdout.buffer.write(b"context\\n")\n'
                              'sys.stderr.buffer.write(b"diagnostic\\n")\n'
                              'sys.exit(3)\n')
            binary.chmod(0o700)
            log = root / "hooks.jsonl"
            output = subprocess.run([str(MODULE.with_name("hook_capture.py")), "hook", "SessionStart"],
                env={**os.environ, "CAIRN_M2_REAL_BIN": str(binary), "CAIRN_M2_HOOK_LOG": str(log)},
                capture_output=True)
            self.assertEqual((output.stdout, output.stderr, output.returncode),
                             (b"context\n", b"diagnostic\n", 3))
            record = json.loads(log.read_text())
            self.assertEqual(record["stdout"], "context\n")
            self.assertEqual(record["stderr"], "diagnostic\n")
            self.assertTrue(record["complete"])
            self.assertTrue(record["stdout_forwarded"])
            self.assertTrue(record["stderr_forwarded"])

    def test_project_trust_path_is_not_cairn_configuration(self):
        with tempfile.TemporaryDirectory(prefix="cairn-") as directory:
            root = Path(directory)
            repo, home = root / "repo", root / "home"
            repo.mkdir()
            (home / ".codex").mkdir(parents=True)
            config = home / ".codex/config.toml"
            trust = f'[projects."{repo}"]\ntrust_level = "trusted"\n'
            config.write_text(trust)
            self.assertTrue(RUN.control_configuration(repo, home)["hook_configuration_absent"])
            config.write_text(trust + f'[mcp_servers.cairn]\ncommand = "{repo}/target/debug/cairn"\n')
            self.assertFalse(RUN.control_configuration(repo, home)["hook_configuration_absent"])

    def test_privacy_summary_checks_stderr_and_complete_traces(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cases = [{"id": "P1", "lane": "cross_project", "seed": "PRIVATE"}]
            (root / "cases.json").write_text(json.dumps(cases))
            result = {"treatment_memory_count": 0, "seed_tool_visible": False,
                      "delivery_probe": {"context": {"sentinel_visible": True}},
                      "arms": {}}
            for arm in ("control", "treatment"):
                folder = root / "P1" / arm
                folder.mkdir(parents=True)
                (folder / "later.out").write_text("")
                (folder / "later.err").write_text("PRIVATE" if arm == "treatment" else "")
                result["arms"][arm] = {"later": {"tools": [], "status": 0, "seconds": 1,
                    "model": "test", "trace_complete": True, "usage": {}}}
            os.environ.update({"CAIRN_M2_CASES": str(root / "cases.json"), "CAIRN_M2_OUT": str(root)})
            for complete, visible in ((True, True), (False, True), (False, False)):
                (root / "P1/treatment/later.err").write_text("PRIVATE" if visible else "")
                result["arms"]["treatment"]["later"]["trace_complete"] = complete
                (root / "P1/result.json").write_text(json.dumps(result))
                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    runpy.run_path(str(MODULE.with_name("summarize.py")))
                summary = json.loads(output.getvalue())["summary"]
                self.assertEqual(summary["privacy_boundary_leaks"], 1)
                self.assertEqual(summary["privacy_trace_leaks"], int(visible))
                self.assertEqual(summary["privacy_trace_cases"], int(complete or visible))

    def test_observed_reported_model_drift_invalidates_pair(self):
        arms = {arm: {"later": {"reported_model": model}}
                for arm, model in (("control", "model-a"), ("treatment", "model-b"))}
        status = RUN.model_status(arms, ("later",))
        self.assertTrue(status["reported_model_drift"])
        self.assertTrue(status["reported_identity_verified"])

    def test_refusal_probe_requires_an_actual_refusal(self):
        refused = RUN.inspection(0, json.dumps({"result": {"isError": True}}), "FAKE", True)
        accepted = RUN.inspection(0, json.dumps({"result": {"isError": False}}), "FAKE", True)
        self.assertTrue(refused["passed"])
        self.assertFalse(accepted["passed"])

    def test_malformed_probe_response_is_not_valid(self):
        probe = RUN.inspection(0, "not json", "FAKE", False)
        self.assertFalse(probe["json_valid"])
        self.assertFalse(probe["passed"])

    def test_hazard_visibility_does_not_break_transport_probe(self):
        sentinel = "SYNTHETIC-STALE"
        probe = RUN.inspection(0, json.dumps({"result": {"isError": False,
                                                "content": [{"text": sentinel}]}}), sentinel, False)
        self.assertTrue(probe["passed"])
        self.assertTrue(probe["sentinel_visible"])
        self.assertFalse(probe["sentinel_absent"])

    def test_setup_failure_is_a_denominator_result(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            case = {"id": "T1", "repo": "missing", "agent": "codex", "lane": "natural"}
            originals = {name: getattr(RUN, name) for name in
                         ("OUT", "identities", "clone", "environment", "api", "subprocess")}
            try:
                RUN.OUT = root
                RUN.identities = lambda _: {"candidate_sha": "c", "corpus_sha256": "h", "source_sha": "s"}
                RUN.clone = lambda *_: (root, "https://example.invalid/repo.git")
                RUN.environment = lambda *_: ({}, root)
                RUN.api = lambda *_: {"id": "project"}
                RUN.subprocess = type("Subprocess", (), {
                    "run": staticmethod(lambda *_, **__: type("Run", (), {"returncode": 1})())})
                RUN.run_case(case, {"server_url": "http://example.invalid", "server_token": "test"})
            finally:
                for name, value in originals.items():
                    setattr(RUN, name, value)
            result = json.loads((root / "T1/result.json").read_text())
        self.assertFalse(result["valid"])
        self.assertEqual(result["failure"], {"stage": "setup", "kind": "RuntimeError"})
        self.assertIn("runner_setup_failure", result["invalid_reasons"])


if __name__ == "__main__":
    unittest.main()
