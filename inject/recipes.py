"""Recipes: each game's machine-readable stereo keys (docs/notes/DECISIONS.md, "Recipe keys").

Where the records live is #15's question; for 1.0 the engine needs Spider-Man 2's (SPEC M12).
"""
from __future__ import annotations
from dataclasses import dataclass
from pathlib import Path

from .notimpl import NotBuilt

STATUSES = ("works-now", "likely", "build", "not-safe", "wont-work", "untested")
APIS = ("dx9", "dx11", "dx12", "vulkan", "opengl", "unknown")


@dataclass(frozen=True)
class Recipe:
    appid: int
    name: str
    status: str        # stereo-status, one of STATUSES
    route: str         # stereo-route, e.g. "build: port spidy stereo screen"
    engine: str        # stereo-engine
    api: str           # stereo-api, one of APIS
    mod: str           # stereo-mod: URL or "none"
    free: str          # stereo-free: "yes" | "no" | "unclear"
    safe: str          # stereo-safe: "yes" | "no"
    checked: str       # stereo-checked: YYYY-MM-DD


def load_recipes(folder: Path) -> dict[int, Recipe]:
    """Every recipe file under `folder`, keyed by Steam appid. Rejects unknown status or api values."""
    raise NotBuilt("recipes.load_recipes: see the recipes issue")
