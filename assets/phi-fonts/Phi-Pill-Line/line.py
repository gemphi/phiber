"""Build Phi-Pill Line and Phi-Pill Link from `glyph.tsv` (needs fontTools).

    python line.py                  writes every face the table names (.otf)
    python line.py --mock out.png   draws the glyphs to look at, no font made
    python line.py --sheet out.png  a specimen of the built faces, with a dial

Every outline is a chain of circles joined by the lines that touch them,
as Phiogy's `script/chain.py` draws (its `touch` and `spot` are used
here): a corner written `x y r` becomes the circle of radius r touching
both its sides. The solid face fills the chain; the line face draws it as
one thin line, the chain with every radius grown by half the line and
again shrunk by half the line. Nothing here is a size or a shape: those
are the table's rows, and the measures are read from Phi-Pill's files."""
import ast
import math
import os
import sys

from fontTools.ttLib import TTFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', '..', '..', '..', 'appkit', 'app', 'phiogy', 'script'))
from chain import touch, spot  # noqa: E402  the house's outline construction


def reckon(text, known):
    """A measure written as + - * / of numbers and known names."""
    def walk(node):
        match node:
            case ast.Expression(body=body):
                return walk(body)
            case ast.Constant(value=value) if isinstance(value, (int, float)):
                return float(value)
            case ast.Name(id=name):
                return known[name]
            case ast.UnaryOp(op=ast.USub(), operand=one):
                return -walk(one)
            case ast.BinOp(left=a, op=op, right=b):
                x, y = walk(a), walk(b)
                match op:
                    case ast.Add(): return x + y
                    case ast.Sub(): return x - y
                    case ast.Mult(): return x * y
                    case ast.Div(): return x / y
        raise ValueError('not a measure: %s' % text)
    return walk(ast.parse(text, mode='eval'))


def cut(font, glyph, axis, at):
    """Where a straight cut across a glyph meets its outline, in order."""
    from fontTools.pens.recordingPen import DecomposingRecordingPen
    rec = DecomposingRecordingPen(font.getGlyphSet())
    font.getGlyphSet()[glyph].draw(rec)
    hit, lead = [], 0 if axis == 'y' else 1
    for start, end in segment(rec.value):
        a, b = start[1 - lead] - at, end[1 - lead] - at
        if a * b < 0:
            hit.append(start[lead] + (end[lead] - start[lead]) * a / (a - b))
    return sorted(hit)


def segment(ops):
    """A recorded outline as straight pieces (curves as their chords)."""
    out, cur, first = [], None, None
    for op, args in ops:
        match op:
            case 'moveTo':
                cur = first = args[0]
            case 'lineTo' | 'curveTo' | 'qCurveTo':
                out.append((cur, args[-1])); cur = args[-1]
            case 'closePath' | 'endPath':
                out.append((cur, first))
    return out


def take(what, fonts):
    from fontTools.pens.boundsPen import BoundsPen
    word = what.split()
    path = os.path.join(HERE, '..', 'Phi-Pill', word[0] + '.otf')
    font = fonts.setdefault(word[0], TTFont(path))
    glyph = font.getBestCmap().get(ord(word[1]), word[1]) if len(word[1]) == 1 else word[1]
    pen = BoundsPen(font.getGlyphSet())
    font.getGlyphSet()[glyph].draw(pen)
    x0, y0, x1, y1 = pen.bounds
    match word[2:]:
        case ['cap']: return float(font['OS/2'].sCapHeight)
        case ['top']: return float(y1)
        case ['bottom']: return float(y0)
        case ['left']: return float(x0)
        case ['wide']: return float(x1 - x0)
        case [axis, at, kind, k]:
            hit = cut(font, glyph, axis, float(at))
            k = int(k) * 2 + (1 if kind == 'gap' else 0)
            return round(hit[k + 1] - hit[k])
    raise ValueError('cannot take ' + what)


def table(path=os.path.join(HERE, 'glyph.tsv')):
    """The measures, the faces, and each glyph's rows in order."""
    known, face, glyph, fonts = {}, [], {}, {}
    for line in open(path, encoding='utf-8'):
        cell = line.rstrip('\n').split('\t')
        if line.startswith('#') or len(cell) < 3:
            continue
        name, part, what = cell
        match part:
            case 'take': known[name] = take(what, fonts)
            case 'let': known[name] = reckon(what, known)
            case 'face': face.append((name, *what.split()))
            case _: glyph.setdefault(name, []).append((part, what))
    return known, face, glyph, fonts


def corners(what, known):
    return [tuple(reckon(n, known) for n in group.split()) for group in what.split(',')]


def outlines(name, glyph, known):
    """A glyph's outlines as (corners, is_ink), its width and character."""
    wide, char, ring = known['w'], None, []
    for part, what in glyph[name]:
        match part:
            case 'char': char = int(what, 16)
            case 'wide': wide = reckon(what, known)
            case 'fill' | 'cut': ring.append((corners(what, known), part == 'fill'))
            case 'turn' | 'mirror':
                # Turned about its own top: a figure's cap, a letter's x-height.
                other, w, _ = outlines(what, glyph, known)
                h = max(y for points, _ in other for _, y, _ in points)
                for points, ink in other:
                    match part:
                        case 'turn': ring.append(([(w - x, h - y, r) for x, y, r in points], ink))
                        case 'mirror': ring.append(([(w - x, y, r) for x, y, r in reversed(points)], ink))
                wide = w
    return ring, wide, char


