"""Find RTTI vtables containing a native method, without starting the game.

Usage: find_method_owners.py EXECUTABLE RVA [RVA ...]"""
# Copied from SpidyVR v0.3.8 (aed5c0c), tools/find_method_owners.py, by Ilya Mezerowsky (GPL-3.0).
# https://github.com/IlyaMez/SpidyVR . Modified for Flat to 3D: the Remastered hash/path
# gates are gone, so it reads any PE32+ (Spider-Man2.exe included). GPL-3.0.
import pathlib
import struct
import sys
from inspect_game import PE
from discover_render_types import occurrences, rva_for_offset


def main():
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2
    data = pathlib.Path(sys.argv[1]).read_bytes()
    pe = PE(data)
    for value in sys.argv[2:]:
        target = int(value, 0)
        for offset in occurrences(data, struct.pack('<Q', pe.image_base + target)):
            if offset % 8:
                continue
            at = rva_for_offset(pe, offset)
            for slot in range(256):
                table = at-slot*8
                try:
                    owner = pe.vtable(table)
                except (ValueError, struct.error):
                    continue
                print(dict(method=hex(target), table=hex(table), slot=hex(slot*8), **owner))
                for member in (0x138,):
                    address = struct.unpack('<Q', pe.bytes(table+member, 8))[0]-pe.image_base
                    print(f'  method[{member:#x}] = {address:#x}')
                break


if __name__ == '__main__':
    sys.exit(main())
