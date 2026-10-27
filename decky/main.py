"""Decky backend for Flat to 3D (SPEC M11). Decky backends are Python; this one calls the Rust
Inject engine binary `flat3d-inject` as a subprocess. The QAM panel (decky/src/, TypeScript) is
built in its own issue. Skeleton: methods return a not-built error."""
import asyncio
import json
import os

# Where the plugin finds the engine binary: bundled next to this file by the packaging issue.
ENGINE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "bin", "flat3d-inject")


async def run_engine(*args: str) -> dict:
    """Run `flat3d-inject <args>` and return {"ok", "code", "out", "err"}. Not wired up yet."""
    return {"ok": False, "error": "not built yet", "argv": [ENGINE, *args]}


class Plugin:
    async def list_games(self) -> dict:
        return await run_engine("list")

    async def inject(self, appid: int) -> dict:
        return await run_engine("inject", str(int(appid)))

    async def start_3d(self) -> dict:
        return await run_engine("start-3d")


# json and asyncio are used once the subprocess call is built (create_subprocess_exec, json output).
_ = (asyncio, json)
