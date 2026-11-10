# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""Minimal mmap-backed PE32+ reader for offline analysis (read-only, low memory)."""
import mmap, struct, bisect, hashlib
import numpy as np


class PE:
    def __init__(self, path):
        self.path = path
        self.f = open(path, 'rb')
        self.m = mmap.mmap(self.f.fileno(), 0, access=mmap.ACCESS_READ)
        m = self.m
        pe = struct.unpack_from('<I', m, 0x3c)[0]
        assert m[pe:pe + 4] == b'PE\0\0'
        nsec = struct.unpack_from('<H', m, pe + 6)[0]
        optsz = struct.unpack_from('<H', m, pe + 20)[0]
        opt = pe + 24
        self.image_base = struct.unpack_from('<Q', m, opt + 24)[0]
        self.size_of_image = struct.unpack_from('<I', m, opt + 56)[0]
        self.timestamp = struct.unpack_from('<I', m, pe + 8)[0]
        dd = opt + 112
        self.dirs = [struct.unpack_from('<II', m, dd + 8 * i) for i in range(16)]
        self.sections = []
        s = opt + optsz
        for i in range(nsec):
            name = m[s:s + 8].rstrip(b'\0').decode('latin1')
            vsize, va, rawsize, rawptr = struct.unpack_from('<IIII', m, s + 8)
            ch = struct.unpack_from('<I', m, s + 36)[0]
            self.sections.append(dict(name=name, va=va, vsize=vsize, raw=rawptr, rawsize=rawsize, ch=ch))
            s += 40
        prva, psize = self.dirs[3]
        self.funcs = np.frombuffer(self.bytes(prva, psize), dtype='<u4').reshape(-1, 3)[:, :2].copy() if psize else None
        self.starts = self.funcs[:, 0] if self.funcs is not None else None
        self._text = None

    def sec(self, name):
        return next(s for s in self.sections if s['name'] == name)

    def off(self, rva):
        for s in self.sections:
            if s['va'] <= rva < s['va'] + max(s['vsize'], s['rawsize']):
                o = rva - s['va']
                if o < s['rawsize']:
                    return s['raw'] + o
                return None
        return None

    def rva_of(self, off):
        for s in self.sections:
            if s['raw'] <= off < s['raw'] + s['rawsize']:
                return s['va'] + off - s['raw']
        return None

    def bytes(self, rva, n):
        o = self.off(rva)
        if o is None:
            return None
        return bytes(self.m[o:o + n])

    def exec_rva(self, rva):
        return any(s['va'] <= rva < s['va'] + s['vsize'] and s['ch'] & 0x20000000 for s in self.sections)

    def func_at(self, rva):
        """(begin, end) of the .pdata entry containing rva, or None."""
        i = int(np.searchsorted(self.starts, rva, side='right')) - 1
        if i >= 0 and rva < int(self.funcs[i, 1]):
            return int(self.funcs[i, 0]), int(self.funcs[i, 1])
        return None

    def is_start(self, rva):
        i = int(np.searchsorted(self.starts, rva))
        return i < len(self.starts) and int(self.starts[i]) == rva

    def text(self):
        if self._text is None:
            t = self.sec('.text')
            self._text = (t['va'], t['raw'], min(t['vsize'], t['rawsize']))
        return self._text

    def sha256(self):
        h = hashlib.sha256()
        h.update(self.m)
        return h.hexdigest()

    def cstr(self, rva, maxlen=200):
        b = self.bytes(rva, maxlen)
        if not b:
            return None
        e = b.find(b'\0')
        if e < 4:
            return None
        s = b[:e]
        if all(32 <= c < 127 for c in s):
            return s.decode()
        return None

    def find_all(self, needle, limit=1000, start=0, end=None):
        out = []
        p = self.m.find(needle, start, end if end is not None else len(self.m))
        while p != -1 and len(out) < limit:
            out.append(p)
            p = self.m.find(needle, p + 1, end if end is not None else len(self.m))
        return out

    def rel32_refs(self, targets, chunk=8 << 20):
        """All .text positions whose rel32 (pos..pos+4) resolves to any of targets (rva set).
        Returns {target: [rva_of_disp_field, ...]}. Caller verifies the instruction."""
        tva, traw, tsz = self.text()
        tg = np.array(sorted(targets), dtype=np.int64)
        res = {int(t): [] for t in tg}
        for c0 in range(0, tsz - 4, chunk):
            n = min(chunk + 3, tsz - c0)
            a = np.frombuffer(self.m, dtype=np.uint8, count=n, offset=traw + c0).astype(np.int64)
            d = a[:-3] | (a[1:-2] << 8) | (a[2:-1] << 16) | (a[3:] << 24)
            d = np.where(d >= 1 << 31, d - (1 << 32), d)
            pos = np.arange(len(d), dtype=np.int64) + (tva + c0)
            tgt = pos + 4 + d
            del d, a
            hit = np.isin(tgt, tg)
            for i in np.nonzero(hit)[0]:
                res[int(tgt[i])].append(int(pos[i]))
            del tgt, pos, hit
        return res
