# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""Map every Spidy stereo-path item from Remastered (A) into Spider-Man 2 (B).
usage: run_match.py A.exe B.exe out.json [maxprio]"""
import sys, json, time, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from collections import Counter, defaultdict
from pe import PE
import fmatch, vt
from targets import T, hexbytes

A, B = PE(sys.argv[1]), PE(sys.argv[2])
outp = sys.argv[3]
maxprio = int(sys.argv[4]) if len(sys.argv) > 4 else 3
t0 = time.time()
pairs = {}          # A func rva -> dict(b, ratio, second, how)
prop_calls = defaultdict(Counter)
prop_rips = defaultdict(Counter)
cache = {}


def conf(r, second):
    if r >= 0.85 and r - second >= 0.08:
        return 'high'
    if r >= 0.65 and r - second >= 0.03:
        return 'medium'
    return 'low'


def do_match(rva, how='windows', extra=()):
    if rva in pairs:
        return pairs[rva]
    fa, sc = fmatch.best_match(A, B, rva, extra=extra)
    if not sc:
        pairs[rva] = dict(b=None, ratio=0, second=0, how=how, nA=len(fa.ins))
        return pairs[rva]
    sc.sort(key=lambda x: (-x[0], -x[1]))
    r, v, c, fb, sm = sc[0]
    second = sc[1][0] if len(sc) > 1 else 0
    res = dict(b=c, ratio=round(r, 3), second=round(second, 3), votes=round(v, 2), how=how, nA=len(fa.ins), nB=len(fb.ins))
    res['conf'] = conf(r, second)
    if r >= 0.6:
        calls, rips = fmatch.propagate(fa, fb, sm)
        for k, cn in calls.items():
            prop_calls[k].update(cn)
        for k, cn in rips.items():
            prop_rips[k].update(cn)
        # sites inside A's function -> B instruction address
        res['_align'] = {fa.ins[i].address: fb.ins[j].address for i, j in fmatch.aligned(fa, fb, sm)}
    cache[rva] = (fa, fb)
    pairs[rva] = res
    return res


items = [t for t in T if t[3] <= maxprio]
funcs = [t for t in items if t[2] in ('hook', 'call', 'check', 'gate') and A.exec_rva(t[1])]
# containing functions of call sites / mid-function checks
site_parent = {}
for name, rva, kind, prio, hx in items:
    if kind in ('site', 'check') and A.exec_rva(rva):
        f = A.func_at(rva)
        if f:
            site_parent[rva] = f[0]
work = sorted({t[1] if A.is_start(t[1]) else A.func_at(t[1])[0] for t in funcs if A.func_at(t[1])} | set(site_parent.values()))
print('matching', len(work), 'functions', flush=True)
for rva in work:
    r = do_match(rva)
    print(hex(rva), '->', hex(r['b']) if r['b'] else None, r.get('ratio'), r.get('second'), r.get('conf'), '%.0fs' % (time.time() - t0), flush=True)

# second pass: propagate callees for weak/unmatched functions
for rva in work:
    r = pairs[rva]
    if rva in prop_calls and r.get('conf') != 'high':
        cand, n = prop_calls[rva].most_common(1)[0]
        fa = fmatch.Fn(A, rva); fb = fmatch.Fn(B, cand)
        ratio, sm = fmatch.similarity(fa, fb)
        if ratio > r.get('ratio', 0) or cand == r.get('b'):
            r.update(b=cand, ratio=round(ratio, 3), how=r['how'] + '+callgraph(%d)' % n,
                     conf='high' if ratio >= 0.7 and cand == r.get('b') else conf(ratio, 0 if cand == r.get('b') else r.get('second', 0)))
            if cand == r.get('b') and r['conf'] == 'low':
                r['conf'] = 'medium'

# data: find A functions referencing each global, match them, propagate rips
datas = [t for t in items if t[2] == 'data' and not A.exec_rva(t[1])]
refs = A.rel32_refs({t[1] for t in datas})
for name, rva, kind, prio, hx in datas:
    if rva in prop_rips:
        continue
    fs = []
    for p in refs.get(rva, [])[:40]:
        f = A.func_at(p)
        if f and f[0] not in fs:
            fs.append(f[0])
    for f in fs[:3]:
        do_match(f, how='xref-of-global')
        if rva in prop_rips:
            break

out = []
for name, rva, kind, prio, hx in items:
    e = dict(name=name, rem=hex(rva), kind=kind, prio=prio)
    if kind == 'vtable':
        nm = vt.vtable_name(A, rva)
        e['rtti'] = nm[0] if nm else None
        if nm:
            vs = [v for v in vt.vtables_for(B, nm[0]) if v[1] == nm[1]]
            e['sm2'] = [hex(v[0]) for v in vs]
            e['conf'] = 'high' if len(vs) == 1 else ('medium' if vs else 'none')
            e['how'] = 'RTTI type name'
    elif kind == 'data' or (not A.exec_rva(rva)):
        if hx:
            want = hexbytes(hx)
            hits = B.find_all(want, limit=5)
            e['sm2_bytes_hits'] = [hex(B.rva_of(h)) for h in hits]
        c = prop_rips.get(rva)
        if c:
            (b, n), = c.most_common(1)
            tot = sum(c.values())
            e['sm2'] = hex(b); e['how'] = 'RIP ref in aligned code (%d/%d refs agree)' % (n, tot)
            e['conf'] = 'high' if n >= 2 and n == tot else ('medium' if n / tot >= 0.6 else 'low')
            e['alts'] = {hex(k): v for k, v in c.items()}
        else:
            e['conf'] = 'none'
    elif kind in ('site', 'check') and rva in site_parent and not A.is_start(rva):
        par = site_parent[rva]
        r = pairs.get(par, {})
        e['parent_rem'] = hex(par); e['parent_sm2'] = hex(r['b']) if r.get('b') else None
        e['sm2'] = hex(r['_align'][rva]) if r.get('_align') and rva in r['_align'] else None
        # return address: the instruction after a call
        if e['sm2'] is None and r.get('_align'):
            prev = max((a for a in r['_align'] if a < rva), default=None)
            if prev is not None:
                e['note'] = 'not aligned; nearest aligned A %s -> B %s' % (hex(prev), hex(r['_align'][prev]))
        e['conf'] = r.get('conf', 'none') if e['sm2'] else 'none'
        e['how'] = 'aligned instruction in matched parent'
    else:
        key = rva if A.is_start(rva) else A.func_at(rva)[0]
        r = pairs.get(key, {})
        e.update({k: v for k, v in r.items() if not k.startswith('_')})
        e['sm2'] = hex(r['b']) if r.get('b') else None
        if rva in prop_calls:
            e['callgraph'] = {hex(k): v for k, v in prop_calls[rva].most_common(3)}
    if hx and e.get('sm2') and isinstance(e['sm2'], str):
        want = hexbytes(hx)
        e['spidy_bytes_same_in_sm2'] = B.bytes(int(e['sm2'], 16), len(want)) == want
    out.append(e)

# signatures for matched functions / sites
for e in out:
    s = e.get('sm2')
    if not isinstance(s, str) or e['kind'] in ('data', 'vtable') or e.get('conf') in (None, 'none'):
        continue
    sv = int(s, 16)
    fb = fmatch.Fn(B, sv, maxins=60)
    sig, off = fmatch.unique_sig(B, fb, 0, maxins=40)
    e['sig'] = sig
    if e['kind'] == 'site' and sig is None:
        # anchor a few instructions earlier
        f = B.func_at(sv)
        if f:
            fp = fmatch.Fn(B, f[0])
            k = next((i for i, x in enumerate(fp.ins) if x.address == sv), None)
            if k:
                sig, off = fmatch.unique_sig(B, fp, max(0, k - 6), maxins=40)
                e['sig'] = sig; e['sig_offset_to_item'] = sv - (f[0] + off) if sig else None

json.dump(dict(A=sys.argv[1], B=sys.argv[2], items=out, secs=round(time.time() - t0)), open(outp, 'w'), indent=1)
c = Counter(e.get('conf', 'none') for e in out)
print('done', dict(c), '%.0fs' % (time.time() - t0))
