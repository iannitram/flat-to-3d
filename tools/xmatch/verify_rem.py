# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""Step-0 risk check: every Spidy address against an on-disk Remastered exe."""
import sys, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'research'))
from pe import PE
from targets import T, hexbytes
from inspect_game import file_version, EXPECTED_SHA256, EXPECTED_VERSION

path = sys.argv[1]
p = PE(path)
sha = p.sha256()
out = dict(path=path, sha256=sha, expected_sha256=EXPECTED_SHA256, sha_match=sha == EXPECTED_SHA256,
           version=file_version(path), expected_version=EXPECTED_VERSION, size_of_image=p.size_of_image,
           spidy_size_of_image=140496896, timestamp=p.timestamp, sections=[s['name'] for s in p.sections], items=[])
ok = bad = nocheck = 0
for name, rva, kind, prio, hx in T:
    r = dict(name=name, rva=hex(rva), kind=kind)
    if hx:
        want = hexbytes(hx)
        got = p.bytes(rva, len(want))
        r['match'] = got == want
        r['got'] = got.hex() if got else None
        ok += r['match']; bad += not r['match']
    else:
        nocheck += 1
    if kind in ('hook', 'call', 'gate'):
        r['pdata_start'] = p.is_start(rva)
    if kind == 'vtable':
        b = p.bytes(rva - 8, 8)
        col = int.from_bytes(b, 'little') - p.image_base if b else 0
        cb = p.bytes(col + 12, 4) if col > 0 else None
        if cb:
            td = int.from_bytes(cb, 'little')
            r['rtti'] = p.cstr(td + 16)
    out['items'].append(r)
out['summary'] = dict(byte_checks_match=ok, byte_checks_mismatch=bad, no_bytes=nocheck)
print(json.dumps(out, indent=1))
