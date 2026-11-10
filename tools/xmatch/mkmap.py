# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""Render the address-map table (markdown) from targets.py plus an optional run_match JSON."""
import sys, os, json
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__))))
from targets import T
res = {}
if len(sys.argv) > 1 and os.path.exists(sys.argv[1]):
    for e in json.load(open(sys.argv[1]))['items']:
        res[e['rem']] = e
pn = {1: 'Steps 1-3: camera, frame entry, view setup', 2: 'Steps 4-5: eye views, render jobs, buffers, memory, pause gate',
      3: 'Step 6-7 and removable: HUD, game-time checks, shadows, diagnostics, gate extras'}
for p in (1, 2, 3):
    print('\n### Priority %d. %s\n' % (p, pn[p]))
    print('| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found |')
    print('|---|---|---|---|---|---|---|---|')
    for name, rva, kind, prio, hx in T:
        if prio != p:
            continue
        e = res.get(hex(rva), {})
        sm2 = e.get('sm2')
        if isinstance(sm2, list):
            sm2 = ', '.join(sm2)
        print('| %s | %s | `%s` | %s | %s | %s | %s | %s |' % (
            name, kind, hex(rva), ('`%s`' % hx.replace(' ', '')) if hx else '-',
            ('`%s`' % sm2) if sm2 else 'pending', ('`%s`' % e['sig']) if e.get('sig') else '-',
            e.get('conf', 'pending'), e.get('how', '-')))
