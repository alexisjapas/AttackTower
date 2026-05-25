# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

POC of a 3D bilateral tower defense game (see `README.md` for the design spec). Two players share one screen; each buys units that walk straight toward the opposing base. First base destroyed loses.

Stack: **Rust + Bevy 0.18 + Avian3d 0.6**. Dev shell is a **Nix flake** (Vulkan, Wayland/X11, mold linker), loaded automatically via `direnv` when entering the directory.

Input is **gamepad-only**: two pads connect during the SideSelect screen, each player claims a side, and from then on every action (menu nav, unit purchase, tower placement) flows through that pad's bindings.

## Commands

```sh
cargo run                 # debug build + run
cargo run --release       # release build + run (use this for perf-sensitive testing)
cargo check               # fast type-check
cargo fmt
cargo clippy
```

No tests in this repository.

The Nix dev shell exports `LD_LIBRARY_PATH` for Vulkan/Wayland/X11. Running outside the shell will fail to find dynamic libraries — always run from within `nix develop` (or with direnv active).

### Cargo features (declared in `Cargo.toml`)

- `raytracing` (default) — pulls in `bevy_solari` for raytraced GI/shadows. Guarded everywhere with `#[cfg(feature = "raytracing")]`.
- `dlss` + `force_disable_dlss` (both default) — DLSS is *compiled in but mocked* by default so the build never needs the NVIDIA NGX SDK; at runtime `DlssRayReconstructionSupported` is absent and the menu entry stays "N/A". To run with real DLSS:
  ```sh
  cargo run --release --no-default-features --features raytracing,dlss
  ```

### Runtime resources

- Music: `setup_music` loads `assets/music/battleTheme.mp3` at startup (path defined in `music::MUSIC_PATH`; the `mp3` Cargo feature is enabled on Bevy). The file is intentionally optional — Bevy logs a "Path not found" error and the game runs silently if it is missing.
- Persisted settings: `~/.config/attack_tower/settings.cfg` (or `$XDG_CONFIG_HOME/attack_tower/settings.cfg`). Written by `persist_settings` whenever `GameSettings` changes; loaded once at startup by `load_settings` in place of `init_resource::<GameSettings>()`.

## Architecture

The whole game is a single Bevy `App` in `src/main.rs` that registers `DefaultPlugins`, `PhysicsPlugins::default()` (Avian, no colliders used yet — it is loaded so physics can be layered in later without re-architecting), and groups every gameplay system into five `SystemSet`s in `Update`:

1. **`AppSet::Input`** — gamepad-driven systems that mutate `GameState` (one per state). Chained because they all touch the same resource.
2. **`AppSet::World`** — gameplay tick: spawn / time / sun / combat / damage / animate / cleanup. The combat → damage → animate → cleanup chain is preserved; `spawn_*` and `animate_sun` run in parallel.
3. **`AppSet::React`** — systems that flip state (`check_winner`, `detect_pad_disconnect`) and the overlays that rebuild on `state.is_changed()`. Overlays run in parallel since they touch disjoint marker components.
4. **`AppSet::Visual`** — text refreshes, healthbar billboarding, settings application. Mostly parallel; Bevy infers ordering from query conflicts.
5. **`AppSet::FrameLimit`** — `limit_fps` only. Must be last so the sleep happens after every other system.

The sets are `.chain()`ed at configuration time so state changes propagate across them within a single frame. Adding a new system means picking the right set and using `.after(...)` only where there's an actual data dependency.

Modules:

