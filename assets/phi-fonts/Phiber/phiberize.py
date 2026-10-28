"""Make "Phiber", a more open interface face, from Phi-Soft.

    python phiberize.py SRC.otf DST.otf WIDTH TRACK STYLE [FAMILY [RATIO]]

WIDTH  horizontal scale of every glyph (1.08 = 8% wider)
TRACK  letter spacing added to every advance, in font units (split either
       side of the glyph, so letters move apart evenly)
STYLE  the style name written into the name table (Light, Regular, ...)

The outlines are redrawn through a transform (CFF), advances and side
bearings follow, kerning (GPOS pair adjustments) scales with the width, and
the family is renamed so it registers beside Phi-Soft instead of over it.
"""
import sys
from fontTools.ttLib import TTFont
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.boundsPen import BoundsPen

src, dst, width, track, style = sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4]), sys.argv[5]
family = sys.argv[6] if len(sys.argv) > 6 else "Phiber"
# Optional: thin the vertical strokes until a stem is RATIO times as thick as
# a horizontal stroke (Phi-Soft draws its stems twice as thick as its bars,
# which is what crowds the counters). Horizontal strokes keep their thickness.
ratio = float(sys.argv[7]) if len(sys.argv) > 7 else None

font = TTFont(src, recalcBBoxes=True)
cff = font["CFF "].cff
top = cff.topDictIndex[0]
strings = top.CharStrings
glyphs = font.getGlyphSet()
metrics = font["hmtx"].metrics


def outline(name):
    """The glyph's contours as point lists [(x, y, on_curve)], composites
    resolved, and the pen operations to replay them."""
    pen = DecomposingRecordingPen(glyphs)
    glyphs[name].draw(pen)
    return pen.value


def edges(ops, vertical):
    """x of the vertical (or y of the horizontal) straight edges."""
    found, cur, start = set(), None, None
    for op, args in ops:
        match op:
            case "moveTo":
                cur = start = args[0]
            case "lineTo":
                (ax, ay), (bx, by) = cur, args[0]
                hit = abs(ax - bx) < 2 and abs(ay - by) > 150 if vertical else abs(ay - by) < 2 and abs(ax - bx) > 60
                if hit:
                    found.add(round(ax if vertical else ay))
                cur = args[0]
            case "curveTo" | "qCurveTo":
                cur = args[-1]
            case _:
                pass
    return sorted(found)


def ink_run(ops, across):
    """The first run of ink a straight cut meets: across=True cuts horizontally
    a quarter of the way up (a stem's thickness), False cuts vertically through the
    arms, near the right end (a horizontal stroke's thickness). Curves are
    flattened, so it works however the letter is drawn."""
    lines, cur, start = [], None, None
    for op, args in ops:
        match op:
            case "moveTo":
                cur = start = args[0]
            case "lineTo":
                lines.append((cur, args[0])); cur = args[0]
            case "curveTo":
                (x0, y0), (x1, y1), (x2, y2), (x3, y3) = cur, *args
                prev = cur
                for i in range(1, 17):
                    t = i / 16; u = 1 - t
                    pt = (u**3*x0 + 3*u*u*t*x1 + 3*u*t*t*x2 + t**3*x3, u**3*y0 + 3*u*u*t*y1 + 3*u*t*t*y2 + t**3*y3)
                    lines.append((prev, pt)); prev = pt
                cur = args[-1]
            case "closePath" | "endPath":
                if cur != start:
                    lines.append((cur, start))
                cur = start
            case _:
                cur = args[-1] if args else cur
    xs = [c for seg in lines for c in (seg[0][0], seg[1][0])]
    ys = [c for seg in lines for c in (seg[0][1], seg[1][1])]
    k = 1 if across else 0
    # A quarter of the way up the H, below its bar: its left stem alone. (The
    # I would do in Light, but Phi-Soft draws it stylised in the heavier weights.)
    at = min(ys) + (max(ys) - min(ys)) * 0.25 if across else max(xs) - (max(xs) - min(xs)) * 0.15
    hits = []
    for a, b in lines:
        lo, hi = sorted((a[k], b[k]))
        if lo <= at < hi and hi > lo:
            f = (at - a[k]) / (b[k] - a[k])
            hits.append(a[1 - k] + f * (b[1 - k] - a[1 - k]))
    hits.sort()
    return hits[1] - hits[0]


def flatten(ops):
    """The outline as straight segments, curves split in 16."""
    lines, cur, start = [], None, None
    for op, args in ops:
        match op:
            case "moveTo":
                cur = start = args[0]
            case "lineTo":
                lines.append((cur, args[0])); cur = args[0]
            case "curveTo":
                (x0, y0), (x1, y1), (x2, y2), (x3, y3) = cur, *args
                prev = cur
                for i in range(1, 17):
                    t = i / 16; u = 1 - t
                    pt = (u**3*x0 + 3*u*u*t*x1 + 3*u*t*t*x2 + t**3*x3, u**3*y0 + 3*u*u*t*y1 + 3*u*t*t*y2 + t**3*y3)
                    lines.append((prev, pt)); prev = pt
                cur = args[-1]
            case "closePath" | "endPath":
                if cur != start:
                    lines.append((cur, start))
                cur = start
            case _:
                pass
    return lines


def run_at(lines, x, y):
    """The width of the run of ink, along the horizontal through (x, y), that
    the point sits on; None when the scanline only grazes the outline there."""
    hits = sorted(a[0] + (y - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) for a, b in lines if min(a[1], b[1]) <= y < max(a[1], b[1]))
    for left, right in zip(hits[0::2], hits[1::2]):
        if left - 1.5 <= x <= right + 1.5:
            return right - left
    return None


