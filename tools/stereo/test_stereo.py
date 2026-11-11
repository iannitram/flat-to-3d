import pathlib
import random
import struct
import sys
import tempfile
import unittest
import zlib
import subprocess

ROOT=pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/"tools/stereo"))
import fps,disparity

class StereoTests(unittest.TestCase):
    def test_fps_mangohud(self):
        with tempfile.TemporaryDirectory() as td:
            p=pathlib.Path(td,"x.csv");p.write_text("os,cpu\nLinux,0\nfps,frametime\n60,10\n60,20\n25,40\n")
            rows=fps.read_mangohud(p);r=fps.summarize(rows);self.assertEqual(r["p50"],60);self.assertEqual(r["p1"],25)

    def test_fps_cli_threshold_and_bad_header(self):
        with tempfile.TemporaryDirectory() as td:
            p=pathlib.Path(td,"fps.csv");p.write_text("os,cpu\nLinux,0\nfps,frametime\n60,10\n60,10\n")
            self.assertEqual(fps.main([str(p),"--min","30"]),0)
            self.assertEqual(fps.main([str(p),"--min","61"]),1)
            p.write_text("no-data\n")
            self.assertEqual(fps.main([str(p)]),2)

    def _png(self,w,h,pixels,ctype=2,filters=None,interlace=0):
        channels={0:1,2:3}[ctype]; rows=[]; prev=bytearray(w*channels)
        for y in range(h):
            raw=bytearray(pixels[y*w*channels:(y+1)*w*channels]);f=(filters or [0]*h)[y]; encoded=bytearray(raw)
            for x in range(len(raw)):
                a=raw[x-channels] if x>=channels else 0;b=prev[x];c=prev[x-channels] if x>=channels else 0
                if f==1:encoded[x]=(raw[x]-a)&255
                elif f==2:encoded[x]=(raw[x]-b)&255
                elif f==3:encoded[x]=(raw[x]-(a+b)//2)&255
                elif f==4:
                    p0=a+b-c;pa=abs(p0-a);pb=abs(p0-b);pc=abs(p0-c);pr=a if pa<=pb and pa<=pc else b if pb<=pc else c;encoded[x]=(raw[x]-pr)&255
            rows.append(bytes([f])+encoded);prev=raw
        def chunk(k,b):return struct.pack(">I",len(b))+k+b+struct.pack(">I",zlib.crc32(k+b)&0xffffffff)
        return b"\x89PNG\r\n\x1a\n"+chunk(b"IHDR",struct.pack(">IIBBBBB",w,h,8,ctype,0,0,interlace))+chunk(b"IDAT",zlib.compress(b"".join(rows)))+chunk(b"IEND",b"")

    def test_png_filters_disparity_and_failures(self):
        w,h=64,32;rng=random.Random(1);left=[rng.randrange(256) for _ in range(w//2*h)];right=[]
        for y in range(h): right.extend(left[y*32+4:y*32+32]+[0]*4)
        rgb=bytearray()
        for y in range(h):
            for v in left[y*32:(y+1)*32]+right[y*32:(y+1)*32]:rgb.extend((v,v,v))
        with tempfile.TemporaryDirectory() as td:
            p=pathlib.Path(td,"img.png");p.write_bytes(self._png(w,h,rgb,2,[0,1,2,3]+[4]*(h-4)));W,H,l=disparity.decode(p);self.assertEqual((W,H),(w,h));m=disparity.measure(W,H,l,"sbs",8,8,8,1);self.assertAlmostEqual(m["mean"],4,delta=1)
            same=bytearray()
            for y in range(h):
                for v in left[y*32:(y+1)*32]*2:same.extend((v,v,v))
            ident=pathlib.Path(td,"identical.png");ident.write_bytes(self._png(w,h,same));iw,ih,il=disparity.decode(ident);im=disparity.measure(iw,ih,il,"sbs",8,8,8,1);self.assertEqual(im["mean"],0)
            self.assertEqual(disparity.main([str(ident),"--block","8","--step","8","--max-shift","8","--min-texture","1"]),1)
            gray=pathlib.Path(td,"gray.png");gray.write_bytes(self._png(8,5,bytes(range(40)),0,[0,1,2,3,4]));self.assertEqual(disparity.decode(gray),(8,5,bytearray(range(40))))
            p.write_bytes(self._png(4,2,bytes(range(8)),0,interlace=1))
            with self.assertRaisesRegex(ValueError,"interlaced"):disparity.decode(p)
            p.write_bytes(b"\x89PNG\r\n\x1a\n\x00\x00")
            with self.assertRaises(ValueError):disparity.decode(p)

    def test_ppm_decode(self):
        with tempfile.TemporaryDirectory() as td:
            p=pathlib.Path(td,"a.ppm");p.write_bytes(b"P6\n1 1\n255\n\xff\x00\x00");self.assertEqual(disparity.decode(p),(1,1,bytearray([76])))
            space=pathlib.Path(td,"space.ppm");space.write_bytes(b"P6\n1 1\n255\n\x20\x00\x00");self.assertEqual(disparity.decode(space)[2],bytearray([9]))

if __name__=="__main__":unittest.main()
