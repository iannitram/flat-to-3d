# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""Address probe for the hook-point research step (ruling 52): disassembly, code xrefs and
unique masked patterns on one exe. Needs capstone and numpy.

usage: probe.py EXE dis RVA [N]      disassemble N instructions (default 40) with RIP targets labelled
       probe.py EXE callers RVA      every verified call/jmp/RIP reference to RVA, with its function start
       probe.py EXE pattern RVA      the shortest unique masked pattern at RVA (rel32 and RIP disp masked)
       probe.py EXE count PATTERN    hits of a pattern ('??' = any byte) in .text
"""
import sys, os, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from pe import PE
import vt
from capstone import Cs, CS_ARCH_X86, CS_MODE_64, CS_OP_MEM, CS_OP_IMM
from capstone.x86 import X86_REG_RIP

md = Cs(CS_ARCH_X86, CS_MODE_64); md.detail = True
lite = Cs(CS_ARCH_X86, CS_MODE_64)
p = TVA = TRAW = TSZ = None
_bounds = {}


def open_exe(path):
    global p, TVA, TRAW, TSZ
    p = PE(path)
    TVA, TRAW, TSZ = p.text()
    _bounds.clear()
    return p


def region(rva):
    """The linear-decode range holding rva: its .pdata entry, else the gap between two entries."""
    b = p.func_at(rva)
    if b:
        return b
    i = int(np.searchsorted(p.starts, rva, side='right'))
    lo = int(p.funcs[i - 1, 1]) if i > 0 else TVA
    hi = int(p.starts[i]) if i < len(p.starts) else TVA + TSZ
    return lo, hi


def boundaries(rva):
    """Instruction starts of rva's region, decoded linearly from the region start
    (an undecodable byte is stepped over as one byte)."""
    r = region(rva)
    got = _bounds.get(r)
    if got is None:
        lo, hi = r
        code = p.bytes(lo, hi - lo) or b''
        got, off = set(), 0
        while off < len(code):
            n = 0
            for a, size, _, _ in lite.disasm_lite(code[off:], lo + off):
                got.add(a); n += size
            if off + n < len(code):
                n += 1  # step over the undecodable byte
            off += n
        _bounds[r] = got
    return got


def fn(rva):
    return p.func_at(rva)


def fstart(rva):
    """Function start, following chained unwind info to the primary entry."""
    b = p.func_at(rva)
    if not b:
        return None
    i = int(np.searchsorted(p.starts, b[0]))
    # pdata entry: begin, end, unwind
    while True:
        ent = p.bytes(p.dirs[3][0] + 12 * i, 12)
        bg, en, uw = struct.unpack('<III', ent)
        u = p.bytes(uw, 4)
        flags = u[0] >> 3
        if flags & 4:  # UNW_FLAG_CHAININFO
            cnt = u[2]
            off = uw + 4 + ((cnt + 1) & ~1) * 2
            pb = struct.unpack('<I', p.bytes(off, 4))[0]
            i = int(np.searchsorted(p.starts, pb))
            continue
        return bg


def ins(rva, n=None, end=None):
    b = p.func_at(rva)
    if end is None:
        end = b[1] if b else rva + 0x200
    code = p.bytes(rva, end - rva)
    out = []
    for i in md.disasm(code, rva):
        out.append(i)
        if n and len(out) >= n:
            break
    return out


def dis(rva, n=60, end=None):
    for i in ins(rva, n, end):
        extra = ''
        for op in i.operands:
            if op.type == CS_OP_MEM and op.mem.base == X86_REG_RIP:
                t = i.address + i.size + op.mem.disp
                extra = ' ; ->%#x %s' % (t, label(t))
        if i.mnemonic in ('call', 'jmp') and i.operands and i.operands[0].type == CS_OP_IMM:
            extra = ' ; %s' % label(i.operands[0].imm)
        print('%#9x  %-8s %s%s' % (i.address, i.mnemonic, i.op_str, extra))


def label(t):
    s = p.cstr(t, 120)
    if s:
        return repr(s)
    try:
        w = p.bytes(t, 80)
        if w and w[1] == 0 and w[3] == 0:
            ws = w.decode('utf-16le', 'ignore').split('\0')[0]
            if len(ws) > 3 and ws.isprintable():
                return 'L' + repr(ws)
    except Exception:
        pass
    n = vt.vtable_name(p, t)
    if n:
        return 'vt ' + str(n)
    return ''


def strs(needle, limit=50):
    if isinstance(needle, str):
        needle = needle.encode()
    return [p.rva_of(o) for o in p.find_all(needle, limit=limit)]


def xrefs(targets):
    """Verified code refs (call/jmp rel32 and RIP-relative operands) to each target rva; each site
    must be an instruction start in its region's linear decode (see boundaries).
    Returns {target: [(insn_rva, mnemonic, op_str)]}."""
    targets = set(int(t) for t in targets)
    cand = set()
    for d in range(0, 5):
        cand |= {t - d for t in targets}
    raw = p.rel32_refs(cand)
    res = {t: [] for t in targets}
    best = {}
    for tg, poss in raw.items():
        for pos in poss:
            for back in range(1, 4 + 3):
                st = pos - back
                code = p.bytes(st, 16)
                if not code:
                    continue
                try:
                    i = next(md.disasm(code, st))
                except StopIteration:
                    continue
                if i.address + i.size < pos + 4:
                    continue
                tgt = None
                if i.mnemonic in ('call', 'jmp') or i.mnemonic.startswith('j'):
                    if i.operands and i.operands[0].type == CS_OP_IMM and i.imm_offset == back and i.imm_size == 4:
                        tgt = i.operands[0].imm
                else:
                    for op in i.operands:
                        if op.type == CS_OP_MEM and op.mem.base == X86_REG_RIP and i.disp_offset == back:
                            tgt = i.address + i.size + op.mem.disp
                if tgt in targets and p.bytes(st, 1)[0] not in (0x26, 0x2e, 0x36, 0x3e, 0x64, 0x65) \
                        and st in boundaries(st):
                    best[pos] = (st, tgt, i.mnemonic, i.op_str)
    for pos, (st, tgt, mn, op) in best.items():
        res[tgt].append((st, mn, op))
    return res


def callers(t):
    return sorted({(fstart(a) or 0, a) for a, m, o in xrefs([t])[t]})


def masked(rva, n):
    """Bytes of n instructions from rva with rel32/rip disp masked -> (bytes, mask)."""
    raw, mask = bytearray(), bytearray()
    for i in ins(rva, n):
        b = bytearray(i.bytes); m = bytearray(b'\xff' * len(b))
        if (i.mnemonic == 'call' or i.mnemonic.startswith('j')) and i.operands and i.operands[0].type == CS_OP_IMM and i.imm_size == 4:
            for j in range(i.imm_offset, i.imm_offset + 4):
                m[j] = 0
        for op in i.operands:
            if op.type == CS_OP_MEM and op.mem.base == X86_REG_RIP:
                for j in range(i.disp_offset, i.disp_offset + 4):
                    m[j] = 0
        raw += b; mask += m
    return bytes(raw), bytes(mask)


def count_pattern(raw, mask, limit=3):
    """Occurrences of a masked pattern in .text (numpy, anchored on the longest exact run)."""
    # longest unmasked run as anchor
    best, cur, bs = (0, 0), 0, 0
    for k, mm in enumerate(mask + b'\0'):
        if mm == 0xff:
            if cur == 0:
                bs = k
            cur += 1
        else:
            if cur > best[1]:
                best = (bs, cur)
            cur = 0
    a0, alen = best
    anchor = raw[a0:a0 + alen]
    hits = []
    tb = p.m
    start, end = TRAW, TRAW + TSZ
    o = tb.find(anchor, start, end)
    while o != -1:
        s = o - a0
        seg = tb[s:s + len(raw)]
        if s >= start and s + len(raw) <= end and len(seg) == len(raw) and all((x & m) == (y & m) for x, y, m in zip(seg, raw, mask)):
            hits.append(p.rva_of(s))
            if len(hits) >= limit:
                break
        o = tb.find(anchor, o + 1, end)
    return hits


def pat_str(raw, mask):
    return ' '.join('%02x' % b if m == 0xff else '??' for b, m in zip(raw, mask))


def unique_pattern(rva, maxn=40, minn=4):
    for n in range(minn, maxn + 1):
        raw, mask = masked(rva, n)
        if sum(1 for m in mask if m == 0xff) < 8:
            continue
        h = count_pattern(raw, mask, limit=2)
        if h == [rva]:
            return pat_str(raw, mask), n
    return None, None


def parse_pattern(s):
    toks = s.replace(' ', '')
    toks = [toks[i:i + 2] for i in range(0, len(toks), 2)]
    raw = bytes(0 if t == '??' else int(t, 16) for t in toks)
    mask = bytes(0 if t == '??' else 0xff for t in toks)
    return raw, mask


def main(argv):
    if len(argv) < 4:
        print(__doc__, file=sys.stderr)
        return 2
    open_exe(argv[1])
    cmd, arg = argv[2], argv[3]
    if cmd == 'dis':
        dis(int(arg, 16), int(argv[4]) if len(argv) > 4 else 40)
    elif cmd == 'callers':
        for f, a in callers(int(arg, 16)):
            print('%#x in %#x' % (a, f))
    elif cmd == 'pattern':
        pat, n = unique_pattern(int(arg, 16), maxn=60)
        if not pat:
            print('no unique pattern within 60 instructions', file=sys.stderr)
            return 1
        print('%s  (%d instructions, 1 hit in .text)' % (pat.replace(' ', ''), n))
    elif cmd == 'count':
        print(' '.join(hex(h) for h in count_pattern(*parse_pattern(arg), limit=10)))
    else:
        print(__doc__, file=sys.stderr)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
