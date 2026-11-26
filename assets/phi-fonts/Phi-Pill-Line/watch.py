"""Draw the watches of `watch.tsv`, set in the built Phi-Pill Line and Link.

    python watch.py OUT_FOLDER     one picture a watch, and sheet.png of all

Every part is a pill or a rounded box of the table's radius; the figures
are the fonts themselves, turned along the dial as `dial.tsv` says."""
import math
import os
import sys

from PIL import Image, ImageDraw, ImageFont

from build import PAPER, dial, file_name

HERE = os.path.dirname(os.path.abspath(__file__))
FINE = 2


def table():
    measure, watch = {}, []
    for line in open(os.path.join(HERE, 'watch.tsv'), encoding='utf-8'):
        cell = line.rstrip('\n').split('\t')
        if line.startswith('#') or len(cell) < 3:
            continue
        match cell[1]:
            case 'measure': measure[cell[0]] = [float(n) for n in cell[2].split()]
            case 'watch': watch.append((cell[0], cell[2].split()))
    return measure, watch


def face(style, size):
    family = {'line': ('Phi-Pill Line', 'Light'), 'link': ('Phi-Pill Link', 'Regular')}[style]
    return ImageFont.truetype(os.path.join(HERE, file_name(*family)), int(size * FINE))


def shade(colour, by):
    """A colour toward white (by > 0) or black (by < 0)."""
    rgb = [int(colour[k:k + 2], 16) for k in (1, 3, 5)]
    end = 255 if by > 0 else 0
    return '#%02x%02x%02x' % tuple(round(c + (end - c) * abs(by)) for c in rgb)


def box(pen, x0, y0, x1, y1, radius, **paint):
    pen.rounded_rectangle([v * FINE for v in (x0, y0, x1, y1)], radius=radius * FINE, **paint)


def pill(pen, x0, y0, x1, y1, **paint):
    box(pen, x0, y0, x1, y1, min(x1 - x0, y1 - y0) / 2, **paint)


def hand(pen, at, angle, tail, length, wide, fill, edge=None):
    """A pill from the centre along a clock angle, its ends fully round."""
    a = math.radians(angle)
    ux, uy, nx, ny = math.sin(a), -math.cos(a), math.cos(a), math.sin(a)
    point = []
    # Each end a half circle: the tip's from one side over the end to the
    # other, the tail's back again.
    for reach, turn in ((length - wide / 2, 0), (-tail + wide / 2, math.pi)):
        cx, cy = at[0] + ux * reach, at[1] + uy * reach
        for k in range(17):
            t = turn + math.pi * k / 16
            dx, dy = math.cos(t), math.sin(t)
            point.append(((cx + (nx * dx + ux * dy) * wide / 2) * FINE, (cy + (ny * dx + uy * dy) * wide / 2) * FINE))
    pen.polygon(point, fill=fill, outline=edge, width=int(2.5 * FINE) if edge else 1)


def text(page, words, font, colour, at, turn=0.0):
    pen = ImageDraw.Draw(page)
    left, top, right, bottom = pen.textbbox((0, 0), words, font=font)
    tile = Image.new('L', (right - left + 8, bottom - top + 8), 0)
    ImageDraw.Draw(tile).text((4 - left, 4 - top), words, font=font, fill=255)
    tile = tile.rotate(-turn, expand=True, resample=Image.BICUBIC)
    spot = (int(at[0] * FINE - tile.size[0] / 2), int(at[1] * FINE - tile.size[1] / 2))
    page.paste(Image.new('RGB', tile.size, colour), spot, tile)


def band(pen, make, colour, other, x0, x1, y0, y1, low):
    box(pen, x0, y0, x1, y1, 30, fill=colour)
    match make:
        case 'sport':
            if low:
                for k in range(5):
                    y = y0 + 90 + k * 44
                    pill(pen, (x0 + x1) / 2 - 15, y, (x0 + x1) / 2 + 15, y + 16, fill=shade(other, -0.35))
        case 'knit':
            stripe = (x1 - x0) / 5
            for k in (1, 3):
                pen.rectangle([(x0 + k * stripe) * FINE, y0 * FINE, (x0 + (k + 1) * stripe) * FINE, y1 * FINE], fill=other)
            for y in range(int(y0), int(y1), 7):
                pen.line([x0 * FINE, y * FINE, x1 * FINE, y * FINE], fill=shade(colour, -0.25), width=FINE)
        case 'link':
            for y in range(int(y0) + 8, int(y1) - 20, 34):
                pill(pen, x0 + 8, y + 3, x1 - 8, y + 27, fill=shade(colour, -0.2))
                pill(pen, x0 + 8, y, x1 - 8, y + 24, fill=shade(colour, 0.1))
        case 'plain':
            for x in (x0 + 10, x1 - 10):
                for y in range(int(y0), int(y1), 14):
                    pen.line([x * FINE, y * FINE, x * FINE, (y + 7) * FINE], fill=shade(colour, 0.35), width=FINE)


