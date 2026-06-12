# Backlog

History: the June 2026 architecture/perf review produced lots (a) quick wins,
(b) UI/healthbar perf, (c) Bevy `States` migration + per-module plugins, and
(d) file/module split — all done. (Longer-term idea parked from lot d:
per-kind AI systems sharing a once-per-frame combatant snapshot.)

**Direction (2026-06-12): this is no longer a POC — the target is a
distributable Steam game.** The bar is "fun with friends, 1v1 or 2v2".
Design anchor: **~25-minute matches, chill pace** — a player should be able
to put the pad down for a moment. Playtest feedback so far: gameplay too
limited; balance unproven (not enough testing yet). V1 scope: at least 3
more nations, SFX, gold-sending between teammates. Perf target: potentially
hundreds of simultaneous units. Platforms: Linux/Windows/macOS (CI/CD
already compiles all three).

Backlog is organized by priority level: **P0** = hurts every session right
now, **P1** = V1 / Steam foundations, **P2** = small or cosmetic.

## P0 — fun now (every-session pain)

- [ ] **HUD rework** — mockup validated 2026-06-12, spec in
  `docs/hud-redesign.md`: horizontal action bar per player (cells with cost,
  gamepad glyphs, single-line stats), fixed sizes everywhere so layout never
  shifts (miner cap → grey `MAX` cell, not `Display::None`), same bar ×4 in
  2v2. Phase 1 (layout + frosted-glass look) is implemented — pending
  in-game validation; phase 2 = real backdrop blur via a custom `UiMaterial`
  (user picked real blur over the fake), and an icon font for proper button
  glyphs (Bevy's default font is ASCII-only, hints use "(A)"/"(X)"). (A
  production queue is a separate gameplay item in P1, not a HUD concern for
  now.)
- [x] **Action timing tied to animation keyframes** — done 2026-06-12:
  per-kind `impact_fraction` in `UnitStats` (soldier blade contact 0.35,
  miner pick bite 0.9, priest cast 0.6; archer keeps its release fraction);
  `combat_tick` queues a `PendingImpact`, `animate_unit_model` fires it when
  the clip crosses the point, `apply_action_impacts` applies it. Units with
  no bound animation fall back to the timer cadence (headless-harness
  groundwork; the archer still needs an animation to shoot). Fractions are
  first-guess values — tune in-game.
- [ ] **Pause: full freeze + blur** — entering Paused zeroes velocities and
  pauses `Time<Physics>`, but the `AnimationPlayer`s keep running so
  character animations play to completion. Pause/resume every unit's
  animation player on `OnExit`/`OnEnter(Playing)`. Also blur the battlefield
  behind the pause overlay. Any player's pad can pause (verify this is
  already the case in `gameplay_input_system`).
- [x] **Miner carry capacity** — done 2026-06-12: `MINER_CAPACITY` = 5 (one
  gold per swing, five swings per round-trip); the carry/phase logic already
  existed, and the keyframe-timing item makes the miner finish its last
  swing before turning back.
- [ ] **2v2 arena too cramped** — the terrain doesn't widen enough in 2v2;
  rescale the play field (lane widths, ground sand band, tower zone, base
  spacing) so four armies have room.
- [ ] **Startup asset-load hitching → masked loading** — chosen approach:
  hide loading behind the menu (menu appears instantly, models/scenes stream
  in the background; gate match launch — or the SideSelect → Playing
  transition — on `AssetServer` load state so the first spawn never hitches).
  No visible loading screen.

## P1 — V1 / Steam foundations

- [ ] **Headless auto-battle harness** — windowless/renderless sim runner
  (arena + Avian + `CombatSet` at an accelerated fixed timestep): feed it a
  scenario (army compositions, starting gold), get winner / duration /
  surviving HP; sweep matchups × seeds into a win-rate matrix. Unblocks the
  balancing pass without human testers and becomes the non-regression net
  for every nation/tech change. Prereq couplings: partly broken by the P0
  animation-timing item (soldier/miner/priest now have a timer-driven
  fallback when no animation is bound); still needed: an archer fallback
  (its shot only fires from `animate_unit_model`) and "bare" units in
  `spawn_unit` (collider + stats, no glTF). Do this *before* the balancing
  pass.
