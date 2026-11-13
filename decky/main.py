"""Decky shim for the Flat to 3D engine."""
import asyncio
import json
import logging
import os
import signal

try:
    import decky
except ImportError:
    decky = None


ENGINE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "bin", "flat3d-inject")
_MODES_FIXED = False


def _failure(command: str, code: str, message: str) -> dict:
    return {"ok": False, "command": command, "error": {"code": code, "message": message}}


def _env() -> dict:
    env = os.environ.copy()
    engine_dir = os.path.dirname(ENGINE)
    env.setdefault("XDG_RUNTIME_DIR", f"/run/user/{os.getuid()}")
    env.setdefault("DBUS_SESSION_BUS_ADDRESS", f"unix:path=/run/user/{os.getuid()}/bus")
    env.setdefault("FLAT3D_SRC_DIR", engine_dir)
    return env


def _fix_modes() -> None:
    global _MODES_FIXED
    if _MODES_FIXED:
        return
    engine_dir = os.path.dirname(ENGINE)
    paths = (
        ENGINE,
        os.path.join(engine_dir, "flat3d-run"),
        os.path.join(engine_dir, "tools", "adb"),
    )
    for path in paths:
        try:
            os.chmod(path, 0o755)
        except OSError:
            pass
    _MODES_FIXED = True


async def run_engine(*args: str, timeout: float = 30.0) -> dict:
    command = args[0] if args else ""
    _fix_modes()
    try:
        proc = await asyncio.create_subprocess_exec(
            ENGINE, *args, "--json", stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE, env=_env(), start_new_session=True,
        )
    except OSError as e:
        return _failure(command, "io", f"cannot run the engine: {e}")

    communication = asyncio.create_task(proc.communicate())
    try:
        stdout, _stderr = await asyncio.wait_for(asyncio.shield(communication), timeout)
    except asyncio.TimeoutError:
        await _stop_engine(proc, communication)
        return _failure(command, "command-failed", f"timed out after {timeout:g}s")
    except asyncio.CancelledError:
        await _stop_engine(proc, communication)
        raise

    detail = stdout.decode(errors="replace").strip()
    try:
        result = json.loads(detail)
    except (json.JSONDecodeError, TypeError):
        result = None
    if isinstance(result, dict) and isinstance(result.get("ok"), bool):
        return result
    return _failure(
        command, "command-failed",
        f"engine printed invalid JSON (exit code {proc.returncode}): {detail[:200]}",
    )


async def _stop_engine(proc, communication) -> None:
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    await communication


class Plugin:
    def __init__(self) -> None:
        self.log = decky.logger if decky is not None else logging.getLogger("flat3d")

    async def list_games(self) -> dict:
        return await run_engine("list")

    async def inject(self, appid: object = None) -> dict:
        return await run_engine("inject", "" if appid is None else str(appid))

    async def start_3d(self) -> dict:
        return await run_engine("start-3d")

    async def engine(self, args: list) -> dict:
        return await run_engine(*[str(a) for a in args])

    async def _main(self) -> None:
        _fix_modes()

    async def _unload(self) -> None:
        return None
