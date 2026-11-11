"""Locked tests for F1a: decky/main.py run_engine. A fake engine script stands in for
bin/flat3d-inject; the contract is docs/code-map.md, "API shape"."""
import os
import stat
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import main  # noqa: E402


class RunEngineTest(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.saved = main.ENGINE
        self.addCleanup(setattr, main, "ENGINE", self.saved)

    def fake(self, body):
        path = os.path.join(self.tmp.name, "engine")
        with open(path, "w") as f:
            f.write("#!/bin/sh\n" + body + "\n")
        os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)
        main.ENGINE = path

    async def test_success_returns_the_envelope_and_appends_json(self):
        self.fake('echo "{\\"ok\\": true, \\"command\\": \\"$1\\", \\"data\\": [\\"$*\\"]}"')
        got = await main.run_engine("list")
        self.assertEqual(got, {"ok": True, "command": "list", "data": ["list --json"]})

    async def test_arguments_keep_their_order_before_json(self):
        self.fake('echo "{\\"ok\\": true, \\"command\\": \\"inject\\", \\"data\\": \\"$*\\"}"')
        got = await main.run_engine("inject", "2651280")
        self.assertEqual(got["data"], "inject 2651280 --json")

    async def test_nonzero_exit_with_an_envelope_returns_that_envelope(self):
        env = '{"ok": false, "command": "inject", "error": {"code": "refused", "message": "no"}}'
        self.fake("echo '%s'\nexit 1" % env)
        got = await main.run_engine("inject", "1")
        want = {"ok": False, "command": "inject", "error": {"code": "refused", "message": "no"}}
        self.assertEqual(got, want)

    async def test_nonzero_exit_without_json_is_command_failed(self):
        self.fake("echo boom >&2\nexit 3")
        got = await main.run_engine("status")
        self.assertFalse(got["ok"])
        self.assertEqual(got["command"], "status")
        self.assertEqual(got["error"]["code"], "command-failed")
        self.assertTrue(got["error"]["message"].startswith("engine printed invalid JSON (exit code 3)"))
        self.assertNotIn("data", got)

    async def test_bad_json_with_exit_zero_is_command_failed(self):
        self.fake("echo not json")
        got = await main.run_engine("list")
        self.assertEqual(got["error"]["code"], "command-failed")
        self.assertTrue(got["error"]["message"].startswith("engine printed invalid JSON (exit code 0)"))

    async def test_json_that_is_not_an_envelope_is_command_failed(self):
        self.fake("echo '[1, 2]'")
        got = await main.run_engine("list")
        self.assertFalse(got["ok"])
        self.assertEqual(got["error"]["code"], "command-failed")
        self.fake("echo '{\"command\": \"list\"}'")
        got = await main.run_engine("list")
        self.assertEqual(got["error"]["code"], "command-failed")

    async def test_a_slow_engine_times_out(self):
        self.fake("exec sleep 5")
        got = await main.run_engine("start-3d", timeout=0.3)
        self.assertEqual(
            got,
            {"ok": False, "command": "start-3d",
             "error": {"code": "command-failed", "message": "timed out after 0.3s"}},
        )

    async def test_a_missing_engine_is_an_io_error(self):
        main.ENGINE = os.path.join(self.tmp.name, "no-such-engine")
        got = await main.run_engine("list")
        self.assertFalse(got["ok"])
        self.assertEqual(got["error"]["code"], "io")
        self.assertTrue(got["error"]["message"].startswith("cannot run the engine: "))

    async def test_a_failure_with_no_arguments_has_an_empty_command(self):
        self.fake("exit 2")
        got = await main.run_engine()
        self.assertEqual(got["command"], "")
        self.assertEqual(got["error"]["code"], "command-failed")


if __name__ == "__main__":
    unittest.main()