def inset(ops, d, floor):
    """Move every point sideways into the ink by up to d: a vertical edge moves
    the full d, a horizontal one not at all, curves and diagonals in between.
    No stroke is thinned below `floor` (a horizontal stroke's thickness): a
    point on a run narrower than floor + 2d moves only what keeps that much
    ink, so a narrow diagonal (the middle of an M) never turns inside out."""
    lines = flatten(ops)
    contours, current = [], []
    for op, args in ops:
        match op:
            case "moveTo":
                current = [(op, [args[0]])]
                contours.append(current)
            case "closePath" | "endPath":
                current.append((op, []))
            case _:
                current.append((op, list(args)))
    flat = [[p for _, pts in c for p in pts] for c in contours]
    area = lambda pts: sum(pts[i - 1][0] * pts[i][1] - pts[i][0] * pts[i - 1][1] for i in range(len(pts))) / 2
    outer = max(flat, key=lambda pts: abs(area(pts)), default=[])
    ink_left = area(outer) > 0 if outer else True
    moved = []
    for contour, pts in zip(contours, flat):
        n = len(pts)
        def normal_x(a, b):
            tx, ty = b[0] - a[0], b[1] - a[1]
            length = (tx * tx + ty * ty) ** 0.5
            if length < 1e-6:
                return 0.0
            left_x = -ty / length
            return left_x if ink_left else -left_x
        shifted = []
        for i, p in enumerate(pts):
            before, after = normal_x(pts[i - 1], p), normal_x(p, pts[(i + 1) % n])
            nx = before if abs(before) >= abs(after) else after
            run = run_at(lines, p[0], p[1])
            room = d if run is None else max(0.0, min(d, (run - floor) / 2))
            shifted.append((p[0] + room * nx, p[1]))
        k = 0
        for op, pts_ in contour:
            moved.append((op, shifted[k:k + len(pts_)]))
            k += len(pts_)
    return moved


thin = 0.0
bar = 0.0
if ratio is not None:
    stem = ink_run(outline("H"), across=True)
    bar = ink_run(outline("E"), across=False)
    # The ratio is of the finished face: widening scales the stems too, so
    # thin them to RATIO x bar / WIDTH before the widening brings them back.
    thin = max(0.0, (stem - ratio * bar / width) / 2)
    print(f"stem {stem}, bar {bar}: each side of a stem moves in {thin:.1f} (finished stem {(stem - 2 * thin) * width:.0f} = {ratio} x {bar})")

redrawn = {}
for name in font.getGlyphOrder():
    advance, lsb = metrics[name]
    new_advance = round(advance * width + track)
    original = strings[name]
    private = original.private
    pen = T2CharStringPen(width=new_advance - private.nominalWidthX, glyphSet=glyphs)
    target = TransformPen(pen, (width, 0, 0, 1, track / 2, 0))
    ops = outline(name)
    if thin:
        ops = inset(ops, thin, bar)
    bounds = BoundsPen(None)
    measured = TransformPen(bounds, (width, 0, 0, 1, track / 2, 0))
    for op, args in ops:
        getattr(target, op)(*args)
        getattr(measured, op)(*args)
    redrawn[name] = pen.getCharString(private=private, globalSubrs=original.globalSubrs)
    # The left side bearing is where the drawn outline now starts.
    left = bounds.bounds[0] if bounds.bounds else lsb * width + track / 2
    metrics[name] = (new_advance, round(left))
for name, charstring in redrawn.items():
    strings[name] = charstring

# Kerning scales with the glyphs it sits between.
def scale_value(record):
    if record is not None and getattr(record, "XAdvance", None):
        record.XAdvance = round(record.XAdvance * width)

if "GPOS" in font:
    for lookup in font["GPOS"].table.LookupList.Lookup:
        for sub in lookup.SubTable:
            pair = sub.ExtSubTable if lookup.LookupType == 9 else sub
            match getattr(pair, "Format", None), type(pair).__name__:
                case 1, "PairPos":
                    for pair_set in pair.PairSet:
                        for record in pair_set.PairValueRecord:
                            scale_value(getattr(record, "Value1", None))
                            scale_value(getattr(record, "Value2", None))
                case 2, "PairPos":
                    for class1 in pair.Class1Record:
                        for class2 in class1.Class2Record:
                            scale_value(getattr(class2, "Value1", None))
                            scale_value(getattr(class2, "Value2", None))
                case _:
                    pass

font["hhea"].advanceWidthMax = max(advance for advance, _ in metrics.values())
font["OS/2"].recalcAvgCharWidth(font)

# A family of its own, beside Phi-Soft rather than over it.
ribbi = style in ("Regular", "Bold")
postscript = f"{family}-{style}".replace(" ", "")
names = {
    1: family if ribbi else f"{family} {style}",
    2: style if ribbi else "Regular",
    3: f"{postscript};phiberize {width} {track:g}",
    4: f"{family} {style}",
    6: postscript,
    16: family,
    17: style,
}
table = font["name"]
for record in list(table.names):
    if record.nameID in names:
        table.removeNames(nameID=record.nameID)
for name_id, text in names.items():
    table.setName(text, name_id, 3, 1, 0x409)
    table.setName(text, name_id, 1, 0, 0)
cff.fontNames = [postscript]
top.FullName = names[4]
top.FamilyName = family
font.save(dst)
print(f"{dst}: width x{width}, tracking +{track:g}, '{names[4]}'")