- [ ] **Balancing pass** — parked at P1: the gap so far is *lack of testing*,
  not a diagnosed imbalance. Anchor: ~25-minute matches, chill pace. Once
  the auto-battle harness and/or playtests identify what dominates, iterate
  on `config/units.rs` + `config/weapons.rs`.
- [ ] **`combat_tick` O(n²)** — real subject now (target: hundreds of
  units). Per unit per frame: multiple linear scans; `volley_aim` is
  O(enemies²) per archer with 2 Vec allocs per call. Plan: cache `volley_aim`
  per side, pre-partition combatants by side, then a spatial grid. Same
  family: `apply_player_focus_visual` counts miners by iterating all units
  every frame in-match — fold a per-slot cached count into the same pass.
- [ ] **Non-deterministic damage ordering** — real subject. `combat_tick`,
  `tower_attack_tick` and `arrow_flight_system` conflict on `Health`, so
  Bevy serializes them in an ambiguous order (frame-to-frame nondeterminism
  in a versus game). Fix the order explicitly.
- [ ] **Nations ×3 (epic)** — at least 3 more nations for V1 (only Ada'Ram
  exists). Everything is to invent (no design notes yet); assets via Meshy
  like Ada'Ram. Each nation mixes units similar to and/or different from the
  others' rosters, with distinct mechanics. The `Nation` plumbing already
  exists in SideSelect. First step: a design doc for nation #2, then break
  the epic down.
- [ ] **SFX (epic)** — V1 scope: combat hits, arrows, mining, UI navigation/
  purchase; only `battleTheme.mp3` exists today.
- [ ] **Tech / upgrades (epic)** — the answer to "what fills a chill
  25-minute match": phases driven by unlockable improvements/tech. All
  design to invent (what's upgradable — unit stats, new kinds, towers,
  economy? per-match or persistent? cost/pacing?). This is the main
  gameplay-depth lever for V1, alongside the nations.
- [ ] **Production queue (explore)** — recurring/queued unit purchases so a
  player can set up production and put the pad down (fits the chill-pace
  anchor). Design first: queue vs repeat-order, gold reservation, HUD
  representation.
- [ ] **Gold sending between teammates** — 2v2 mechanic: keep per-slot gold
  but let a player send gold to their ally.
- [ ] **Steam packaging** — targets Linux/Windows/macOS; CI/CD already
  compiles all three, so what remains is distribution polish (asset
  bundling, per-platform smoke tests — note `raytracing`/Solari likely has
  no macOS/Metal path, verify the fallback) and, much later, Steamworks
  integration + store page (explicitly distant).
- [ ] **`debug_camera_control` ships in release** — gate behind
  `cfg(debug_assertions)` so a player bumping the keyboard can't derail the
  camera. Mandatory before any distributed build.

## P2 — small / cosmetic

- [ ] **Settings key table** — `load_settings`/`save_settings` list every key
  twice; a shared `(key, accessor)` table prevents drift when adding params.
- [ ] **Menu slot counts** — `pause_input_system` hardcodes `SLOTS = 3`,
  `menu_input_system` hardcodes 4/1; must stay in sync with the buttons
  spawned in the overlay builders. Give each screen a single source of truth
  like the settings tab's `tab_slots()`.
- [ ] **Dying tower** — keeps its collider during the 0.45 s collapse and can
  fire one last arrow on the frame HP hits 0 (`TowerDying` inserted a frame
  later). Cosmetic.
- [ ] **`bind_unit_weapon_hand`** — does the hierarchy walk before checking
  the bone name; pre-filter on `name == "LeftHand" | "RightHand"` to skip the
  walk for every scenery prop `Name`.
- [ ] **Clippy const asserts** — the 3 `assertions_on_constants` warnings in
  tests: move those invariants to `const { assert!(…) }` outside tests so a
  violation breaks compilation, not just `cargo test`.

## Deferred decisions (answers pending)

- Counter-pick / unit-variety depth: "not yet" — revisit with the nations.
- Combat visual feedback (impact FX, damage numbers): not for now.
