"""Make "Phi-Pill Open" from Phi-Pill Light: the same letters, the space inside them wider.

    python pillopen.py SRC.otf DST.otf [GROW [colour]]

GROW    font units every closed space inside a letter grows by (64 by
        default; Phi-Pill keeps the space between two stems at 129)
colour  keep Phi-Pill's 3D side as a colour font; without it the font is
        one colour, the letter alone, so text takes the colour it is set in

Only the spaces inside a letter change, and only across: every point right
of a space moves right by GROW (a point inside the space by its share), so
each stroke, round, fillet and height stays exactly as Phi-Pill draws it and
the letter gets GROW wider for each space it has. A space counts when it goes
on for TALL units up or down; where two strokes only pass each other, nothing
grows. The light pill inside each stroke is left out: it belongs to Phi-Pill.
It needs fontTools and pyclipper.
"""
import sys
from fontTools.ttLib import TTFont
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.t2CharStringPen import T2CharStringPen
from outline import bounds, merge, runs, swept

src, dst = sys.argv[1], sys.argv[2]
grow = float(sys.argv[3]) if len(sys.argv) > 3 else 64.0
colour = len(sys.argv) > 4 and sys.argv[4] == "colour"
family = "Phi-Pill Open"
SIDE = (45, -45)  # Phi-Pill's extrusion
STEP, TALL = 2, 60  # rows every 2 units; a space shorter than 60 is two strokes passing

font = TTFont(src)
glyphs = font.getGlyphSet()
cff = font["CFF "].cff
top = cff.topDictIndex[0]
strings = top.CharStrings
metrics = font["hmtx"].metrics
layers = font["COLR"].ColorLayers


def polygons(name):
    """The glyph's contours as point lists; Phi-Pill draws only straight lines."""
    pen = DecomposingRecordingPen(glyphs)
    glyphs[name].draw(pen)
    out = []
    for op, args in pen.value:
        match op:
            case "moveTo":
                out.append([args[0]])
            case "lineTo":
                out[-1].append(args[0])
            case "closePath" | "endPath":
                pass
            case _:
                raise SystemExit(f"{name}: {op} - only straight outlines are handled")
    return out


def charstring(name, polys, advance):
    original = strings[name]
    private = original.private
    pen = T2CharStringPen(width=advance - private.nominalWidthX, glyphSet=None)
    for p in polys:
        pen.moveTo(p[0])
        for pt in p[1:]:
            pen.lineTo(pt)
        pen.closePath()
    strings[name] = pen.getCharString(private=private, globalSubrs=original.globalSubrs)


def spaces(polys):
    """Each row's closed spaces, (left, right) between two runs of ink,
    keeping only those that go on for TALL units up or down."""
    y0, y1 = bounds(polys)[1], bounds(polys)[3]
    rows = {}
    for k in range(int(y0) - STEP, int(y1) + STEP, STEP):
        line = runs(polys, k + 0.25)
        rows[k] = [(a[1], b[0]) for a, b in zip(line, line[1:]) if b[0] - a[1] >= 4]
    kept = {}
    for k, found in rows.items():
        kept[k] = []
        for lo, hi in found:
            tall = 0
            for way in (-STEP, STEP):
                j = k + way
                while any(a < hi and b > lo for a, b in rows.get(j, [])):
                    tall, j = tall + STEP, j + way
            match tall >= TALL:
                case True:
                    kept[k].append((lo, hi))
                case False:
                    pass
    return kept


def shift(rows, x, y):
    """How far right the point at (x, y) moves: GROW for each space left of
    it, a share while inside one. A row with no space (the top of an arch)
    blends the nearest rows above and below that have one."""
    k = int(y // STEP) * STEP
    near = []
    for way in (-STEP, STEP):
        j = k
        while j in rows and not rows[j]:
            j += way
        match j in rows:
            case True:
                near.append((abs(j - k) + 1, rows[j]))
            case False:
                pass
    total, moved = 0.0, 0.0
    for dist, found in near:
        for lo, hi in found:
            mid, half = (lo + hi) / 2, max((hi - lo) / 2, 30)
            moved += grow * min(max((x - mid + half) / (2 * half), 0), 1) / dist
        total += 1 / dist
    return moved / total if total else 0.0


for base, stack in layers.items():
    names = {layer.name.rsplit(".", 1)[-1]: layer.name for layer in stack}
    body = merge(polygons(names["body"]))
    was = bounds(body)
    advance, lsb = metrics[base]
    rows = spaces(body)
    new_body = [[(x + shift(rows, x, y), y) for x, y in c] for c in body]
    new_advance = round(advance + (bounds(new_body)[2] - was[2]))
    parts = {"body": new_body, "side": swept(new_body, SIDE), "pill": []}
    for part, polys in parts.items():
        match part in names:
            case True:
                charstring(names[part], polys, new_advance)
                ink = bounds(polys)
                metrics[names[part]] = (new_advance, round(ink[0]) if ink else 0)
            case False:
                pass
    stack[:] = [layer for layer in stack if not layer.name.endswith(".pill")]
    # The plain glyph, all a one-colour font draws: the letter alone.
    charstring(base, new_body, new_advance)
    metrics[base] = (new_advance, lsb)

match colour:
    case True:
        pass
    case False:
        del font["COLR"]
        del font["CPAL"]
font["hhea"].advanceWidthMax = max(advance for advance, _ in metrics.values())
font["OS/2"].recalcAvgCharWidth(font)

# A family of its own, beside Phi-Pill rather than over it; Light, as its source.
style = "Light"
postscript = f"{family}-{style}".replace(" ", "")
names = {
    1: f"{family} {style}",
    2: "Regular",
    3: f"{postscript};pillopen {grow:g}",
    4: f"{family} {style}",
    6: postscript,
    16: family,
    17: style,
}
table = font["name"]
for record in list(table.names):
    match record.nameID in names:
        case True:
            table.removeNames(nameID=record.nameID)
        case False:
            pass
for name_id, text in names.items():
    table.setName(text, name_id, 3, 1, 0x409)
    table.setName(text, name_id, 1, 0, 0)
font["OS/2"].usWeightClass = 300
cff.fontNames = [postscript]
top.FullName = names[4]
top.FamilyName = family
font.save(dst)
print(f"{dst}: the space inside +{grow:g}, '{names[4]}', {'colour' if colour else 'one colour'}")