def circles(points, name):
    """Each corner as the circle touching both its sides (y down, as
    chain.py takes it): inside the ink where the outline turns with the
    clock, outside it, radius below nothing, where it turns against."""
    pt = [(x, -y, r) for x, y, r in points]
    n, out = len(pt), []
    for k in range(n):
        (ax, ay, _), (px, py, r), (bx, by, _) = pt[k - 1], pt[k], pt[(k + 1) % n]
        one, two = math.hypot(ax - px, ay - py), math.hypot(bx - px, by - py)
        ux, uy, vx, vy = (ax - px) / one, (ay - py) / one, (bx - px) / two, (by - py) / two
        turn = math.acos(max(-1.0, min(1.0, ux * vx + uy * vy)))
        bx_, by_ = ux + vx, uy + vy
        size = math.hypot(bx_, by_) or 1.0
        reach = r / math.sin(turn / 2) if r else 0.0
        centre = (px + bx_ / size * reach, py + by_ / size * reach)
        sign = 1 if (px - ax) * (by - py) - (py - ay) * (bx - px) > 0 else -1
        if r / math.tan(turn / 2) > min(one, two) + 0.5:
            raise ValueError('%s: corner %d is rounder than its sides allow' % (name, k))
        out.append((centre, sign * r))
    kept = []
    for c in out:
        if not kept or not same(kept[-1], c):
            kept.append(c)
    while len(kept) > 1 and same(kept[0], kept[-1]):
        kept.pop()
    return kept


def same(a, b):
    return math.hypot(a[0][0] - b[0][0], a[0][1] - b[0][1]) < 0.01 and abs(a[1] - b[1]) < 0.01


def arcs(ring, name):
    """The chain as arcs (centre, radius, from, span), y down."""
    if len(ring) == 1:
        return [(ring[0][0], ring[0][1], 0.0, 2 * math.pi)]
    n = len(ring)
    for k in range(n):
        (a, ra), (b, rb) = ring[k], ring[(k + 1) % n]
        if math.hypot(b[0] - a[0], b[1] - a[1]) <= abs(ra - rb) + 1e-6:
            raise ValueError('%s: circles %d and %d have no line touching both' % (name, k, (k + 1) % n))
    normal = [touch(ring[k], ring[(k + 1) % n]) for k in range(n)]
    out = []
    for k, (centre, r) in enumerate(ring):
        span = (normal[k] - normal[k - 1]) % (2 * math.pi)
        out.append((centre, r, normal[k - 1], span if r > 0 else span - 2 * math.pi if r < 0 else 0.0))
    return out


def grow(ring, by, name):
    """The chain moved out by `by` (in where negative): each radius by it."""
    moved = [(c, r + by) for c, r in ring]
    for (c, r), (_, was) in zip(moved, ring):
        if was == 0 or (r > 0) != (was > 0):
            raise ValueError('%s: a corner too sharp for the line (radius %g)' % (name, was))
    return moved


def draw(pen, arc, backwards=False):
    """One closed outline into a pen, in font units (y up)."""
    piece = []
    for (cx, cy), r, start, span in arc:
        steps = max(1, math.ceil(abs(span) / (math.pi / 2) - 1e-9)) if span else 0
        at = lambda a: (cx + r * math.cos(a), cy + r * math.sin(a))
        lean = lambda a: (-r * math.sin(a), r * math.cos(a))
        piece.append(('L', at(start)))
        for k in range(steps):
            a0, a1 = start + span * k / steps, start + span * (k + 1) / steps
            f = 4 / 3 * math.tan((a1 - a0) / 4)
            p0, p3, t0, t1 = at(a0), at(a1), lean(a0), lean(a1)
            piece.append(('C', (p0[0] + f * t0[0], p0[1] + f * t0[1]), (p3[0] - f * t1[0], p3[1] - f * t1[1]), p3))
    flip = lambda p: (round(p[0], 2), round(-p[1], 2))
    if backwards:
        from fontTools.pens.reverseContourPen import ReverseContourPen
        pen = ReverseContourPen(pen)
    pen.moveTo(flip(piece[0][1]))
    for one in piece[1:]:
        match one[0]:
            case 'L': pen.lineTo(flip(one[1]))
            case 'C': pen.curveTo(*map(flip, one[1:]))
    pen.closePath()


def shape(name, glyph, known, mode, pen):
    """Draw a glyph as the face draws it, shifted by the side bearing."""
    from fontTools.pens.transformPen import TransformPen
    ring, wide, char = outlines(name, glyph, known)
    pen = TransformPen(pen, (1, 0, 0, 1, known['side'], 0))
    for points, ink in ring:
        chain = circles(points, name)
        match mode:
            case 'solid':
                draw(pen, arcs(chain, name), backwards=ink)
            case 'line':
                half = known['line'] / 2
                draw(pen, arcs(grow(chain, half, name), name), backwards=True)
                draw(pen, arcs(grow(chain, -half, name), name))
    return wide + 2 * known['side'], char


if __name__ == '__main__':
    from build import build, mock, sheet
    known, face, glyph, _ = table()
    args = sys.argv[1:]
    match args[:1]:
        case ['--mock']: mock(known, face, glyph, args[1])
        case ['--sheet']: sheet(face, args[1])
        case _: build(known, face, glyph)
