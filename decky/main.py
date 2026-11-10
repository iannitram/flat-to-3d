"""Decky backend for Flat to 3D (SPEC M11). Decky backends are Python; this one calls the Rust
Inject engine binary `flat3d-inject` as a subprocess. The QAM panel (decky/src/, TypeScript) is
built in its own issue. It holds no rules: the engine checks every argument (inject/src/cli.rs).
Until the engine binary is bundled, every method returns the io error of a missing binary."""
import asyncio
import json
import os

# Where the plugin finds the engine binary: bundled next to this file by the packaging issue.
ENGINE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "bin", "flat3d-inject")


def failure(command: str, code: str, message: str) -> dict:
    """The engine's failure envelope (docs/code-map.md, the --json output)."""
    return {"ok": False, "command": command, "error": {"code": code, "message": message}}


async def run_engine(command: str, *args: str) -> dict:
    """Run `flat3d-inject <command> <args> --json` and return its envelope. Not wired up yet, so
    this returns the io error a missing engine binary gives (docs/code-map.md, the Decky backend's
    own errors)."""
    line = " ".join(["flat3d-inject", command, *args, "--json"])
    return failure(command, "io", f"engine binary not bundled yet: {line}")


class Plugin:
    async def list_games(self) -> dict:
        return await run_engine("list")

    async def inject(self, appid: object = None) -> dict:
        # The face only turns the value into text; the engine decides whether it is an appid (a u32).
        return await run_engine("inject", "" if appid is None else str(appid))

    async def start_3d(self) -> dict:
        return await run_engine("start-3d")


# json and asyncio are used once the subprocess call is built (create_subprocess_exec, json output).
_ = (asyncio, json)
