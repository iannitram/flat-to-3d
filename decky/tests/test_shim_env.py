import json
import os
import stat
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
import main  # noqa: E402


class ShimEnvironmentTest(unittest.IsolatedAsyncioTestCase):
    def test_defaults_and_existing_environment_are_preserved(self):
        keys = ("XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS", "FLAT3D_SRC_DIR")
        old = {key: os.environ.pop(key, None) for key in keys}
        try:
            env = main._env()
            self.assertEqual(env["XDG_RUNTIME_DIR"], f"/run/user/{os.getuid()}")
            self.assertEqual(env["DBUS_SESSION_BUS_ADDRESS"], f"unix:path=/run/user/{os.getuid()}/bus")
            self.assertEqual(env["FLAT3D_SRC_DIR"], os.path.dirname(main.ENGINE))
            supplied = dict(zip(keys, ("/tmp/runtime", "unix:path=/tmp/bus", "/tmp/source")))
            with mock.patch.dict(os.environ, supplied):
                self.assertEqual({key: main._env()[key] for key in keys}, supplied)
        finally:
            for key, value in old.items():
                if value is not None:
                    os.environ[key] = value

    async def test_standin_engine_receives_environment_and_trailing_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            engine_dir = os.path.join(tmp, "bin")
            os.makedirs(engine_dir)
            engine = os.path.join(engine_dir, "flat3d-inject")
            with open(engine, "w", encoding="utf-8") as file:
                file.write(
                    f"#!{sys.executable}\n"
                    "import json, os, sys\n"
                    "print(json.dumps({'ok': True, 'command': 'view', 'data': {"
                    "'XDG_RUNTIME_DIR': os.environ['XDG_RUNTIME_DIR'], "
                    "'DBUS_SESSION_BUS_ADDRESS': os.environ['DBUS_SESSION_BUS_ADDRESS'], "
                    "'FLAT3D_SRC_DIR': os.environ['FLAT3D_SRC_DIR'], 'argv': sys.argv[1:]}}))\n"
                )
            os.chmod(engine, stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR)
            with mock.patch.object(main, "ENGINE", engine), mock.patch.object(main, "_MODES_FIXED", False):
                expected = main._env()
                result = await main.run_engine("view")

            self.assertTrue(result["ok"])
            data = result["data"]
            for key in ("XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS", "FLAT3D_SRC_DIR"):
                self.assertEqual(data[key], expected[key])
            self.assertEqual(data["argv"], ["view", "--json"])


if __name__ == "__main__":
    unittest.main()
