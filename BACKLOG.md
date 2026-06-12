# Backlog

Findings from the June 2026 architecture/perf review. Lot (a) — quick wins
(Priest HUD stats, miner death doc, `FACE_TURN_*`/`ATTACK_HOLD` renames,
`shortest_yaw_diff` reuse, `load_settings` clamp dedup, single shared `Rng`,
`expect` → `let-else` in `combat_tick`) — and lot (b) — UI/healthbar perf
(shared `HealthBarAssets` + tint ramp, gated healthbar/sideselect/focus-visual
writes, change-gated `MouseUi`, structural-only settings-overlay rebuild) —
are done. Lot (c) — Bevy `States` migration (`GameState` + `InMatch` computed
state + `Winner` resource, overlays on `OnEnter`/`OnExit`, `run_if` gating,
physics paused outside Playing, `reset_match` on `OnEnter(Menu)` in game.rs)
and per-module `Plugin`s — is done too. Lot (d) — file/module split — is done:
`UnitStats` table + `spawn_combat_unit` + `Side::base_x()`, `free_march()` +
`CombatantKind::Unit(UnitKind)` in combat_tick, graphics application
consolidated into graphics.rs, constants split into `config/{units,weapons,
arena,world}.rs` (re-exported flat via `common`), and `ui.rs` split into
`ui/{mod,hud,overlays,input}.rs` + top-level `placement.rs`. Only the items
below remain; pick them opportunistically. (Longer-term idea parked from lot
d: per-kind AI systems sharing a once-per-frame combatant snapshot.)

## Unscheduled / smaller

- [ ] **Soldier first melee hit is instant** — `AttackCooldown::ready()` spawns
  finished and the soldier (unlike miner/priest) never resets on engage, so the
  first damage tick lands the frame contact starts while the slash clip starts
  at 0. Decide: reset on *first* engage (keep the documented no-reset rule for
  kiting) or accept. Animation/damage sync issue.
- [ ] **Settings key table** — `load_settings`/`save_settings` list every key
  twice; a shared `(key, accessor)` table prevents drift when adding params.
- [ ] **Menu slot counts** — `pause_input_system` hardcodes `SLOTS = 3`,
  `menu_input_system` hardcodes 4/1; must stay in sync with the buttons spawned
  in the overlay builders. Give each screen a single source of truth like the
  settings tab's `tab_slots()`.
- [ ] **`combat_tick` O(n²)** — per unit per frame: multiple linear scans +
  `volley_aim` is O(enemies²) per archer with 2 Vec allocs per call. Fine at
  POC scale; revisit with a profile beyond ~150–200 units (cache `volley_aim`
  per side, pre-partition combatants by side, then spatial grid if needed).
  Same family: `apply_player_focus_visual` still counts miners by iterating
  all units once per frame while in-match (cheap; cache per-slot counts if it
  ever shows in a profile).
- [ ] **Non-deterministic damage ordering** — `combat_tick`,
  `tower_attack_tick` and `arrow_flight_system` are declared parallel but
  conflict on `Health`, so Bevy serializes them in an ambiguous order (frame-
  to-frame nondeterminism in a versus game). Fix the order explicitly.
- [ ] **Dying tower** — keeps its collider during the 0.45 s collapse and can
  fire one last arrow on the frame HP hits 0 (`TowerDying` inserted a frame
  later). Cosmetic.
- [ ] **`debug_camera_control` ships in release** — gate behind
  `cfg(debug_assertions)` or a toggle so a player bumping the keyboard can't
  derail the camera.
- [ ] **`bind_unit_weapon_hand`** — does the hierarchy walk before checking the
  bone name; pre-filter on `name == "LeftHand" | "RightHand"` to skip the walk
  for every scenery prop `Name`.
- [ ] **Clippy const asserts** — the 3 `assertions_on_constants` warnings in
  tests: move those invariants to `const { assert!(…) }` outside tests so a
  violation breaks compilation, not just `cargo test`.