- **`common.rs`** — shared types touched by every other module.
  - `Side` (`Left`/`Right`) — both a marker `Component` (on each unit/base/tower) and an enum used as a map key for `Gold`. When querying entities filter on the side component; when reading resources, pass the side enum.
  - `PlayerSlot` (`LeftBottom`/`LeftTop`/`RightBottom`/`RightTop`) — finer than `Side`. Each unit/base/tower carries both. In 1v1 only `LeftBottom`/`RightBottom` are active; in 2v2 all four. Used for per-slot gold, base Z offset, and miner ownership (so the right miner returns to the right base in 2v2).
  - Components: `Base`, `BaseDestroyed` (marker added when HP hits 0 so combat/tower targeting ignores it and the HUD greys out the panel), `Unit`, `Tower`, `Rock`, `Health`, `Damage`, `MoveSpeed`, `AttackCooldown`, plus animation helpers (`UnitAnim`, `UnitRig`), miner state (`MinerCarry`, `MinerPhase`, `MinerSlot`), `Arrow`, `Sun`, `TorchLight`/`TorchFlame`.
  - Resources: `Gold` (per-`PlayerSlot` pool), `GameState`, `GameMode` (`OneVsOne`/`TwoVsTwo`, selected on the SideSelect screen), `GameSettings`, `SettingsTab`, `SettingsOrigin`, `MatLibrary`, `PlacementMode`, `PlayerControllers`, `MenuFocus`, `TimeOfDay`, `GameTime`, `DlssAvailable`, `RaytracingAvailable`, `AtmosphereHandle`.
  - **All tunable constants live here** (HP, damage, gold cost, speeds, ranges, geometry, animation amplitudes, day/night period). Change game balance from one place.

- **`setup.rs`** — `Startup` systems and most static scene authoring. `init_mat_library` populates `MatLibrary` (must run before anything spawns meshes — enforced via `.chain()` in `main.rs`). `setup_world` spawns the camera (fixed 3/4 view with `Hdr`, atmosphere, bloom, fog), the sun, the ground, mountains, sky, zone markers, scenery (trees, bushes, grass, flowers). `spawn_arena` (an `Update` system) builds the castles and rocks lazily on the `Menu→Playing` transition so the layout can reflect the chosen `GameMode` (two or four bases). Also hosts the Solari/DLSS sync systems (gated on cargo features) and `update_torches` / `animate_sun`.

