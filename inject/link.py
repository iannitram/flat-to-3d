"""Start 3D (SPEC M10): Quest Tether's USB link plus WiVRn's wired mode (adb reverse tcp:9757)."""
from __future__ import annotations
from dataclasses import dataclass

from .notimpl import NotBuilt

WIVRN_PORT = 9757


@dataclass(frozen=True)
class LinkState:
    headset_seen: bool    # adb lists the Quest
    tether_up: bool       # Quest Tether reports its link
    wivrn_ready: bool     # adb reverse is set and the WiVRn server is running


def start_3d_commands() -> list[list[str]]:
    """The commands Start 3D runs, in order (pure: tests assert on this list)."""
    raise NotBuilt("link.start_3d_commands: see the Start 3D issue")


def start_3d() -> LinkState:
    """Run start_3d_commands() and report where the link stands."""
    raise NotBuilt("link.start_3d: see the Start 3D issue")
