"""Decky backend for Flat to 3D (SPEC M11). Calls the Inject engine; the QAM panel
(decky/src/, TypeScript) is built in its own issue. Skeleton: methods return a not-built error."""


class Plugin:
    async def list_games(self) -> dict:
        return {"ok": False, "error": "not built yet"}

    async def inject(self, appid: int) -> dict:
        return {"ok": False, "error": "not built yet"}

    async def start_3d(self) -> dict:
        return {"ok": False, "error": "not built yet"}
