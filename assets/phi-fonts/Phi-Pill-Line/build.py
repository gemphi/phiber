"""What `line.py` writes: the .otf files, a mock drawn straight from the
geometry (before any font exists), and a specimen set in the built fonts.
The dial's turns are `dial.tsv`; the colours are the sheet's two inks."""
import math
import os

from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.boundsPen import BoundsPen

from line import HERE, shape

INK, PAPER = '#16161a', '#e9e9e4'


def file_name(family, style):
    return family.replace(' ', '').replace('-', '') + '-' + style + '.otf'


def drawn(known, glyph, mode):
    """Every glyph the table has, recorded: name -> (pen, advance, char)."""
    out = {}
    for name in glyph:
        pen = RecordingPen()
        advance, char = shape(name, glyph, known, mode, pen)
        out[name] = (pen, advance, char)
    return out


def build(known, face, glyph):
    from fontTools.fontBuilder import FontBuilder
    from fontTools.pens.t2CharStringPen import T2CharStringPen
    from line import table
    source = table()[3]['PhiPill-Regular']
    for family, mode, style, weight in face:
        made = drawn(known, glyph, mode)
        order = ['.notdef'] + list(made)
        empty = round(known['w'] + 2 * known['side'])
        strings, metric = {}, {'.notdef': (empty, 0)}
        blank = T2CharStringPen(empty, None)
        strings['.notdef'] = blank.getCharString()
        for name, (pen, advance, _) in made.items():
            t2, box = T2CharStringPen(round(advance), None), BoundsPen(None)
            pen.replay(t2)
            pen.replay(box)
            strings[name] = t2.getCharString()
            metric[name] = (round(advance), round(box.bounds[0]) if box.bounds else 0)
        ps = family.replace(' ', '').replace('-', '') + '-' + style
        fb = FontBuilder(1000, isTTF=False)
        fb.setupGlyphOrder(order)
        fb.setupCharacterMap({char: name for name, (_, _, char) in made.items() if char is not None})
        fb.setupCFF(ps, {'FullName': family + ' ' + style}, strings, {})
        fb.setupHorizontalMetrics(metric)
        hhea, os2 = source['hhea'], source['OS/2']
        fb.setupHorizontalHeader(ascent=hhea.ascent, descent=hhea.descent, lineGap=hhea.lineGap)
        fb.setupNameTable({
            'familyName': family + ('' if style == 'Regular' else ' ' + style),
            'styleName': 'Regular',
            'typographicFamily': family, 'typographicSubfamily': style,
            'uniqueFontIdentifier': ps + '-1.0', 'fullName': family + ' ' + style,
            'psName': ps, 'version': 'Version 1.000'})
        fb.setupOS2(usWeightClass=int(weight), sTypoAscender=os2.sTypoAscender,
                    sTypoDescender=os2.sTypoDescender, sTypoLineGap=os2.sTypoLineGap,
                    usWinAscent=os2.usWinAscent, usWinDescent=os2.usWinDescent,
                    sCapHeight=round(known['h']), sxHeight=round(known['xh']), fsSelection=0x40)
        fb.setupPost()
        path = os.path.join(HERE, file_name(family, style))
        fb.save(path)
        print('wrote', os.path.basename(path), len(made), 'glyphs')


def dial():
    """Each hour and how far its numeral turns, with the clock, in degrees."""
    turn = []
    for line in open(os.path.join(HERE, 'dial.tsv'), encoding='utf-8'):
        cell = line.rstrip('\n').split('\t')
        if not line.startswith('#') and len(cell) == 2:
            turn.append((cell[0], float(cell[1])))
    return turn


def polygons(pen, scale, at, turn=0.0):
    """A recorded outline as point lists on the page (curves cut fine)."""
    cos, sin = math.cos(math.radians(turn)), math.sin(math.radians(turn))
    place = lambda p: (at[0] + (p[0] * cos + p[1] * sin) * scale, at[1] + (p[0] * sin - p[1] * cos) * scale)
    out, cur = [], []
    for op, args in pen.value:
        match op:
            case 'moveTo':
                cur = [args[0]]
            case 'lineTo':
                cur.append(args[0])
            case 'curveTo':
                p0, (p1, p2, p3) = cur[-1], args
                for k in range(1, 13):
                    t = k / 12
                    cur.append(tuple((1 - t) ** 3 * a + 3 * (1 - t) ** 2 * t * b + 3 * (1 - t) * t * t * c + t ** 3 * d
                                     for a, b, c, d in zip(p0, p1, p2, p3)))
            case 'closePath':
                out.append([place(p) for p in cur])
    return out


