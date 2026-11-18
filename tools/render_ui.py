#!/usr/bin/env python3
"""Render NyashkChannel's faceplate (see hwrender.py). Run from the repo root."""
import os, sys
sys.path.insert(0, os.path.dirname(__file__))
from hwrender import *

ACC = (255, 190, 70)
INK = (232, 232, 228, 235); DIM = (150, 152, 156, 225)
p = Panel(1000, 654, (0.115, 0.12, 0.13), ACC, INK, DIM)
header(p, 'CHANNEL', 'CONSOLE CHANNEL STRIP', x0=36)
p.button('power', 930, 32, 64, 22, 'POWER')
RED = (0.80, 0.12, 0.10); GRN = (0.10, 0.58, 0.26); BLU = (0.12, 0.34, 0.80); BRN = (0.45, 0.27, 0.12); BLK = (0.12, 0.12, 0.12); WHT = (0.88, 0.88, 0.86)
G_ = (60, 220, 110)
def sec(x0, y0, x1, y1, title, col=None):
    p.inset(x0, y0, x1, y1, r=8)
    p.text(title, (x0 + x1) / 2, y0 + 12, 7, col or DIM, True, sp=1.5)
def kn(id, x, y, cap, lbl, D=46, bip=False):
    p.knob(id, x, y, D, 'cap', cap=cap, label=lbl, label_dy=D * 0.5 + 16, bip=bip,
           scale=dict(r=D * 0.62, n=11, tick=3.5), led=dict(r=D * 0.53, n=15))
# ---- left column
sec(30, 60, 330, 170, 'INPUT')
p.button('phase', 62, 100, 40, 20, 'Ø'); p.button('pad', 62, 130, 40, 20, 'PAD')
kn('line', 140, 112, WHT, 'LINE', bip=True); kn('mic', 212, 112, RED, 'MIC')
p.button('analog', 288, 112, 62, 22, 'ANALOG')
sec(30, 178, 330, 300, 'FILTERS')
p.button('hp_on', 62, 220, 40, 20, 'HP'); p.button('lp_on', 62, 250, 40, 20, 'LP')
kn('hp_f', 140, 234, BLK, 'HP FREQ'); kn('lp_f', 212, 234, BLK, 'LP FREQ')
p.button('split', 288, 234, 56, 22, 'SPLIT')
sec(30, 308, 330, 452, 'EXPANDER / GATE', (120, 210, 150, 230))
p.button('x_on', 62, 350, 44, 20, 'ON', G_); p.button('x_gate', 62, 378, 44, 20, 'GATE', G_); p.button('x_fast', 62, 406, 44, 20, 'FAST', G_)
kn('x_thr', 140, 380, GRN, 'THRESH'); kn('x_range', 212, 380, GRN, 'RANGE'); kn('x_rel', 284, 380, GRN, 'RELEASE')
sec(30, 460, 330, 636, 'COMPRESSOR')
p.button('c_on', 62, 502, 44, 20, 'ON', (255, 80, 64)); p.button('c_fast', 62, 530, 44, 20, 'FAST'); p.button('c_auto', 62, 558, 44, 20, 'AUTO')
kn('c_thr', 140, 530, WHT, 'THRESH'); kn('c_ratio', 212, 530, WHT, 'RATIO'); kn('c_rel', 284, 530, WHT, 'RELEASE')
p.button('dyn_post', 180, 612, 220, 20, 'DYNAMICS AFTER EQ')
# ---- centre: graph + EQ
p.screen('graph', 348, 64, 420, 212)
p.button('v_eq', 376, 292, 40, 20, 'EQ'); p.button('v_dyn', 422, 292, 44, 20, 'DYN')
p.button('eq_on', 560, 292, 56, 20, 'EQ IN', (255, 80, 64))
p.button('brown', 668, 292, 52, 20, 'BROWN'); p.button('black', 726, 292, 52, 20, 'BLACK')
sec(342, 312, 774, 636, 'EQUALISER')
bands = [('LF', BRN, (230, 150, 70)), ('LMF', BLU, (80, 150, 255)), ('HMF', GRN, (60, 220, 110)), ('HF', RED, (255, 80, 64))]
for i, (nm, cap, acc) in enumerate(bands):
    x = 396 + i * 108
    p.text(nm, x, 340, 10, tuple(acc) + (255,), True)
    kn(nm.lower() + '_g', x, 392, cap, 'GAIN', 54, True)
    kn(nm.lower() + '_f', x, 482, cap, 'FREQ', 48)
    if nm in ('LMF', 'HMF'): kn(nm.lower() + '_q', x, 568, BLK, 'Q', 44)
    else: p.button(nm.lower() + '_bell', x, 568, 56, 22, 'BELL')
# ---- right: meters + output
sec(784, 60, 970, 636, 'METERS')
lad = [(0.0, (60, 220, 110)), (0.75, (255, 190, 50)), (0.9, (255, 70, 60))]
gr = [(0.0, ACC)]
for id, x, cols in [('in', 812, lad), ('out', 840, lad), ('exp', 884, [(0.0, (60, 220, 110))]), ('comp', 912, gr)]:
    p.ladder(id, x, 86, 16, 250, 26, cols)
for t, x in [('IN', 820), ('OUT', 848), ('EXP', 892), ('COMP', 920)]: p.text(t, x, 348, 6.5, None, True)
p.readout('gr_exp', 892, 364, size=9); p.readout('gr_comp', 920, 364, size=9)
p.text('OUTPUT', 877, 396, 7, None, True, sp=1.5)
kn('out', 834, 450, WHT, 'OUTPUT', 50, True); kn('width', 920, 450, BLK, 'WIDTH', 50)
p.button('mono', 877, 560, 70, 22, 'MONO')
p.export(os.path.join(os.path.dirname(__file__), '..'), 'plugins/nyashkchannel')