def watch(measure, row):
    case, rim, band_c, band2, make, ground, ink, accent, figures, hands, date = row
    pw, ph = measure['page']
    page = Image.new('RGB', (int(pw * FINE), int(ph * FINE)), PAPER)
    pen = ImageDraw.Draw(page)
    (cw, ch, cr), (ri,), (si, sr), (bw,), (fs,), (hh, mm, ss) = (measure[k] for k in ('case', 'rim', 'screen', 'band', 'figure', 'time'))
    cx, cy = pw / 2, ph / 2
    x0, y0, x1, y1 = cx - cw / 2, cy - ch / 2, cx + cw / 2, cy + ch / 2
    band(pen, make, band_c, band2, cx - bw / 2, cx + bw / 2, -40, y0 + 30, False)
    band(pen, make, band_c, band2, cx - bw / 2, cx + bw / 2, y1 - 30, 940, True)
    pill(pen, x1 - 6, cy - 110, x1 + 16, cy - 40, fill=shade(case, -0.15))
    for y in range(int(cy - 104), int(cy - 44), 8):
        pen.line([(x1 + 4) * FINE, y * FINE, (x1 + 14) * FINE, y * FINE], fill=shade(case, -0.4), width=FINE)
    pill(pen, x1 - 6, cy + 10, x1 + 8, cy + 80, fill=shade(case, -0.1))
    box(pen, x0, y0, x1, y1, cr, fill=case)
    box(pen, x0 + ri, y0 + ri, x1 - ri, y1 - ri, cr - ri, fill=rim)
    sx0, sy0, sx1, sy1 = x0 + si, y0 + si, x1 - si, y1 - si
    box(pen, sx0, sy0, sx1, sy1, sr, fill=ground)
    mx, my = (sx0 + sx1) / 2, (sy0 + sy1) / 2
    a, b = (sx1 - sx0) / 2 - fs * 0.85, (sy1 - sy0) / 2 - fs * 0.85
    match figures:
        case 'digital':
            text(page, '%d:%02d' % (hh, mm), face('link', fs * 2.1), ink, (mx, my - 18))
            text(page, 'fri 27' if date == 'day' else '27 nov', face('line', fs * 0.95), accent, (mx, my + 62))
            return page
        case _:
            for hour, turn in dial():
                n = int(hour)
                cardinal = n % 3 == 0
                if (figures == 'four' and not cardinal) or (date == 'box' and n == 3):
                    continue
                style = {'line': 'line', 'link': 'link', 'mixed': 'link' if cardinal else 'line', 'four': 'link'}[figures]
                t = math.radians(n * 30)
                text(page, hour, face(style, fs * (1.25 if figures == 'four' else 1)), ink, (mx + math.sin(t) * a, my - math.cos(t) * b), turn)
    match date:
        case 'box':
            box(pen, mx + a - 30, my - 20, mx + a + 18, my + 20, 10, outline=accent, width=int(2 * FINE))
            text(page, '27', face('line', fs * 0.7), ink, (mx + a - 6, my))
        case 'day':
            text(page, 'fri', face('line', fs * 0.7), accent, (mx, my + b * 0.45))
        case 'none':
            pass
    edge = ink if hands == 'line' else None
    fill = ground if hands == 'line' else ink
    hand(pen, (mx, my), (hh + mm / 60) * 30, 0, b * 0.5, 14, fill, edge)
    hand(pen, (mx, my), mm * 6, 0, b * 0.82, 11, fill, edge)
    hand(pen, (mx, my), ss * 6, 22, b * 0.9, 3.5, accent)
    pill(pen, mx - 6, my - 6, mx + 6, my + 6, fill=accent)
    pill(pen, mx - 2.5, my - 2.5, mx + 2.5, my + 2.5, fill=ground)
    return page


if __name__ == '__main__':
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    measure, watches = table()
    pw, ph = (int(n) for n in measure['page'])
    sheet = Image.new('RGB', (4 * pw // 2, 4 * ph // 2), PAPER)
    for k, (name, row) in enumerate(watches):
        page = watch(measure, row).resize((pw, ph), Image.LANCZOS)
        page.save(os.path.join(out, name.replace(' ', '-') + '.png'))
        sheet.paste(page.resize((pw // 2, ph // 2), Image.LANCZOS), ((k % 4) * pw // 2, (k // 4) * ph // 2))
    sheet.save(os.path.join(out, 'sheet.png'))
    print('drew', len(watches), 'watches into', out)
