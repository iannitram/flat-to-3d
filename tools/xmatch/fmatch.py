# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""Cross-binary function matching: Remastered (A) -> Spider-Man 2 (B).

Candidates come from masked byte windows (rel32 / RIP displacements wildcarded)
and from shared string references; each candidate is scored by the similarity of
its instruction-shape sequence (mnemonic + registers, numbers blanked) to A's
function. Aligned instruction pairs then map callees and RIP-relative data refs,
so one confident match yields its callees and globals.
"""
import re, difflib
from collections import Counter, defaultdict
from capstone import Cs, CS_ARCH_X86, CS_MODE_64, CS_OP_MEM, CS_OP_IMM
from capstone.x86 import X86_REG_RIP

md = Cs(CS_ARCH_X86, CS_MODE_64)
md.detail = True
NUM = re.compile(r'0x[0-9a-f]+|\b\d+\b')
BR = {'call', 'jmp'} | {'j' + c for c in ('a ae b be e ne g ge l le o no p np s ns rcxz ecxz'.split())}


class Fn:
    """One function's instructions, masked bytes, tokens, call and data refs."""

    def __init__(self, pe, rva, maxins=4000):
        self.rva = rva
        b = pe.func_at(rva)
        end = b[1] if b and b[0] == rva else rva + 0x400
        if b and b[0] != rva:  # rva inside a function (chunk): use to its end
            end = b[1]
        code = pe.bytes(rva, min(end - rva, 0x10000))
        self.ins = []
        for i in md.disasm(code, rva):
            self.ins.append(i)
            if len(self.ins) >= maxins:
                break
        self.tokens, self.calls, self.rips, self.masked = [], {}, {}, []
        for k, i in enumerate(self.ins):
            raw = bytearray(i.bytes)
            mask = bytearray(b'\xff' * len(raw))
            tgt = None
            if i.mnemonic in BR and i.operands and i.operands[0].type == CS_OP_IMM:
                tgt = i.operands[0].imm
                isz = i.imm_size or 0
                if isz:
                    for j in range(i.imm_offset, i.imm_offset + isz):
                        mask[j] = 0
                if i.mnemonic == 'call':
                    self.calls[k] = tgt
            for op in i.operands:
                if op.type == CS_OP_MEM and op.mem.base == X86_REG_RIP:
                    t = i.address + i.size + op.mem.disp
                    self.rips[k] = t
                    for j in range(i.disp_offset, i.disp_offset + 4):
                        mask[j] = 0
            self.masked.append((bytes(raw), bytes(mask)))
            tok = i.mnemonic + ' ' + NUM.sub('#', i.op_str)
            self.tokens.append(tok)
        self.offs = [i.address - rva for i in self.ins]

    def pattern(self, k0, k1):
        b = b''.join(x[0] for x in self.masked[k0:k1])
        m = b''.join(x[1] for x in self.masked[k0:k1])
        return b, m


def anchor(b, m):
    best, cur, start, bs = 0, 0, 0, 0
    for j, x in enumerate(m):
        if x:
            if cur == 0:
                start = j
            cur += 1
            if cur > best:
                best, bs = cur, start
        else:
            cur = 0
    return bs, best


def search(pe, b, m, limit=64, cap=4000):
    """File-offset-free: returns RVAs in .text where masked pattern matches."""
    tva, traw, tsz = pe.text()
    a0, alen = anchor(b, m)
    if alen < 4:
        return None
    needle = b[a0:a0 + alen]
    mm = pe.m
    out, seen = [], 0
    p = mm.find(needle, traw, traw + tsz)
    while p != -1:
        seen += 1
        if seen > cap:
            return None  # too common
        s = p - a0
        if s >= traw:
            chunk = mm[s:s + len(b)]
            if all((c & k) == (x & k) for c, x, k in zip(chunk, b, m)):
                out.append(tva + s - traw)
                if len(out) > limit:
                    return out
        p = mm.find(needle, p + 1, traw + tsz)
    return out


def similarity(fa, fb):
    sm = difflib.SequenceMatcher(None, fa.tokens, fb.tokens, autojunk=False)
    return sm.ratio(), sm


def candidates(A, B, fa, win=6, step=3, maxwin=60):
    votes = Counter()
    n = len(fa.ins)
    wins = list(range(0, max(1, n - win), step))
    if len(wins) > maxwin:
        stride = len(wins) / maxwin
        wins = [wins[int(i * stride)] for i in range(maxwin)]
    for k in wins:
        b, m = fa.pattern(k, min(n, k + win))
        if sum(1 for x in m if x) < 12:
            continue
        hits = search(B, b, m, limit=20)
        if not hits:
            continue
        w = 1.0 / len(hits)
        for h in hits:
            f = B.func_at(h)
            if f:
                votes[f[0] + 0] += w if (h - f[0]) >= 0 else 0
    return votes


def string_refs(pe, fn):
    out = {}
    for k, t in fn.rips.items():
        s = pe.cstr(t)
        if s and len(s) >= 5:
            out[k] = s
    return out


def best_match(A, B, rva, extra=(), topn=6):
    fa = Fn(A, rva)
    votes = candidates(A, B, fa)
    for e in extra:
        votes[e] += 0.5
    scored = []
    for c, v in votes.most_common(topn):
        fb = Fn(B, c)
        r, sm = similarity(fa, fb)
        scored.append((r, v, c, fb, sm))
    scored.sort(key=lambda x: -x[0])
    return fa, scored


def aligned(fa, fb, sm):
    """Instruction index pairs from equal blocks."""
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == 'equal' or (tag == 'replace' and i2 - i1 == j2 - j1):
            for d in range(i2 - i1):
                yield i1 + d, j1 + d


def propagate(fa, fb, sm):
    calls, rips = defaultdict(Counter), defaultdict(Counter)
    for i, j in aligned(fa, fb, sm):
        if i in fa.calls and j in fb.calls:
            calls[fa.calls[i]][fb.calls[j]] += 1
        if i in fa.rips and j in fb.rips:
            rips[fa.rips[i]][fb.rips[j]] += 1
    return calls, rips


def unique_sig(pe, fn, k0=0, maxins=40):
    """Shortest masked pattern starting at instruction k0 that is unique in .text."""
    for k1 in range(k0 + 3, min(len(fn.ins), k0 + maxins) + 1):
        b, m = fn.pattern(k0, k1)
        if sum(1 for x in m if x) < 8:
            continue
        hits = search(pe, b, m, limit=2)
        if hits is not None and len(hits) == 1:
            return ' '.join('%02x' % x if k else '??' for x, k in zip(b, m)), fn.offs[k0]
    return None, None
