"""Decky backend for Flat to 3D (SPEC M11). Decky backends are Python; this one calls the Rust
Inject engine binary `flat3d-inject` as a subprocess. The QAM panel (decky/src/, TypeScript) is
built in its own issue. Skeleton: methods return a not-built error."""
import asyncio
import json
import os

# Where the plugin finds the engine binary: bundled next to this file by the packaging issue.
ENGINE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "bin", "flat3d-inject")

# The largest Steam appid the engine accepts (a u32, as in inject/src/cli.rs).
MAX_APPID = 0xFFFFFFFF


def failure(command: str, code: str, message: str) -> dict:
    """The engine's failure envelope (docs/code-map.md, the --json output)."""
    return {"ok": False, "command": command, "error": {"code": code, "message": message}}


def appid_text(appid: object) -> str:
    """The appid as plain decimal text, or "" if it is not one. bool and float are refused:
    int() would turn True into 1 and 5.9 into 5. Digit strings are trimmed to ten digits before
    int(), since int() refuses very long strings (a Python 3.11+ limit) and no u32 needs more."""
    # Exact types only: a subclass can override __str__, __int__, lstrip and the rest, which
    # would change the text sent on. type() is int also refuses bool.
    if type(appid) is int:
        value = appid
    elif type(appid) is str and appid.isascii() and appid.isdigit():
        digits = appid.lstrip("0") or "0"
        if len(digits) > 10:
            return ""
        value = int(digits)
    else:
        return ""
    return str(value) if 0 <= value <= MAX_APPID else ""


async def run_engine(command: str, *args: str) -> dict:
    """Run `flat3d-inject <command> <args> --json` and return its envelope. Not wired up yet: the
    engine's commands are still skeletons, so this returns the not-built error."""
    line = " ".join(["flat3d-inject", command, *args, "--json"])
    return failure(command, "not-built", f"not built yet: {line}")


class Plugin:
    async def list_games(self) -> dict:
        return await run_engine("list")

    async def inject(self, appid: object = None) -> dict:
        text = appid_text(appid)
        if not text:
            # A bad appid is a bad command line to the engine, so its envelope has no command name.
            return failure("", "usage", "usage: flat3d-inject inject <appid>")
        return await run_engine("inject", text)

    async def start_3d(self) -> dict:
        return await run_engine("start-3d")


# json and asyncio are used once the subprocess call is built (create_subprocess_exec, json output).
_ = (asyncio, json)
