# phi-fonts — the typefaces Phiber ships

Every font in this folder is offered in **Preferences ▸ Typography ▸ Family**. Phiber finds them at start-up, so nothing in the code lists them; see `src/theme/style/bundle.rs`.

## Add a typeface

1. Make a folder here, one per typeface: `assets/phi-fonts/<Name>/`.
2. Put its `.otf` or `.ttf` files in it, every weight and width you have.
3. Optionally, add a `font.toml` (below).
4. Start Phiber. The new families appear in the Family list.

Families are named by what's **inside** the files (the typographic family, name ID 16, else name ID 1), not by the folder or file names. So a folder can hold more than one family. `Phi-Soft/` holds both **Phi-Soft** and **Phi-Soft Condensed**, and each is its own choice. The slug stored in Preferences is that name in lower case, with words joined by `_`: `Phi-Soft Condensed` is `phi_soft_condensed`.

**Typography ▸ Weight** offers Light (300), Regular (400), SemiBold (600) and Bold (700). The text system uses the nearest weight a family actually has.

## `font.toml` — a folder's defaults

These are optional and apply to every family in the folder. They're what Preferences starts on, and resets to, while one of those families is chosen. Anything you've set yourself in Preferences still wins.

```toml
weight = 300   # Light; 400 Regular, 600 SemiBold, 700 Bold
body = 13      # px
header = 14
caption = 12
```

Any key left out keeps the theme's own value. A line is `key = number`, and `#` starts a comment.

## What's here

| Folder | Families | Weights |
|---|---|---|
| Phi-Pill-Open | Phi-Pill Open (the interface default) | Light |
| Phiber | Phiber | Light, Regular, SemiBold, Bold |
| Phi-Soft | Phi-Soft, Phi-Soft Condensed | Light, Regular, SemiBold, Bold |
| Phi-Sans | Phi-Sans, Phi-Sans Condensed | same |
| Phi-Serif | Phi-Serif, Phi-Serif Condensed | same |
| Phi-Hard | Phi-Hard, Phi-Hard Condensed | same |
| Phi-Pill | Phi-Pill, Phi-Pill Condensed | same |
| Phi-Walt | Phi-Walt, Phi-Walt Condensed | same |
| Loras-Spark | Lora's Spark, Lora's Spark Condensed (slug `lora_s_spark`) | same |
| Chatty-Whims | Chatty Whims, Chatty Whims Condensed | same |

The default is **Phi-Pill Open Light** (below). Before it, the default was **Phiber Light**: Phi-Soft Light opened up for the interface. Phi-Soft draws its vertical stems twice as thick as its horizontal strokes, which crowds the inside of each letter; Phiber thins the verticals to 1.2x the horizontals (the horizontals keep their thickness exactly), then makes every letter 20% wider, so text reads at 12-14 px without feeling cramped. The space between letters stays Phi-Soft's own: each letter is set back between Phi-Soft's side bearings after it is thinned and widened, so the gaps neither shrink with the thinning nor grow with the widening, and no extra tracking is added. (An earlier build added 64 units of tracking and let the widening scale the side bearings too; the letters read spaced apart.) Phi-Soft's Regular read heavy, its Condensed width cramped, and even its full width tight.

`Phiber/phiberize.py` makes the family from Phi-Soft's four weights (it needs `fontTools`):

```
python phiberize.py ../Phi-Soft/PhiSoft-Light.otf Phiber-Light.otf 1.20 0 Light Phiber 1.2
```

Rerun it with other numbers to tune the openness: the width scale, the extra spacing in font units, and the stem ratio (vertical stem thickness as a multiple of the horizontal stroke, measured on H and E). No stroke is thinned below a horizontal's thickness. Phi-Soft's SemiBold and Bold draw M and W as solid stencil shapes by design, and Phiber keeps them. It redraws the outlines, keeps the side bearings, scales any kerning, and renames the family so it registers beside Phi-Soft.

## Notes

- **The browser build** has no disk to read this folder from. It carries one face compiled into the binary, Phi-Pill Open Light (`font::UI_FACES`). A desktop build whose folder is missing falls back to the same face.
- **Windows:** our vendored DirectWrite backend (`appkit/gpu/gpui_windows`, `typographic_collection`) groups added fonts by typographic family. Without that, the two widths of a folder would merge into one family and the Condensed faces could never be reached.

**Phi-Pill Open** is Phi-Pill Light with the space inside each letter 64 units wider, for reading; it is the interface default. Phi-Pill keeps the space between two stems at 129 units in every weight, so even its Light is cramped inside. Only those spaces change, and only across: every point right of a closed space inside a letter moves right by 64 (a point inside the space by its share), so each stroke, round, fillet and height is exactly Phi-Pill Light's, and a letter is 64 units wider for each space it has (an n by 64, an m by 128). A space counts when it runs at least 60 units up or down, so two round ends that only pass each other grow nothing, while the space between a v's or k's arms does. Letters with no space between strokes (c, i, l, r, t) are Phi-Pill Light's unchanged. It is one colour: Phiber's text system bakes a colour font's palette into every glyph, so Phi-Pill's 3D side and light pills are left out, and text takes the theme's ink. Its x-height and line metrics match Phiber's, so `font.toml` keeps Phiber's sizes. `Phi-Pill-Open/pillopen.py` makes it (it needs `fontTools` and `pyclipper`):

```
python pillopen.py ../Phi-Pill/PhiPill-Light.otf PhiPillOpen-Light.otf 64
```

The number is how many units each space inside grows by. Add `colour` after it to keep Phi-Pill's 3D side as a colour font, for headings drawn where the palette is wanted. (The first Phi-Pill Open thinned Phi-Pill Regular's stems instead, which changed the letters' strokes and rounds; it is in `_temp/phi-pill-open-thinning`.)
