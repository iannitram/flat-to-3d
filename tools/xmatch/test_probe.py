import os
import struct
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
try:
    import numpy as np
    import probe
except ImportError:  # capstone and numpy are analysis-only dependencies
    probe = None


class FakePE:
    """In-memory stand-in for pe.PE: file offset == rva, .text is m[traw:traw+tsz]."""

    def __init__(self, m, funcs=(), traw=0, tsz=None):
        self.m = m
        self.traw, self.tsz = traw, len(m) - traw if tsz is None else tsz
        self.funcs = np.array(funcs or [(0, 0)], dtype=np.int64).reshape(-1, 2)
        self.starts = self.funcs[:, 0]

    def text(self):
        return self.traw, self.traw, self.tsz

    def rva_of(self, off):
        return off

    def bytes(self, rva, n):
        return bytes(self.m[rva:rva + n]) if 0 <= rva < len(self.m) else None

    def func_at(self, rva):
        i = int(np.searchsorted(self.starts, rva, side='right')) - 1
        if i >= 0 and rva < int(self.funcs[i, 1]):
            return int(self.funcs[i, 0]), int(self.funcs[i, 1])
        return None

    def rel32_refs(self, targets):
        res = {int(t): [] for t in targets}
        for pos in range(self.traw, self.traw + self.tsz - 3):
            t = pos + 4 + struct.unpack_from('<i', self.m, pos)[0]
            if t in res:
                res[t].append(pos)
        return res


def use(fake):
    probe.p = fake
    probe.TVA, probe.TRAW, probe.TSZ = fake.text()
    probe._bounds.clear()


def call(site, target):
    return b'\xe8' + struct.pack('<i', target - (site + 5))


@unittest.skipIf(probe is None, "capstone or numpy is not installed")
class Xrefs(unittest.TestCase):
    T = 0x100

    def sites(self, code, funcs):
        use(FakePE(code + b'\xcc' * (self.T + 0x10 - len(code)), funcs))
        return [s for s, *_ in probe.xrefs([self.T])[self.T]]

    def test_call_after_a_rex_byte_is_reported_at_its_real_start(self):
        # mov [rsp+0x40], rax ends in 0x40, which also decodes as a REX prefix on the call.
        code = b'\x48\x89\x44\x24\x40' + call(5, self.T) + b'\xc3'
        self.assertEqual(self.sites(code, [(0, len(code))]), [5])

    def test_call_bytes_inside_an_immediate_are_not_a_reference(self):
        # mov rax, imm64 whose immediate holds e8 + rel32 to the target.
        imm = call(2, self.T) + b'\x90\x90\x90'
        code = b'\x48\xb8' + imm + b'\xc3'
        self.assertEqual(self.sites(code, [(0, len(code))]), [])

    def test_leaf_code_between_pdata_entries_decodes_from_the_gap_start(self):
        # entry [0, 4) then a leaf at 4 with no .pdata: nop; mov [rsp+0x40], rax; call
        code = b'\x90\x90\x90\xc3' + b'\x90' + b'\x48\x89\x44\x24\x40' + call(10, self.T) + b'\xc3'
        self.assertEqual(self.sites(code, [(0, 4), (self.T, self.T + 1)]), [10])

    def test_rip_relative_reference_is_kept(self):
        # lea rcx, [rip+disp] -> target
        code = b'\x48\x8d\x0d' + struct.pack('<i', self.T - 7) + b'\xc3'
        self.assertEqual(self.sites(code, [(0, len(code))]), [0])


@unittest.skipIf(probe is None, "capstone or numpy is not installed")
class CountPattern(unittest.TestCase):
    def count(self, pat, m, traw, tsz):
        use(FakePE(m, traw=traw, tsz=tsz))
        return probe.count_pattern(*probe.parse_pattern(pat), limit=10)

    def test_match_inside_text_counts(self):
        self.assertEqual(self.count('41??43', b'XABCY', 1, 3), [1])

    def test_leading_wildcard_before_text_is_rejected(self):
        self.assertEqual(self.count('??414243', b'XABCY', 1, 3), [])

    def test_trailing_wildcard_past_text_is_rejected(self):
        self.assertEqual(self.count('414243??', b'XABCY', 1, 3), [])

    def test_truncated_slice_at_end_of_file_is_rejected(self):
        self.assertEqual(self.count('414243??', b'ABC', 0, 3), [])


if __name__ == '__main__':
    unittest.main()
