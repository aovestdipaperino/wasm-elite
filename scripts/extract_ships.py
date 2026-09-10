#!/usr/bin/env python3
"""Transcribe ship blueprints from Mark Moxon's annotated BBC Micro Elite (disc) disassembly.

Usage: extract_ships.py <path-to-moxon-checkout> > crates/elite-ships/src/blueprints.rs
"""
import re, glob, sys
root = sys.argv[1] + '/1-source-files/main-sources/'
ships = {}
for f in sorted(glob.glob(root + 'elite-ships-*.asm')) + [root + 'elite-missile.asm']:
    txt = open(f).read()
    for m in re.finditer(r'^\.SHIP_([A-Z0-9_]+)\n(.*?)(?=^\.SHIP_(?![A-Z0-9_]*_(?:VERTICES|EDGES|FACES)\b)|\Z)', txt, re.S | re.M):
        name, body = m.group(1), m.group(2)
        if name.endswith(('_VERTICES','_EDGES','_FACES')): continue
        if name in ships: continue
        hdr = {}
        lines = body.split('\n')
        def grab(pat):
            mm = re.search(pat, body); return mm
        g = grab(r'EQUB\s+(\d+)(?:\s*\+\s*\((\d+)\s*<<\s*4\))?\s*\\ Max\. canisters'); hdr['canisters']=int(g.group(1)); hdr['scoop']=int(g.group(2)) if g.group(2) else 0
        g = grab(r'EQUW\s+(\d+)\s*\*\s*\1\s*\\ Targetable'); hdr['area']=int(g.group(1))
        g = grab(r'\\ Max\. edge count\s*=\s*\((\d+) - 1\) / 4 = (\d+)'); hdr['max_edges']=int(g.group(2))
        g = grab(r'EQUB\s+(\d+)\s*\\ Gun vertex'); hdr['gun_vertex']=int(g.group(1))//4 if False else int(g.group(1))
        g = grab(r'Explosion count\s*=\s*(\d+)'); hdr['explosion']=int(g.group(1))
        g = grab(r'EQUW\s+(\d+)\s*\\ Bounty'); hdr['bounty']=int(g.group(1))
        g = grab(r'EQUB\s+(\d+)\s*\\ Visibility distance'); hdr['vis']=int(g.group(1))
        g = grab(r'EQUB\s+(\d+)\s*\\ Max\. energy'); hdr['energy']=int(g.group(1))
        g = grab(r'EQUB\s+(\d+)\s*\\ Max\. speed'); hdr['speed']=int(g.group(1))
        g = grab(r'Normals are scaled by\s*=\s*2\^(\d+)'); hdr['scale']=int(g.group(1))
        g = grab(r'Laser power\s*=\s*(\d+)'); hdr['laser']=int(g.group(1))
        g = grab(r'Missiles\s*=\s*(\d+)'); hdr['missiles']=int(g.group(1))
        verts = [tuple(int(x) for x in re.findall(r'-?\d+', l.split('\\')[0])) for l in lines if l.strip().startswith('VERTEX')]
        edges = [tuple(int(x) for x in re.findall(r'-?\d+', l.split('\\')[0])) for l in lines if l.strip().startswith('EDGE ')]
        faces = [tuple(int(x) for x in re.findall(r'-?\d+', l.split('\\')[0])) for l in lines if l.strip().startswith('FACE')]
        hdr['gun_vertex'] = hdr['gun_vertex']//4
        ships[name]=(hdr,verts,edges,faces)
BORROW={'SPLINTER':'ESCAPE_POD','THARGON':'CANISTER'}
for k,src in BORROW.items():
    h,v,e,f=ships[k]; ships[k]=(h,v,ships[src][2],f)
def ident(n): return ''.join(p.capitalize() for p in n.split('_'))
out = ["// Generated from Mark Moxon's annotated disassembly of BBC Micro Elite (disc version).",
       "// Ship designs are copyright of their owners; this is data transcription for a fan reimplementation.",
       "use crate::{Blueprint, Edge, Face, Vertex};", ""]
for name,(h,v,e,f) in ships.items():
    out.append(f"pub const {name}: Blueprint = Blueprint {{")
    out.append(f"    name: \"{ident(name)}\",")
    for k in ['canisters','scoop','area','max_edges','gun_vertex','explosion','bounty','vis','energy','speed','scale','laser','missiles']:
        out.append(f"    {k}: {h[k]},")
    out.append("    vertices: &[")
    for x,y,z,f1,f2,f3,f4,vis in v: out.append(f"        Vertex {{ x: {x}, y: {y}, z: {z}, faces: [{f1}, {f2}, {f3}, {f4}], vis: {vis} }},")
    out.append("    ],\n    edges: &[")
    for a,b,f1,f2,vis in e: out.append(f"        Edge {{ v1: {a}, v2: {b}, faces: [{f1}, {f2}], vis: {vis} }},")
    out.append("    ],\n    faces: &[")
    for x,y,z,vis in f: out.append(f"        Face {{ nx: {x}, ny: {y}, nz: {z}, vis: {vis} }},")
    out.append("    ],\n};\n")
out.append("pub const ALL: &[&Blueprint] = &[" + ", ".join(f"&{n}" for n in ships) + "];")
sys.stdout.write('\n'.join(out) + '\n')
for n,(h,v,e,f) in ships.items(): print(n, len(v), len(e), len(f), file=sys.stderr)