- **`units.rs`** — unit lifecycle, AI and rendering rigs.
  - Three unit kinds spawned by `spawn_soldier`, `spawn_miner`, `spawn_archer`. All units share a body+head+limbs rig (`UnitRig`) so `animate_units` can drive walking/attack/hurt/death animations from a single component (`UnitAnim`).
  - `combat_tick` is the heart of the game. It uses a `ParamSet` over three conflicting queries (units' `Transform`, bases' `Transform`, anyone's `Health`) and runs in three passes per frame: snapshot, decide (move vs. attack with `ENGAGE_RANGE`; allies directly ahead within ~one diameter form a queue), apply damage. Miners have their own multi-phase loop (`ToRock` → `Mining` → `Returning`) and feed `Gold` on deposit — there is no passive income.
  - Archers shoot via `spawn_arrow` + `arrow_flight_system` (parabolic trajectory; arrow despawns on impact and applies queued damage).
  - `process_damage_effects` and `cleanup_dead_units` close the loop.

- **`towers.rs`** — tower entity construction and aiming (`tower_attack_tick`, `cleanup_dead_towers`, validity helpers `is_valid_tower_zone` and `collides_with_existing_tower`). Each side may only build inside its own zone, defined by `ZONE_BOUNDARY` in `common.rs`. `is_valid_tower_zone` also takes `GameMode` so the Z bounds match the active lane layout (tighter in 1v1, wider in 2v2).

- **`healthbar.rs`** — billboarded health bars over units, towers and bases. `spawn_health_bar_for_{unit,tower,base}` are called by each spawner; `update_health_bars` runs every frame to position, billboard (Y-axis only), rescale the fill and re-tint green→red. Bars whose owner has despawned clean themselves up.

- **`game.rs`** — `check_winner` flips `GameState::Ended(winner)` as soon as a base hits 0 HP.

- **`graphics.rs`** — settings UX backend (no UI nodes here).
  - `GraphicsPreset` (Low/Medium/High/Ultra/Custom) — `apply` only touches quality fields (raytracing, dlss, taa, bloom, atmosphere, volumetric_fog, distance_fog), display fields (fullscreen, vsync, hdr, tonemapping) are preserved as user prefs. `Custom` is **derived**, never selected: `update_graphics_preset` runs `detect` whenever settings or DLSS availability change.
  - `ParamId` + `MenuSlot` + `tab_slots(tab)` — single source of truth for "which slots live on which tab and in what order". `ui.rs` builds buttons by iterating this; `settings_input_system` matches on the slot at the focused index. Adding a parameter means editing both `ParamId` and the tab arrays.
  - `param_description` returns functional + technical text + per-resource `Impact` (None/Low/Medium/High). Use `Impact::None` (renders as "none") for parameters with no measurable cost.
  - `load_settings` / `save_settings` / `persist_settings` — flat `key = value` text file, sanitised against missing build features on load.

- **`music.rs`** — `setup_music` spawns a paused looping `AudioPlayer<AudioSource>` tagged `GameMusic`. `sync_music_playback` calls `sink.play()` only while `GameState::Playing` (Paused, Menu, Settings, SideSelect, Ended all keep it silent). Also reacts to `Added<AudioSink>` so playback starts the moment the file finishes loading.

- **`ui.rs`** — every Bevy UI node lives here, plus the per-state input systems.
  - `setup_ui` builds the persistent HUD (top: in-game clock; bottom corners: one panel per `PlayerSlot` with the unit/tower buttons, the base HP readout and the gold counter). In 2v2 the top corners host the second pair of player panels.
  - State overlays: `update_menu_overlay`, `update_settings_overlay`, `update_pause_overlay`, `update_sideselect_overlay`, `update_endgame_overlay` — each rebuilds when `GameState` changes (the settings overlay also rebuilds on `SettingsTab` change). Despawn is implicitly recursive (Bevy 0.18 relationships) so removing the root removes the children.
  - Settings overlay is the most involved: a two-column layout (menu column + description card) with a tab strip (Video/Graphics) at the top, switched with LB/RB. Background is translucent so live setting changes are visible behind the menu. Description text is populated at spawn and refreshed by `update_settings_description` on `MenuFocus`/`SettingsTab`/`GraphicsPreset` change.
  - Input systems: `menu_input_system`, `sideselect_input_system`, `pause_input_system`, `settings_input_system`, `gameplay_input_system`, `placement_system` — all guard with `*state == GameState::X` and `!state.is_changed()` so the system that just transitioned the state does not also process the activation press.
  - `apply_graphics_settings` is the bridge from `GameSettings` back to the camera / window. It inserts or removes per-camera components (`Hdr`, `Bloom`, `Atmosphere`, `VolumetricFog`, `DistanceFog`, `TemporalAntiAliasing`) and mutates `Window.mode` / `Window.present_mode` / `Tonemapping`. Raytracing/DLSS toggling lives in `setup.rs` (feature-gated).

### Conventions and Bevy 0.18 specifics

- All gameplay systems early-return when `GameState != Playing`, so pause/end-game freezes movement, income, combat and music without touching the schedule.
- Camera setup pattern: `Camera3d::default()` + `Hdr` marker (`bevy::render::view::Hdr`) — HDR is no longer a field on `Camera`.
- Use `Camera3d` / `Mesh3d` / `MeshMaterial3d` component pattern (no bundles). Ambient light is configured via the `GlobalAmbientLight` resource (the per-camera `AmbientLight` component overrides it).
- `Time::delta_secs()` (not `delta_seconds`) for f32 delta time in this Bevy version.
- `Msaa::Off` is required wherever the deferred renderer is active (Solari forces deferred globally; TAA also forces MSAA off).
- Audio: `AudioPlayer<AudioSource>` is the spawnable component; the `AudioSink` component is added **asynchronously** by the audio backend once the source decodes — systems that pause/resume must tolerate the sink being absent for the first few frames.
