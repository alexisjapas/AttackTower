# Backlog

Findings from the June 2026 architecture/perf review. Lot (a) — quick wins
(Priest HUD stats, miner death doc, `FACE_TURN_*`/`ATTACK_HOLD` renames,
`shortest_yaw_diff` reuse, `load_settings` clamp dedup, single shared `Rng`,
`expect` → `let-else` in `combat_tick`) — and lot (b) — UI/healthbar perf
(shared `HealthBarAssets` + tint ramp, gated healthbar/sideselect/focus-visual
writes, change-gated `MouseUi`, structural-only settings-overlay rebuild) —
are done. Remaining lots ordered by suggested sequencing: (c) → (d), the rest
opportunistically.

## Lot (c) — architecture (cross-cutting, do before the code grows)

- [ ] **Migrate `GameState` to Bevy `States`** — biggest structural lever.
  Replaces: per-system `if *state != Playing` early-returns, the
  `!state.is_changed()` double-press guard in every input system, the
  poll-based overlay rebuilds (→ `OnEnter`/`OnExit`), the `spawn_arena` /
  `spawn_initial_miners` guards, and centralizes `reset_match`. `Ended(Side)`
  carries data → use an `Ended` state + a `Winner(Side)` resource.
- [ ] **Per-module `Plugin`s** — everything is registered in `main.rs` with 8
  glob imports. One plugin per module (`UnitsPlugin`, `UiPlugin`, …) owning its
  systems/resources; `main.rs` only assembles. Removes the glob imports.

## Lot (d) — file/module split (ideally alongside lot c)

- [ ] **Split `ui.rs` (~2.9k lines)** into `ui/hud.rs`, `ui/overlays.rs`,
  `ui/input.rs`, `placement.rs`. Move `apply_graphics_settings` to
  `graphics.rs` (camera/window mutation, not UI — CLAUDE.md already describes
  graphics.rs as the settings backend) and consolidate the graphics-application
  logic currently spread over ui.rs / setup.rs (`apply_raytracing_setting`,
  `apply_dlss_setting`) / graphics.rs. Move `reset_match` + `BattlefieldEntity`
  to `game.rs`.
- [ ] **Split `common.rs` (~1.5k lines)** — keep the "all tunables in one
  place" promise but per-domain (`consts_units`, `consts_env`,
  `consts_weapons`, …). The ~200-line weapon-placement block is an obvious
  standalone chunk.
- [ ] **`UnitKind::stats()` table** — per-kind hp/damage/cost/speed/cooldown/
  spawn-offset currently live in 3+ places (consts, `spawn_unit` match,
  `focus_stats_string`, `buy_or_place_slot` costs). A single `UnitStats` table
  makes the Priest-stats class of bug impossible. Also merge the identical
  `spawn_soldier`/`spawn_archer`/`spawn_priest` wrappers into
  `spawn_combat_unit(kind, lane)`, and add `Side::base_x()` (the
  `match side { Left => LEFT_BASE_X, … }` is copied 4×).
- [ ] **`combat_tick` structure** — the "nearest enemy base + `march_dir` +
  `formation_speed_factor`" block is copy-pasted in the Soldier, Archer and
  Priest branches → extract a `free_march()` helper. `CombatantKind` duplicates
  `UnitKind` → `enum CombatantKind { Unit(UnitKind), Base, Rock, Tower }`
  removes the mapping boilerplate. Longer term: per-kind AI systems sharing a
  once-per-frame snapshot.

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
- [ ] **Startup chain over-serialized** — all 6 Startup systems are
  `.chain()`ed; only `init_mat_library → setup_world` and
  `load_env_assets → setup_world` are real dependencies.
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
