# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

POC of a 3D bilateral tower defense game (see `README.md` for the full design spec, in French). Two players share one screen; each buys units that walk straight toward the opposite base. First base destroyed loses.

Stack: **Rust + Bevy 0.18 + Avian3d 0.6**. Dev shell is a **Nix flake** (Vulkan, Wayland/X11, mold linker) and is loaded automatically via `direnv` when entering the directory.

## Commands

```sh
cargo run        # build + run the game (uses mold via flake RUSTFLAGS)
cargo check      # fast type-check
cargo build      # debug build
cargo build --release
cargo fmt
cargo clippy
```

There are no tests yet.

The Nix dev shell exports `LD_LIBRARY_PATH` for Vulkan/Wayland/X11. Running outside the shell will fail to find dynamic libraries — always run from within `nix develop` (or with direnv active).

## Architecture

The game is a single Bevy `App` in `src/main.rs` that registers `DefaultPlugins`, `PhysicsPlugins::default()` (from Avian), and chains all gameplay systems in `Update`. The current chain order in `main.rs` matters: state changes propagate to UI within one frame because systems are explicitly `.chain()`ed.

Code is split into small focused modules:

- **`common.rs`** — shared types touched by every other module. Defines:
  - `Side` enum (`Left`/`Right`) with helpers (`forward()`, `color()`, `opposite()`, `label()`). Used both as a marker component and inside resources.
  - Components: `Base`, `Unit`, `Health { current, max }`, `Damage`, `MoveSpeed`, `AttackCooldown`.
  - Resources: `Gold { left, right }`, `IncomeTimer`, `MatLibrary` (cached mesh + material handles), `GameState` (`Playing` | `Ended(Side)`).
  - **All tunable constants live here** (HP, damage, gold cost, income interval, speeds, ranges, geometry). Change game balance from one place.

- **`setup.rs`** — `Startup` systems. `init_mat_library` populates `MatLibrary` (must run before anything spawns meshes — enforced via `.chain()` in `main.rs`). `setup_world` spawns the camera (fixed 3/4 view), directional light, ground plane, and both bases.

- **`units.rs`** — unit lifecycle and combat:
  - `spawn_unit(commands, lib, side)` is the single spawn entry point, called by the buy button system in `ui.rs` and (indirectly) used in tests of game balance. It applies a small Z jitter so stacked spawns don't perfectly overlap.
  - `combat_tick` is the heart of the game. It uses a `ParamSet` over three conflicting queries (units' `Transform`, bases' `Transform`, anyone's `Health`) and runs in three passes per frame:
    1. snapshot every combatant's position into a `Vec<Combatant>`,
    2. for each unit decide move-vs-attack (nearest enemy within `ENGAGE_RANGE` ⇒ attack on cooldown, else step forward; an ally directly ahead within roughly one diameter blocks movement to form a queue),
    3. apply queued damage events to targets.
  - `cleanup_dead_units` despawns units (not bases) when `Health.current <= 0`.

- **`economy.rs`** — `tick_income` advances `IncomeTimer` and grants both players +1 gold per tick. Skips when `GameState != Playing`.

- **`game.rs`** — `check_winner` flips `GameState` to `Ended(side.opposite())` as soon as a base hits 0 HP.

- **`ui.rs`** — all Bevy UI lives here:
  - `setup_ui` builds the persistent HUD (top: base HP texts, bottom: per-side Buy button + gold counter).
  - `buy_button_system` reads `Interaction` on `BuyButton(Side)`, spends gold, calls `spawn_unit`.
  - `update_gold_text` / `update_base_hp_text` refresh HUD text from the corresponding resource/query.
  - `update_endgame_overlay` reacts to `GameState` changes: on `Ended`, it spawns a full-screen overlay with the winner text and a Restart button; on `Playing`, it despawns any existing overlay. Despawn is implicitly recursive (Bevy 0.18 relationships), so removing the overlay also removes the Restart button.
  - `restart_button_system` despawns all units, resets each base's HP to its `max`, resets `Gold` and `IncomeTimer`, and sets `GameState::Playing`.

### Conventions and Bevy 0.18 specifics

- `Side` is intentionally both a `Component` (on each unit/base) **and** an enum used as a map key for `Gold`. When querying, filter on the side component; when reading resources, pass the side enum.
- All gameplay systems early-return when `GameState != Playing`, so pause/end-game freezes movement, income, and combat without touching the schedule.
- Bevy 0.18 uses the `Camera3d` / `Mesh3d` / `MeshMaterial3d` component pattern (not bundles). Ambient light is configured via the `GlobalAmbientLight` resource (the per-camera `AmbientLight` component overrides it).
- `Time::delta_secs()` (not `delta_seconds`) for f32 delta time in this Bevy version.
- Avian's `PhysicsPlugins` is loaded but no colliders or rigid bodies are used yet — movement is direct `Transform` mutation and combat is distance-based. The plugin is in place so physics can be layered on later without re-architecting.