def paint(page, shapes):
    """Fill outlines even-odd onto a page: each one flips what it covers."""
    from PIL import Image, ImageChops, ImageDraw
    mask = Image.new('1', page.size, 0)
    for one in shapes:
        layer = Image.new('1', page.size, 0)
        ImageDraw.Draw(layer).polygon(one, fill=1)
        mask = ImageChops.logical_xor(mask, layer)
    page.paste(Image.new('RGB', page.size, INK), (0, 0), mask.convert('L'))


def mock(known, face, glyph, out, size=150, fine=3):
    """The glyphs as the geometry draws them, a row a face, and a dial."""
    from PIL import Image
    gap, em = size // 2, size * fine / 1000
    rows = [[n for n in glyph if n != 'space']]
    width = int(sum(drawn(known, glyph, 'solid')[n][1] for n in rows[0]) * em) + gap * fine * 2
    ring = size * fine * 2.6
    page = Image.new('RGB', (max(width, int(ring * 2 + gap * fine * 2)), int(size * fine * 1.4 * len(face) + ring * 2 + gap * fine * 3)), PAPER)
    y = gap * fine + size * fine
    for family, mode, _, _ in face:
        made, x, shapes = drawn(known, glyph, mode), gap * fine, []
        for name in rows[0]:
            shapes += polygons(made[name][0], em, (x, y))
            x += made[name][1] * em
        paint(page, shapes)
        y += size * fine * 1.4
    cx, cy = page.size[0] / 2, y + ring - size * fine * 0.4
    line, solid = drawn(known, glyph, 'line'), drawn(known, glyph, 'solid')
    shapes, names = [], 'zero one two three four five six seven eight nine'.split()
    for hour, turn in dial():
        made = line if int(hour) % 3 else solid
        figure = [names[int(d)] for d in hour]
        wide = sum(made[n][1] for n in figure) * em * 0.6
        a = math.radians(int(hour) * 30)
        px, py = cx + math.sin(a) * ring * 0.82, cy - math.cos(a) * ring * 0.78
        tc, ts = math.cos(math.radians(turn)), math.sin(math.radians(turn))
        x0, y0 = -wide / 2, known['h'] * em * 0.6 / 2
        for n in figure:
            sx, sy = px + x0 * tc - y0 * ts, py + x0 * ts + y0 * tc
            shapes += polygons(made[n][0], em * 0.6, (sx, sy), turn)
            x0 += made[n][1] * em * 0.6
    paint(page, shapes)
    page.resize((page.size[0] // fine, page.size[1] // fine), Image.LANCZOS).save(out)
    print('drew', out)


def sheet(face, out):
    """A specimen set in the built files, through FreeType as apps set it."""
    from PIL import Image, ImageDraw, ImageFont
    page = Image.new('RGB', (1800, 1400), PAPER)
    pen = ImageDraw.Draw(page)
    label = ImageFont.truetype(os.path.join(HERE, '..', 'Phi-Pill', 'PhiPill-Light.otf'), 26)
    y = 40
    for family, _, style, _ in face:
        font = lambda size: ImageFont.truetype(os.path.join(HERE, file_name(family, style)), size)
        pen.text((60, y), family.lower() + ' - ' + style.lower(), font=label, fill=INK)
        pen.text((60, y + 40), '0123456789', font=font(150), fill=INK)
        pen.text((60, y + 230), '10:09  27.11  12-6', font=font(80), fill=INK)
        y += 360
    for family, _, style, _ in face:
        font = ImageFont.truetype(os.path.join(HERE, file_name(family, style)), 64)
        for k, words in enumerate(('mon tue wed thu fri sat sun', 'jan feb mar apr may jun jul aug', 'sep oct nov dec  am pm')):
            pen.text((60, y + 60 + k * 80), words, font=font, fill=INK)
        y += 280
    line = ImageFont.truetype(os.path.join(HERE, file_name(*[(f, s) for f, m, s, _ in face if m == 'line'][0])), 64)
    cx, cy, ring = 1400, 280 + 360, 330
    pen.ellipse((cx - ring, cy - ring, cx + ring, cy + ring), outline=INK, width=3)
    for hour, turn in dial():
        a = math.radians(int(hour) * 30)
        box = pen.textbbox((0, 0), hour, font=line)
        tile = Image.new('L', (box[2] + 40, box[3] + 40), 0)
        ImageDraw.Draw(tile).text((20 - box[0], 20 - box[1] + 0), hour, font=line, fill=255)
        tile = tile.rotate(-turn, expand=True, resample=Image.BICUBIC)
        px, py = cx + math.sin(a) * ring * 0.8, cy - math.cos(a) * ring * 0.8
        page.paste(Image.new('RGB', tile.size, INK), (int(px - tile.size[0] / 2), int(py - tile.size[1] / 2)), tile)
    page.save(out)
    print('drew', out)
