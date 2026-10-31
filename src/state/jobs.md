# Jobs — `phiber/state`

| State | Open | Done |
|---|---|---|
| 🟢 clean | 3 | 8 |

## Todo

- [ ] A handful of verbs: `get`, `has`, `set`, `remove` — fold `set_encoded`, `get_encoded`, `iter_keys` into them
- [ ] Persistence (see `app` jobs): transient-key list lives here
- [ ] `ButsKey::NavbarExpanded` is `ctx.buts.navbar.expanded`, which `part::strip::expanded_key(Buts, "navbar")` also builds — one path, two spellings. Found while adding `PrefsKey` (which deliberately has **no** `NavbarExpanded` for this reason). Either drop the variant and use the builder, or make the builder's Buts case go through the variant

## Done

- [x] Every *fixed* key is a `BKey` enum in `state/key/` — `tests/key_budget.rs` (sync111–116)
- [x] Follow-ups are `BRule`s beside their levels, `state/rule.rs` is the table — `tests/rule_budget.rs` (sync117)
- [x] Area/zone size reducers and space switching moved to `area/size.rs`, `zone/size.rs`, `space/switch.rs` (sync118)
- [x] Keys for view pivot / lens, frame range / fps, links, command query, collaboration
- [x] The `ui_style` token catalogue out of `key/mod.rs` into `key/ui_style.rs` — it was two thirds of the file that holds the `BKey` trait, and serves one consumer (`theme::resolver`). `key/mod.rs` 194 → 108 lines
- [x] The 7 `scene/key.rs` load / error / selected aliases folded into `SceneKey` and `FileKey` — ~300 call sites across `src` and `tests`. They returned exactly what the enum returned, so each key had two spellings; `local_keys.rs` used both, one line apart. Most call sites got **shorter**: `get/set/has/remove` take `impl BKey`, so `s.get(&loading_key())` → `s.get(SceneKey::Loading)`
- [x] `tests/key_alias_budget.rs` — the hole that let the aliases grow. `key_budget.rs` greps for a spelt path, so a path wrapped in a function was invisible to it. The new ruler finds every **zero-argument** `fn () -> BPath` outside `state/key/` and fails on a new one; the list may only shrink
- [x] The 8 view-slot aliases (`zoom_key`, `yaw_key`, `pitch_key`, `pivot_key`, `pan_x_key`, `pan_y_key`, `perspective_key`, `lens_key`) gone. Call sites say `BViewSlot::key(View3dKey::Zoom)`, which names the per-viewport slot resolution the alias hid. `scene/key.rs` now holds no keys at all — only the `view_*` readers that apply a factory default or a clamp
- [x] The four domains with **no key enum** now have one: `key/sidebar.rs`, `key/asset.rs`, `key/spreadsheet.rs`, `key/prefs.rs`, all four added to `key_budget.rs`'s `MOVED`. Two were genuinely duplicated behind their alias functions — `prefs.navbar.tab` in two files, `spreadsheet.domain` in three (`dataset.rs`, `set.rs` via the host accessor, and a test). `key_alias_budget.rs`'s `ALLOWED` is down from 31 to 17, and every remaining entry is either a menu `fn` pointer (structural) or a single-variant alias

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
