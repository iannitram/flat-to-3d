"""Command line: `python3 -m inject list | inject <appid> | start-3d`."""
from __future__ import annotations
import argparse
import sys


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="flat3d", description="Flat to 3D's Inject engine")
    sub = p.add_subparsers(dest="command", required=True)
    sub.add_parser("list", help="installed games with their stereo status")
    i = sub.add_parser("inject", help="inject a game by Steam appid")
    i.add_argument("appid", type=int)
    sub.add_parser("start-3d", help="bring up the headset link")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    print(f"{args.command}: not built yet (skeleton)", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
