"""Installed Steam games: parse libraryfolders.vdf and each library's appmanifest_*.acf."""
from __future__ import annotations
from dataclasses import dataclass
from pathlib import Path

from .notimpl import NotBuilt

DEFAULT_STEAM_ROOT = Path("~/.local/share/Steam").expanduser()


@dataclass(frozen=True)
class Game:
    appid: int
    name: str
    install_dir: Path   # the game's folder under steamapps/common/


def installed_games(steam_root: Path = DEFAULT_STEAM_ROOT) -> list[Game]:
    """Every fully installed game in every Steam library, sorted by name."""
    raise NotBuilt("steam.installed_games: see the engine issue")
