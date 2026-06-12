# HUD redesign — validated mockup (2026-06-12)

Decisions (user-validated):
- **Layout: horizontal action bar** per player along the screen edge
  (variant B), replacing the corner column panels.
- **Stats: single reserved line** under the bar (no card).
- **Blur: real backdrop blur** via a custom `UiMaterial` sampling the scene
  behind the node. Implemented as **phase 2**; phase 1 ships with the
  frosted-glass fake (translucent dark + rounded corners + thin light
  border) so the layout rework isn't blocked by the shader.
- 2v2 reuses the same bar, declined ×4 (top players along the top edge).
- Gamepad glyphs on the hint line; the existing ✛ left/right bindings
  already navigate the bar.

**Hard rule: nothing ever moves.** Every container has a fixed size; state
changes recolor or grey out, they never add/remove/resize nodes. (The two
offenders today: the stats card resizing with its text, and the Miner button
`Display::None` at cap.)

## 1v1 — full screen

```
┌──────────────────────────────────────────────────────────────────────┐
│                              ╭────────╮                              │
│                              │ 06:24  │  ← clock, centered, frosted  │
│                              ╰────────╯                              │
│                                                                      │
│                                                                      │
│                         (battlefield, unobstructed)                  │
│                                                                      │
│                                                                      │
│ ♥ ████████░░░ 32/40    ◆ 12 g        ◆ 8 g    ♥ █████░░░░░ 21/40     │
│ ╭────╮╔════╗╭────╮╭────╮╭────╮        ╭────╮╭────╮╭────╮╔════╗╭────╮ │
│ │Twr │║Sol ║│Arc ││Pri ││Min │        │Min ││Pri ││Arc │║Sol ║│Twr │ │
│ │ 6g │║ 1g ║│ 3g ││ 5g ││ 4g │        │ 4g ││ 5g ││ 3g │║ 1g ║│ 6g │ │
│ ╰────╯╚════╝╰────╯╰────╯╰────╯        ╰────╯╰────╯╰────╯╚════╝╰────╯ │
│ Soldier · HP 100 · DMG 10 · SPD 2.0   Soldier · HP 100 · DMG 10 ·…   │
│ ✛ naviguer  Ⓐ acheter  Ⓧ tour        ✛ naviguer  Ⓐ acheter  Ⓧ tour │
└──────────────────────────────────────────────────────────────────────┘
```

- Left player's bar anchors bottom-left, right player's bottom-right,
  **mirrored** (Tower outermost on both sides, buttons walk inward) so each
  player reads "their" bar from the screen edge.
- Bar accent color = `side.color()` (cell borders, HP/gold icons).
- Cell = fixed square: 3-letter label (later: icon) + cost. Focused cell =
  white double border + lighter background.

## Cell states (recolor only — never resize/remove)

```
 normal      focused     can't afford   miner cap     defeated (whole bar)
╭────╮      ╔════╗      ╭────╮         ╭────╮        all cells + pills
│Arc │      ║Arc ║      │Pri │         │Min │        greyed (BTN_DISABLED
│ 3g │      ║ 3g ║      │ 5g │←red     │MAX │←grey   palette), hint line
╰────╯      ╚════╝      ╰────╯         ╰────╯        cleared
```

- Can't afford: cost text turns red (cell stays buyable-looking otherwise;
  the buy action already refuses).
- Miner at `MAX_MINERS_PER_PLAYER`: cell greys out and cost reads `MAX`;
  it stays in place and focus skips it (input already does).
- Base destroyed: entire bar greys (replaces today's corner grey-out).

## Hint line (fixed height, swaps content by mode)

```
 normal:     ✛ naviguer   Ⓐ acheter   Ⓧ tour
 placement:  ✛ déplacer   Ⓐ poser     Ⓑ annuler
 defeated:   (empty — height still reserved)
```

## Stats line (fixed height, refreshed on focus change)

```
 Tower   · HP 60  · DMG 5  · RNG 9.0 · CD 1.5 s
 Soldier · HP 100 · DMG 10 · SPD 2.0 · CD 1.0 s
 Miner   · HP 80  · CAP 5  · SPD 2.2 · CD 1.2 s
```

(values from `UnitKind::stats()` / tower consts, same source as today)

## 2v2 — full screen

```
┌──────────────────────────────────────────────────────────────────────┐
│ ♥ ███████░ 28/40  ◆ 5 g      ╭────────╮     ◆ 9 g  ♥ ████░░ 17/40   │
│ ╭────╮╭────╮╭────╮╭────╮╭───╮│ 14:51  │╭───╮╭────╮╭────╮╭────╮╭────╮│
│ │Twr ││Sol ││Arc ││Pri ││Min│╰────────╯│Min││Pri ││Arc ││Sol ││Twr ││
│ ╰────╯╰────╯╰────╯╰────╯╰───╯          ╰───╯╰────╯╰────╯╰────╯╰────╯│
│ stats line · hints              ↑LT/RT  stats line · hints           │
│                                                                      │
│                         (battlefield)                                │
│                                                                      │
│ ♥ ████████░ 32/40  ◆ 12 g                ◆ 8 g  ♥ █████░ 21/40      │
│ ╭────╮╔════╗╭────╮╭────╮╭────╮          ╭────╮╭────╮╭────╮╔════╗╭──╮│
│ │ …same bar as 1v1, bottom corners…     │ …mirrored…               ││
│ ╰────╯╚════╝╰────╯╰────╯╰────╯          ╰────╯╰────╯╰────╯╚════╝╰──╯│
│ stats line · hints                      stats line · hints           │
└──────────────────────────────────────────────────────────────────────┘
```

- Top bars are the same component flipped vertically (hint/stats lines
  below the cells, i.e. toward screen center, so cells hug the edge).
- Clock stays centered top between the two top bars (it fits: bars are
  half-width minus clock clearance).

## Visual language

- Frosted glass chips: `srgba(0.04, 0.05, 0.08, 0.72)` bg, 1 px border
  `srgba(0.9, 0.9, 0.95, 0.35)`, `BorderRadius` ~8 px. Phase 2 swaps the
  flat alpha for the blur `UiMaterial` (same geometry).
- Typography: keep current font; sizes — cell label 14, cost 13, stats/hints
  12, HP/gold pills 16, clock 24. Cost in gold color `srgb(0.95, 0.8, 0.35)`
  (red when unaffordable).
- Glyphs Ⓐ Ⓑ Ⓧ ✛: text-rendered (circled unicode) in phase 1; proper
  button icons can come with the SFX/juice pass.

## Implementation notes

- Rewrite `spawn_player_corner`/`spawn_player_panel` in `ui/hud.rs` into a
  `spawn_player_bar(slot)` builder; keep the marker components
  (`PanelSlot`, `GoldText`, `BaseHpText`, `FocusStatsText`, `PlayerCorner`,
  `TopPlayerHud`, `GameHud`) and refresh systems — only layout/state
  visuals change. `apply_player_focus_visual` loses the `Display::None`
  miner branch (becomes a grey+`MAX` recolor).
- Mouse hover/click panel mapping (`read_mouse_ui`) keeps working since
  `PanelSlot{slot,index}` stays on the cells.
- HP pill gains a real gauge: a fixed-width bar node with an inner fill
  whose width is `percent(hp_fraction)` — width changes are fine *inside*
  a fixed-size pill.
- Phase 2 (blur): custom `UiMaterial` — needs the backdrop sample; check
  `bevy_ui` material API in 0.18 for screen-texture access; budget it as
  its own task, risk: may need a small render-graph addition.
