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
| Phiber | Phiber (the interface default) | Light, Regular, SemiBold, Bold |
| Phi-Soft | Phi-Soft, Phi-Soft Condensed | Light, Regular, SemiBold, Bold |
| Phi-Sans | Phi-Sans, Phi-Sans Condensed | same |
| Phi-Serif | Phi-Serif, Phi-Serif Condensed | same |
| Phi-Hard | Phi-Hard, Phi-Hard Condensed | same |
| Phi-Pill | Phi-Pill, Phi-Pill Condensed | same |
| Phi-Walt | Phi-Walt, Phi-Walt Condensed | same |
| Loras-Spark | Lora's Spark, Lora's Spark Condensed (slug `lora_s_spark`) | same |
| Chatty-Whims | Chatty Whims, Chatty Whims Condensed | same |

The default is **Phiber Light**: Phi-Soft Light opened up for the interface. Phi-Soft draws its vertical stems twice as thick as its horizontal strokes, which crowds the inside of each letter; Phiber thins the verticals to 1.2x the horizontals (the horizontals keep their thickness exactly), then makes every letter 20% wider and spaces them 64 units (of 1000) further apart, so text reads at 12-14 px without feeling cramped. Phi-Soft's Regular read heavy, its Condensed width cramped, and even its full width tight.

`Phiber/phiberize.py` makes the family from Phi-Soft's four weights (it needs `fontTools`):

```
python phiberize.py ../Phi-Soft/PhiSoft-Light.otf Phiber-Light.otf 1.20 64 Light Phiber 1.2
```

Rerun it with other numbers to tune the openness: the width scale, the extra spacing in font units, and the stem ratio (vertical stem thickness as a multiple of the horizontal stroke, measured on H and E). No stroke is thinned below a horizontal's thickness. Phi-Soft's SemiBold and Bold draw M and W as solid stencil shapes by design, and Phiber keeps them. It redraws the outlines, moves the side bearings, scales any kerning, and renames the family so it registers beside Phi-Soft.

## Notes

- **The browser build** has no disk to read this folder from. It carries one face compiled into the binary, Phiber Light (`font::UI_FACES`). A desktop build whose folder is missing falls back to the same face.
- **Windows:** our vendored DirectWrite backend (`phigpu/gpui_windows`, `typographic_collection`) groups added fonts by typographic family. Without that, the two widths of a folder would merge into one family and the Condensed faces could never be reached.
