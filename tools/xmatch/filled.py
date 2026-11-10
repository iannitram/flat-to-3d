# Flat to 3D (GPL-3.0). Offline, read-only analysis of game executables on disk.
"""Fraction of 1 MiB blocks of a (Steam-preallocated) file that are not all zero."""
import sys, os
for p in sys.argv[1:]:
    n = os.path.getsize(p); bs = 1 << 20; z = t = 0
    with open(p, 'rb') as f:
        while True:
            b = f.read(bs)
            if not b:
                break
            t += 1
            if b.count(0) == len(b):
                z += 1
    print('%s: %d/%d MiB blocks nonzero' % (os.path.basename(p), t - z, t))
