# state

State — a flat `HashMap<String, String>` keyed by encoded `BPath`.

Every key is `ctx.<seg>.<seg>...`. Host state naturally nests under the
host's slug: `ctx.<host>.<tail>`. Read via `state.host(host).get(tail)`.
Three-letter accessors: `get`, `set`, `has`.

## Files

| File | What it is |
|---|---|
| `mod.rs` | `BState` — the flat store; `get / set / has / remove` take `impl BKey`. |
| `path.rs` | `BPath` — hierarchical state key. |
| `key/` | One file per domain, each an enum that `impl BKey` — the only place a path is spelt (`tests/key_budget.rs`). `key/mod.rs` is the `BKey` trait and the domain list, nothing else. |
| `key/<host>.rs` | Host state nests under the host's slug, so a host's keys are its own enum spelling its own slug: `ButsKey` writes `"buts"`, `PrefsKey` writes `"prefs"`. A key enum never reaches into `host::` to spell itself — that would put the path in two places again. |
| `key/ui_style.rs` | The theme tokens, the one domain spelt as consts: read in bulk by `theme::resolver`, so a variant per token would buy nothing. |
| `rule.rs` | `trait BRule` and the ordered table of follow-ups; each rule lives with its level (`tests/rule_budget.rs`). |
| `action.rs` | Dispatch of a rendered `BAction`: store, `edit::before_set`, rules, close overlays. Compares no paths. |
| `edit.rs` | The caret: which field is being typed into, and its value for Escape. |
| `menu.rs` | Menu open/close reducer and `close_overlays`. |
| `log/` | The action log — one sentence per action, for the status bar. |
| `seed.rs` | Boot defaults — the initial state a fresh `BState` starts with. |
| `val.rs` | Static value constants — flags, modal slugs, Blender's factory defaults. |

Moved out to their levels: area sizes → `area/size.rs`, zone sizes →
`zone/size.rs`, switching space → `space/switch.rs`, file-browser history →
`part/custom/file/op.rs`, the other follow-ups → beside their owners
(see `rule.rs`). `tests/rule_budget.rs` keeps this list from growing.

## Two rules, and where they are still broken

1. **A path is spelt once, in its domain's `key/` file, and is *named* — not
   aliased.** Two rulers:
   - `tests/key_budget.rs` — a moved domain's path is never hand-spelt
     outside its `key/` file. Covers the domains in its `MOVED` table.
   - `tests/key_alias_budget.rs` — no *new* zero-argument `fn () -> BPath`
     outside `key/`. This closes the hole the first one had: it greps for a
     spelt path, so a path wrapped in a function was invisible to it, and
     `scene/key.rs` grew fifteen such wrappers. The 31 that remain are
     listed there and categorised; the list may only shrink.

   Every domain now has an enum and is covered by `MOVED`. What is left in
   the alias budget is 7 menu `fn` pointers (structural — `BMenuKind::Toggle`
   holds a `fn() -> BPath`, so an entry flipping a fixed key needs a named
   function to point at) and 10 single-variant aliases. See `jobs.md`.
2. **A follow-up is a `BRule`, and it claims its work through `wants`.** Not
   by reading a value to decide whether the value was meant for it, and not
   as a free function another rule calls from inside its `after` — that is a
   hand-written `wants` with no table entry. One op key per rule.

Status and next steps: [`jobs.md`](jobs.md).
