"""Minimal reader of Igor Pro packed experiment files (.pxp): lists data folders and numeric waves."""
import struct, sys
import numpy as np

NT = {2: "f4", 4: "f8", 8: "i1", 0x10: "i2", 0x20: "i4"}

def _wave(buf, e):
    ver = struct.unpack_from(e + "h", buf, 0)[0]
    if ver == 5:
        npnts = struct.unpack_from(e + "i", buf, 64 + 12)[0]
        typ = struct.unpack_from(e + "h", buf, 64 + 16)[0]
        name = buf[64 + 28:64 + 60].split(b"\0")[0].decode("latin-1")
        ndim = struct.unpack_from(e + "4i", buf, 64 + 68)
        sfa = struct.unpack_from(e + "4d", buf, 64 + 84)
        sfb = struct.unpack_from(e + "4d", buf, 64 + 116)
        dunits = buf[64 + 148:64 + 152].split(b"\0")[0].decode("latin-1")
        xunits = buf[64 + 152:64 + 156].split(b"\0")[0].decode("latin-1")
        off = 64 + 320
        wfm = struct.unpack_from(e + "i", buf, 4)[0]
        note_size = struct.unpack_from(e + "i", buf, 12)[0]
        formula = struct.unpack_from(e + "i", buf, 8)[0]
        note = buf[64 + wfm + formula:64 + wfm + formula + note_size].decode("latin-1", "replace")
    elif ver in (1, 2, 3):
        hdr = {1: 8, 2: 16, 3: 20}[ver]
        typ = struct.unpack_from(e + "h", buf, hdr + 0)[0]
        name = buf[hdr + 6:hdr + 26].split(b"\0")[0].decode("latin-1")
        dunits = buf[hdr + 34:hdr + 38].split(b"\0")[0].decode("latin-1")
        xunits = buf[hdr + 38:hdr + 42].split(b"\0")[0].decode("latin-1")
        npnts = struct.unpack_from(e + "i", buf, hdr + 42)[0]
        hsa, hsb = struct.unpack_from(e + "2d", buf, hdr + 48)
        ndim = (npnts, 0, 0, 0); sfa = (hsa, 1, 1, 1); sfb = (hsb, 0, 0, 0)
        off = hdr + 110
        note = ""
    else:
        return None
    base = typ & ~0x41
    data = None
    if base in NT:
        dt = np.dtype(e + NT[base].replace("i", "u" if typ & 0x40 else "i"))
        n = npnts * (2 if typ & 1 else 1)
        data = np.frombuffer(buf, dtype=dt, count=n, offset=off)
    return dict(name=name, type=typ, npnts=npnts, ndim=ndim, x0=sfb[0], dx=sfa[0], dunits=dunits, xunits=xunits, data=data, note=note)

def read(path):
    raw = open(path, "rb").read()
    e = "<"
    t, v, n = struct.unpack_from("<HhI", raw, 0)
    if n > len(raw):
        e = ">"
    pos, folder, out, recs = 0, [], [], {}
    while pos + 8 <= len(raw):
        t, v, n = struct.unpack_from(e + "HhI", raw, pos)
        body = raw[pos + 8:pos + 8 + n]
        pos += 8 + n
        t &= 0x7FFF
        recs[t] = recs.get(t, 0) + 1
        if t == 9:
            folder.append(body.split(b"\0")[0].decode("latin-1"))
        elif t == 10:
            if folder: folder.pop()
        elif t == 3:
            w = _wave(body, e)
            if w:
                w["folder"] = "/".join(folder)
                out.append(w)
    return out, recs

if __name__ == "__main__":
    waves, recs = read(sys.argv[1])
    show = len(sys.argv) > 2 and sys.argv[2] == "values"
    print("records by type", recs, "waves", len(waves))
    for w in waves:
        d = w["data"]
        rng = ""
        if show and d is not None and len(d):
            f = d[np.isfinite(d.astype("f8"))]
            rng = f" finite={len(f)}"
        print(f"{w['folder']:<40} {w['name']:<32} type={w['type']:<3} n={w['npnts']:<7} dims={w['ndim']} x0={w['x0']:.6g} dx={w['dx']:.6g} units={w['dunits']!r}/{w['xunits']!r}{rng}")
