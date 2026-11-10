# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""MSVC RTTI: vtable -> type name, and type name -> vtables (x64, image-relative COL)."""
import struct


def vtable_name(pe, vt):
    b = pe.bytes(vt - 8, 8)
    if not b:
        return None
    col = int.from_bytes(b, 'little') - pe.image_base
    c = pe.bytes(col, 24)
    if not c:
        return None
    sig, off, cd, td, ch, me = struct.unpack('<IIIIII', c)
    if sig != 1:
        return None
    return pe.cstr(td + 16), off


def vtables_for(pe, name):
    """All (vtable rva, col offset) whose COL names this decorated type ('.?AV...@@')."""
    out = []
    for o in pe.find_all(b'\0' * 8 + name.encode() + b'\0', limit=4):
        td = pe.rva_of(o + 8) - 16
        for p in pe.find_all(struct.pack('<I', td), limit=200):
            col = pe.rva_of(p) - 12 if pe.rva_of(p) else None
            if col is None:
                continue
            c = pe.bytes(col, 24)
            if not c:
                continue
            sig, off, cd, tdd, ch, me = struct.unpack('<IIIIII', c)
            if sig != 1 or tdd != td or me != col:
                continue
            for q in pe.find_all(struct.pack('<Q', pe.image_base + col), limit=20):
                vr = pe.rva_of(q)
                if vr:
                    out.append((vr + 8, off))
    return out
