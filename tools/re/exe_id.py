#!/usr/bin/env python3
# Flat to 3D (GPL-3.0). Offline, read-only: reads a game executable on disk, never runs it.
"""exe_id.py <exe> [--json]: name the build of a game executable.

Prints `<slug> <sha256> <version>` on one line (`--json`: an object with the keys
`slug`, `sha256` and `version`). The slug comes from KNOWN_BUILDS by the file's
SHA-256; the version is the file's own PE version resource, or `-` when it has none.
An unknown hash exits 3 and names the fix. tools/re/analyze.sh uses the slug as the
Ghidra project folder.
"""
import argparse
import hashlib
import json
import mmap
import struct
import sys

# sha256 -> (slug, version), from tools/research/inspect_game.py (EXPECTED_SHA256 and KNOWN_BUILDS).
KNOWN_BUILDS = {
    # Marvel's Spider-Man Remastered 4.630.0.0 (Steam build 23986256), SpidyVR's target
    "e297d4d94f1ffe4febf289745e79e7b6fa233a788e7a00f480fc77c55db81ad1": ("remastered-4.630", "4.630.0.0"),
    # Marvel's Spider-Man 2 2.810.0.0 (Steam build 24651585), RTTI stripped
    "c9b912eff96facb746a705ac8b0b83c29faa7bbe53d984a0c5eb45efd0b738c9": ("sm2-2.810", "2.810.0.0"),
}

CHUNK = 1 << 20


class UnknownBuild(LookupError):
    """The file's SHA-256 is not in KNOWN_BUILDS."""

    def __init__(self, sha):
        super().__init__(sha)
        self.sha = sha


def sha256_of(path):
    """The file's SHA-256 (hex), read in 1 MiB chunks."""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(CHUNK), b""):
            h.update(block)
    return h.hexdigest()


def pe_version(path):
    """The file's own version from its version resource ('4.630.0.0'), or '-'.

    The same scan as file_version in tools/research/inspect_game.py: find the
    VS_FIXEDFILEINFO signature 0xFEEF04BD after the UTF-16 key "VS_VERSION_INFO".
    A file that is empty or not a PE ('MZ' header) has no version: '-'."""
    with open(path, "rb") as f:
        if f.read(2) != b"MZ":
            return "-"
        with mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ) as data:
            key = "VS_VERSION_INFO".encode("utf-16-le")
            sig = struct.pack("<I", 0xFEEF04BD)
            at = data.rfind(key)
            while at >= 0:
                fixed = data.find(sig, at, at + 128)
                if fixed >= 0 and fixed + 16 <= len(data):
                    _, _, high, low = struct.unpack_from("<4I", data, fixed)
                    return f"{high >> 16}.{high & 0xffff}.{low >> 16}.{low & 0xffff}"
                at = data.rfind(key, 0, at)
    return "-"


def identify(path) -> tuple[str, str, str]:
    """(slug, sha256, pe_version) for the file at `path`; raises UnknownBuild for an unknown hash."""
    sha = sha256_of(path)
    if sha not in KNOWN_BUILDS:
        raise UnknownBuild(sha)
    slug, _ = KNOWN_BUILDS[sha]
    return slug, sha, pe_version(path)


def main(argv=None):
    ap = argparse.ArgumentParser(prog="exe_id.py", description="Name the build of a game executable.")
    ap.add_argument("exe")
    ap.add_argument("--json", action="store_true", help="print an object with slug, sha256 and version")
    args = ap.parse_args(argv)
    try:
        slug, sha, ver = identify(args.exe)
    except UnknownBuild as e:
        print(f"exe_id: unknown {e.sha}; fix: add it to KNOWN_BUILDS in tools/re/exe_id.py", file=sys.stderr)
        return 3
    except OSError as e:
        print(f"exe_id: {args.exe}: {e.strerror or e}; fix: pass the path of a readable executable", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps({"slug": slug, "sha256": sha, "version": ver}))
    else:
        print(f"{slug} {sha} {ver}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
