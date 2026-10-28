"""Make "Phiber", a more open interface face, from Phi-Soft.

    python -I phiberize.py SRC.otf DST.otf WIDTH TRACK STYLE [FAMILY]

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

src, dst, width, track, style = sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4]), sys.argv[5]
family = sys.argv[6] if len(sys.argv) > 6 else "Phiber"

font = TTFont(src, recalcBBoxes=True)
cff = font["CFF "].cff
top = cff.topDictIndex[0]
strings = top.CharStrings
glyphs = font.getGlyphSet()
metrics = font["hmtx"].metrics

redrawn = {}
for name in font.getGlyphOrder():
    advance, lsb = metrics[name]
    new_advance = round(advance * width + track)
    original = strings[name]
    private = original.private
    pen = T2CharStringPen(width=new_advance - private.nominalWidthX, glyphSet=glyphs)
    glyphs[name].draw(TransformPen(pen, (width, 0, 0, 1, track / 2, 0)))
    redrawn[name] = pen.getCharString(private=private, globalSubrs=original.globalSubrs)
    metrics[name] = (new_advance, round(lsb * width + track / 2))
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
