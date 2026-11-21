"""Outline geometry for pillopen.py: merging, measuring rows, the 3D sweep.

Outlines are lists of contours, each a list of (x, y) in font units, filled
by the non-zero rule with the ink on the left of each contour's direction.
"""
import pyclipper

SCALE = 16  # pyclipper works in integers: 1/16 of a font unit


def _to_clipper(polys):
    return [[(round(x * SCALE), round(y * SCALE)) for x, y in p] for p in polys]


def _from_clipper(paths):
    return [[(x / SCALE, y / SCALE) for x, y in p] for p in paths]


def merge(polys, fill=pyclipper.PFT_NONZERO):
    """One clean shape: overlapping pieces joined (the arms of a V), loops
    dropped, every contour turned so the ink is on its left."""
    match polys:
        case []:
            return []
        case _:
            pass
    tool = pyclipper.Pyclipper()
    tool.AddPaths(_to_clipper(polys), pyclipper.PT_SUBJECT, True)
    return _from_clipper(tool.Execute(pyclipper.CT_UNION, fill, fill))


def bounds(polys):
    xs = [x for p in polys for x, _ in p]
    ys = [y for p in polys for _, y in p]
    match xs:
        case []:
            return None
        case _:
            return (min(xs), min(ys), max(xs), max(ys))


def runs(polys, y):
    """The runs of ink along the horizontal at y, as (left, right) pairs."""
    hits = sorted(
        a[0] + (y - a[1]) / (b[1] - a[1]) * (b[0] - a[0])
        for p in polys for a, b in zip(p, p[1:] + p[:1])
        if min(a[1], b[1]) <= y < max(a[1], b[1])
    )
    return list(zip(hits[0::2], hits[1::2]))


def swept(polys, by):
    """The shape and everything it passes through on its way to `by`: the 3D side."""
    tool = pyclipper.Pyclipper()
    path = [(0, 0), (by[0] * SCALE, by[1] * SCALE)]
    for p in _to_clipper(polys):
        tool.AddPath(p, pyclipper.PT_SUBJECT, True)
        tool.AddPaths(pyclipper.MinkowskiSum(path, p, True), pyclipper.PT_SUBJECT, True)
    return _from_clipper(tool.Execute(pyclipper.CT_UNION, pyclipper.PFT_NONZERO, pyclipper.PFT_NONZERO))
