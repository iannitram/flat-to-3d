# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""Single-exe survey: identity, protection signs, imports, RTTI for key types,
anchor-string xrefs, plan-relevant feature strings. usage: survey.py exe out.json"""
import sys, os, json, math, re, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'research'))
from collections import Counter
from pe import PE
import vt
from inspect_game import file_version

p = PE(sys.argv[1])
o = dict(path=sys.argv[1], version=file_version(sys.argv[1]), sha256=p.sha256(), size=os.path.getsize(sys.argv[1]),
         size_of_image=p.size_of_image, timestamp=p.timestamp)


def entropy(b):
    c = Counter(b); n = len(b)
    return round(-sum(v / n * math.log2(v / n) for v in c.values()), 3)


secs = []
for s in p.sections:
    sample = bytes(p.m[s['raw']:s['raw'] + min(s['rawsize'], 4 << 20)])
    secs.append(dict(name=s['name'], va=hex(s['va']), vsize=hex(s['vsize']), raw=hex(s['rawsize']), ch=hex(s['ch']),
                     entropy=entropy(sample) if sample else None))
o['sections'] = secs
# imports
irva, isz = p.dirs[1]
imps = {}
k = 0
while True:
    d = p.bytes(irva + 20 * k, 20)
    if not d or d == b'\0' * 20:
        break
    oft, ts, fc, name, ft = struct.unpack('<IIIII', d)
    dll = p.cstr(name) or '?'
    fns = []
    t = oft or ft
    j = 0
    while True:
        e = p.bytes(t + 8 * j, 8)
        v = int.from_bytes(e, 'little') if e else 0
        if not v:
            break
        fns.append(('#%d' % (v & 0xffff)) if v >> 63 else (p.cstr((v & 0x7fffffff) + 2) or '?'))
        j += 1
    imps[dll] = fns
    k += 1
o['imports'] = {d: (f if len(f) <= 40 else f[:40] + ['...+%d' % (len(f) - 40)]) for d, f in imps.items()}
o['tls'] = p.dirs[9]
o['security_dir'] = p.dirs[4]
# protection-ish strings
prot = {}
for s in [b'Denuvo', b'denuvo', b'VMProtect', b'Themida', b'.vmp', b'Arxan', b'EasyAntiCheat', b'BattlEye', b'anti-tamper',
          b'IsDebuggerPresent', b'CheckRemoteDebuggerPresent', b'NtQueryInformationProcess', b'crs-handler', b'steam_api64.dll',
          b'SteamAPI_RestartAppIfNecessary', b'PSN', b'psn', b'nixxes', b'Nixxes']:
    prot[s.decode()] = len(p.find_all(s, limit=50))
o['protection_strings'] = prot
# RTTI types of interest
names = ['Camera2::Camera', 'FollowCamera', 'CombatCamera', 'PlayerModelHudFollower', 'HeroLocal', 'MoverStandard',
         'ScaleformRTTStream', 'TimeScaleSystem', 'ViewContextManager', 'HeroCameraManager', 'CameraManager',
         'UISettingsMenuSystem', 'UISystemMenu', 'UISystemMenuItem', 'PhotoMode', 'DebugCamera', 'FreeCamera', 'Stereo']
rtti = {}
alltd = [m.group(0).decode() for m in re.finditer(rb'\.\?AV[A-Za-z0-9_@?$]{3,120}@@', p.m)]
o['rtti_type_count'] = len(alltd)
for n in names:
    parts = n.split('::')
    dec = '.?AV' + '@'.join(reversed(parts)) + '@@'
    hits = sorted({t for t in alltd if t == dec})
    if hits:
        rtti[n] = [hex(v) + '+%d' % off for v, off in vt.vtables_for(p, hits[0])][:6]
o['rtti'] = rtti
feat = re.compile(rb'(?i)(stereo|splitscreen|split_screen|photo ?mode|photomode|debug ?cam|freecam|free_cam|multiview|multi_view|'
                  rb'secondary ?view|render ?to ?texture|rendertarget view|eye ?index|left ?eye|right ?eye|openxr|openvr|oculus|vr ?mode)')
fh = Counter()
ex = {}
for m in re.finditer(rb'[\x20-\x7e]{5,120}', p.m):
    s = m.group(0)
    f = feat.search(s)
    if f:
        key = f.group(1).lower().decode()
        fh[key] += 1
        ex.setdefault(key, [])
        if len(ex[key]) < 12:
            ex[key].append(s.decode())
o['feature_strings'] = dict(counts=dict(fh), examples=ex)
# anchor strings -> xref functions
anchors = ['ViewContextManager', 'Adapted Lum', 'kTaaTarget0', 'kSsrHistory', 'RenderAlloc', 'TimeScaleSystem Update',
           'PlayerModelHudFollower', 'ScaleformRTTStream', 'TemporalAa', 'PrepareSrgbOverlay', 'GlobalWorldCBuffer',
           'm_ActiveViewCtxPos', 'uisystemmenu']
an = {}
locs = {}
for a in anchors:
    hits = p.find_all(a.encode() + b'\0', limit=10)
    rv = [p.rva_of(h) for h in hits if p.m[h - 1] == 0]
    locs[a] = rv
targets = {r for v in locs.values() for r in v}
refs = p.rel32_refs(targets) if targets else {}
for a, rv in locs.items():
    fs = []
    for r in rv:
        for x in refs.get(r, []):
            f = p.func_at(x)
            fs.append(dict(ref=hex(x), func=hex(f[0]) if f else None))
    an[a] = dict(string_rvas=[hex(r) for r in rv], refs=fs[:12])
o['anchors'] = an
json.dump(o, open(sys.argv[2], 'w'), indent=1)
print(json.dumps({k: o[k] for k in ('version', 'sha256', 'size_of_image', 'protection_strings', 'rtti_type_count')}, indent=1))
print([ (s['name'], s['entropy']) for s in secs])
print(list(o['imports'].keys()))
print(o['feature_strings']['counts'])
