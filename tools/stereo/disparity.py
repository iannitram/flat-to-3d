#!/usr/bin/env python3
"""Estimate horizontal disparity in side-by-side or top-and-bottom images."""
import argparse
import json
import math
import re
import struct
import sys
import zlib
from pathlib import Path


def _ppm(data):
    i = 0
    def token():
        nonlocal i
        while i < len(data):
            if data[i] == 35:
                while i < len(data) and data[i] not in (10, 13): i += 1
            elif data[i] in b" \t\r\n": i += 1
            else: break
        start = i
        while i < len(data) and data[i] not in b" \t\r\n#": i += 1
        return data[start:i]
    magic = token()
    if magic not in (b"P5", b"P6"): raise ValueError("unsupported PPM/PGM")
    w, h, maxval = int(token()), int(token()), int(token())
    if maxval > 255 or min(w, h) <= 0: raise ValueError("invalid PPM dimensions or maxval")
    if i >= len(data) or data[i] not in b" \t\r\n": raise ValueError("missing PPM data separator")
    if data[i:i+2] == b"\r\n": i += 2
    else: i += 1
    channels = 3 if magic == b"P6" else 1
    raw = data[i:]
    if len(raw) != w*h*channels: raise ValueError("truncated PPM")
    if channels == 1: return w, h, bytearray(raw)
    return w, h, bytearray((299*raw[j]+587*raw[j+1]+114*raw[j+2])//1000 for j in range(0,len(raw),3))


def decode(path):
    data = Path(path).read_bytes()
    if data.startswith((b"P5", b"P6")): return _ppm(data)
    if not data.startswith(b"\x89PNG\r\n\x1a\n"): raise ValueError("unsupported image format")
    pos, w, h, depth, ctype, interlace = 8, 0, 0, 0, 0, 0
    compressed, palette, trans = bytearray(), None, b""
    while pos + 12 <= len(data):
        n = struct.unpack_from(">I", data, pos)[0]; kind = data[pos+4:pos+8]; body = data[pos+8:pos+8+n]
        if len(body) != n or pos+12+n > len(data): raise ValueError("truncated PNG chunk")
        if kind == b"IHDR": w,h,depth,ctype,_,_,interlace = struct.unpack(">IIBBBBB",body)
        elif kind == b"PLTE": palette = body
        elif kind == b"tRNS": trans = body
        elif kind == b"IDAT": compressed.extend(body)
        elif kind == b"IEND": break
        pos += 12+n
    if interlace: raise ValueError("interlaced PNG is unsupported")
    chans = {0:1,2:3,3:1,4:2,6:4}.get(ctype)
    if depth not in (8,16) or not chans or w < 1 or h < 1: raise ValueError("unsupported PNG encoding")
    bpp=chans*(depth//8); stride=w*bpp; raw=zlib.decompress(compressed)
    if len(raw)!=(stride+1)*h: raise ValueError("truncated PNG image data")
    out=bytearray(); prev=bytearray(stride); off=0
    for _ in range(h):
        f=raw[off]; scan=bytearray(raw[off+1:off+1+stride]); off+=stride+1
        for x in range(stride):
            a=scan[x-bpp] if x>=bpp else 0; b=prev[x]; c=prev[x-bpp] if x>=bpp else 0
            if f==1: scan[x]=(scan[x]+a)&255
            elif f==2: scan[x]=(scan[x]+b)&255
            elif f==3: scan[x]=(scan[x]+((a+b)//2))&255
            elif f==4:
                p=a+b-c; pa=abs(p-a);pb=abs(p-b);pc=abs(p-c); scan[x]=(scan[x]+(a if pa<=pb and pa<=pc else b if pb<=pc else c))&255
            elif f!=0: raise ValueError("unknown PNG filter")
        prev=scan
        for x in range(w):
            vals=[scan[x*bpp+j*(depth//8)] for j in range(chans)]
            if ctype==0: lum=vals[0]
            elif ctype==3:
                idx=vals[0]; rgb=palette[idx*3:idx*3+3]
                if len(rgb)!=3: raise ValueError("bad PNG palette")
                lum=(299*rgb[0]+587*rgb[1]+114*rgb[2])//1000
            elif ctype==4:
                lum=vals[0]
            else:
                r,g,b=vals[:3]; lum=(299*r+587*g+114*b)//1000
            out.append(lum)
    return w,h,out


def measure(w,h,luma,layout="sbs",block=16,step=64,max_shift=48,min_texture=20):
    if layout not in ("sbs","tab"): raise ValueError("layout must be sbs or tab")
    if min(w,h)<2 or len(luma)!=w*h: raise ValueError("invalid image dimensions")
    if block<1 or step<1 or max_shift<0 or min_texture<0: raise ValueError("block, step, max-shift and min-texture must be nonnegative, with block and step positive")
    if layout=="sbs": ew,eh=w//2,h; left=lambda x,y:luma[y*w+x]; right=lambda x,y:luma[y*w+x+ew]
    else: ew,eh=w,h//2; left=lambda x,y:luma[y*w+x]; right=lambda x,y:luma[(y+eh)*w+x]
    max_shift=min(max_shift, max(0,(ew-block-1)//2))
    ds=[]
    for y in range(max_shift,eh-block-max_shift+1,step):
      for x in range(max_shift,ew-block-max_shift+1,step):
        vals=[left(x+xx,y+yy) for yy in range(block) for xx in range(block)]
        mean=sum(vals)/len(vals); sd=math.sqrt(sum((v-mean)**2 for v in vals)/len(vals))
        if sd<min_texture: continue
        best=None; cost=None
        for d in range(-max_shift,max_shift+1):
            sad=sum(abs(left(x+xx,y+yy)-right(x+xx+d,y+yy)) for yy in range(block) for xx in range(block))
            if cost is None or sad<cost: cost=sad;best=d
        ds.append(abs(best))
    ordered=sorted(ds)
    return {"n":len(ds),"mean":sum(ds)/len(ds) if ds else 0.0,"p90":ordered[max(0,math.ceil(.9*len(ds))-1)] if ds else 0.0}


def main(argv=None):
    p=argparse.ArgumentParser();p.add_argument("image");p.add_argument("--layout",choices=("sbs","tab"),default="sbs");p.add_argument("--block",type=int,default=16);p.add_argument("--step",type=int,default=64);p.add_argument("--max-shift",type=int,default=48);p.add_argument("--min-texture",type=float,default=20);p.add_argument("--min",type=float,default=1.0);p.add_argument("--json",action="store_true");a=p.parse_args(argv)
    try:
        w,h,l=decode(a.image);r=measure(w,h,l,a.layout,a.block,a.step,a.max_shift,a.min_texture)
    except (OSError,ValueError,struct.error,zlib.error) as e: print(str(e),file=sys.stderr);return 2
    r.update(min=a.min,passed=r["n"]>0 and r["mean"]>a.min)
    print(json.dumps(r,sort_keys=True) if a.json else f"n={r['n']} mean={r['mean']:.2f} p90={r['p90']:.2f} pass={r['passed']}")
    return 0 if r["passed"] else 1
if __name__=="__main__": raise SystemExit(main())
