# AttackTower

POC of a bilateral 3D tower defense game, written in Rust with Bevy and Avian3d.

## Tech stack

- **Language**: Rust (edition 2024)
- **Engine**: Bevy 0.18 (pinned)
- **Physics**: Avian3d 0.6 (dynamic unit capsules driven by velocity + static building/rock colliders)
- **Dev environment**: Nix flake (Vulkan, Wayland/X11, mold linker)

## Prerequisites

The build needs Vulkan and Wayland/X11 development libraries. The repository ships a Nix dev shell that exports the right `LD_LIBRARY_PATH`. With `direnv` installed the shell loads automatically when you `cd` into the directory; otherwise run `nix develop` first. Building or running outside the shell will fail to find the dynamic libraries.

Two **gamepads** are required to play. Keyboard/mouse input is not supported.

## Concept

Real-time bilateral tower defense: one player on the left, one on the right (1v1), or two-vs-two on the same screen (2v2). Each player buys units that walk straight toward the opposing base. First base destroyed loses.

## Specifications

### Modes
- **1v1**: one base per side, centred on Z=0.
- **2v2**: two bases per side, offset on the Z axis. Each player owns one base and one miner economy; allies share the same colour but separate gold pools.

### Bases
- **40 HP** each
- Castle-style mesh assembled from primitives (foundation + keep + crenellated battlement + four corner towers).

### Units
| Unit    | HP | Damage | Cost | Speed | Cooldown | Range |
|---------|----|--------|------|-------|----------|-------|
| Soldier | 10 | 3      | 1    | 1.8   | 1.0 s    | melee |
| Archer  | 7  | 2      | 3    | 1.5   | 1.7 s    | 8.0   |
| Miner   | 8  | 0      | 4    | 1.4   | mining 1.1 s | — |

- Soldier and miner share a **procedural** body+head+limbs rig animated by transforms.
- The archer is a **rigged glTF model** (Meshy export) with skinned animations (walk, shot, death-and-fall). It pivots to aim, releases the arrow partway through the shot clip (at the loose, not the end), and the arrow leaves from its bow (left) hand.

### Towers
- **30 HP**, **3 damage**, **6 gold**, range **8.5**, cooldown **1.5 s**.
- Placed inside the player's own zone (terrain is split into three strips by `ZONE_BOUNDARY`; the Z extent of the zone adapts to `GameMode`). A ghost preview at the cursor turns green on a legal spot, red otherwise.
- Towers shoot physical arrows at the nearest enemy in range; the arrow damages whatever enemy it strikes (or plants on a miss). Killed towers tilt + sink briefly before despawning.

### Economy
- **10 gold** starting per player.
- Each player starts with one miner. Miners walk to their side's rock, mine in place, then return to their base to deposit (capacity 1 per trip). Max 5 miners per player. There is no passive income.

### Combat
- Units march straight toward the enemy base by default and only **redirect to an enemy that enters a short aggro radius** (`AGGRO_RADIUS`), then chase it within a leash and switch to a closer threat that crosses their path. A unit with no target that gets hit **turns on its attacker**. Melee triggers within `ENGAGE_RANGE` (1.4); archers shoot within `ARCHER_RANGE` (8.0).
- Archers hold their position and shoot the nearest enemy in range (no kiting/retreat).
- Units are dynamic Avian bodies: they separate from one another (no overlap or stacking) and are blocked by **enemy** buildings/rocks (they pass through their own) — collision is handled by the physics engine, not by hand.
- Archers aim each volley at the **densest enemy cluster** in range and loose a physical arrow at that spot. The arrow only damages an enemy it actually flies through (tested via Avian `SpatialQuery` — no friendly fire); a miss plants in the ground and fades. Being shot makes an idle unit turn on the shooter.

### Camera & map
- Horizontal map, bases aligned on the left/right axis.
- **Fixed** 3/4 camera at a shallow (~13°) downward angle, set below the mountain ridge height so the peaks rise above the eye-level horizon and are silhouetted against the sky (the camera height is the knob for this — see `CAMERA_DEFAULT_POS`).
- **Procedural sky** via Bevy's native `Atmosphere` (physical scattering, sun-coloured). A large ground plane plus distance fog dissolves the far terrain into the sky so there is no hard horizon line; a mountain ring frames the plain.
- The ground is painted in three regions (a generated base-color texture, quick fades between them): the **sand play field**, a **blue central no-man's-land**, and a **cooler decor tone** outside the play area.
- Designed for two-to-four players sharing the same screen (no split-screen).
- Day/night cycle (90 s period) drives sun position and torch lighting (torches inside castles and on towers light up at night).
- A **free-fly debug camera** (mouse + keyboard — otherwise unused since the game is gamepad-only) is available for development: hold RMB to look, WASD/Space/LShift to fly, LCtrl to boost, scroll to change speed, R to snap back to the default view. Active in every state.

### UI
- Persistent HUD: clock at the top, one player panel in each bottom corner (and top corners in 2v2). Each panel lists the unit/tower buttons, the player's base HP, gold, and currently focused stats.
- Endgame: **"Player X wins"** overlay with options to restart or return to menu.

### Settings
- Two-tab overlay (Video / Graphics) with a description column showing each parameter's role and its rendering-cost impact.
- Persisted to `~/.config/attack_tower/settings.cfg` (or `$XDG_CONFIG_HOME/attack_tower/`). Loaded once at startup; saved every time `GameSettings` changes.
- `GraphicsPreset` (Low/Medium/High/Ultra) only touches quality fields; Video parameters (fullscreen, vsync, HDR, tonemapping, **colorblind palette**) are preserved across preset switches. `Custom` is derived automatically when no preset matches.
- Colorblind palette swaps the Right side from red to orange for deuteranopia / protanopia.

### Input
- **Gamepad-only.** Two pads connect during the SideSelect screen, claim a side, then pick a nation, and from then on all input flows through that pad: D-pad navigates slots, A confirms (spawn / arm tower placement), X arms tower placement directly. Sticks drive the tower placement cursor.
- **SideSelect** is a per-pad flow: choose a seat → choose a nation → lock in. Each card shows the controller's name, its status, and the chosen nation. The match launches once at least one player is locked in and nobody is still mid nation-pick.

### Nations
- Players pick a nation after claiming a seat. Today there is only one — **Ada'Ram** — so the picker is scaffolding; the data model (`Nation` enum, per-player `PlayerNations`) is in place for nation-specific units/stats/visuals later.

## Roadmap

### Done
- 1v1 and 2v2 modes, shared-screen.
- Soldier / miner (procedural rigs) and a rigged glTF archer with animated, hand-anchored shots.
- Towers with ghost-preview placement, physical area-volley arrows, mining economy, and Avian-driven combat (dynamic-body units with march/aggro/retaliate AI, static building colliders).
- Day/night cycle with dynamic sun and torch lighting.
- Native procedural-atmosphere sky with fog-blended horizon and mountain backdrop.
- Raytraced lighting (Bevy Solari) with GPU auto-detection, plus a graphics-settings overlay (presets, per-parameter cost impact, colorblind palette) persisted to disk.
- Music, gamepad-driven menus, SideSelect with side claim + nation pick (controller name shown per card).
- Free-fly debug camera.

### Planned / ideas
- **More nations** with distinct units, stats and visuals (the nation pick is wired but only Ada'Ram exists).
- **Deeper physics use** — units, buildings and arrows already run on Avian (dynamic capsules, static colliders, `SpatialQuery` arrow hits); ragdoll deaths, debris or knockback could build on it.
- Additional unit types and abilities; sound effects.
- Possibly online or split-screen play (currently a single shared screen).
- Automated tests (none today).

### Known limitations
- **Archers are not lit by raytracing.** Bevy Solari does not yet support skinned/animated meshes (planned upstream but not scheduled for 0.20; it needs per-frame BLAS refit). The archer uses the rasterized fallback lighting; static geometry gets full GI. See `CLAUDE.md` and the Solari tracking issue.
- **Real DLSS is blocked.** The integration is wired but crashes inside the NVIDIA NGX SDK on Bevy 0.18, so it is mocked by default (`force_disable_dlss`); revisit on a Bevy upgrade.

## Cargo features

| Feature              | Default | Effect                                                                                     |
|----------------------|---------|--------------------------------------------------------------------------------------------|
| `raytracing`         | yes     | Pulls in `bevy_solari` for raytraced GI/shadows. Auto-disabled at startup on incapable GPUs. |
| `dlss`               | yes     | Compiles the DLSS code paths.                                                              |
| `force_disable_dlss` | yes     | Mocks DLSS at compile time so the NVIDIA NGX SDK isn't required.                           |

To run with real DLSS:
```sh
cargo run --release --no-default-features --features raytracing,dlss
```

## Running

```sh
# Inside the Nix dev shell (auto via direnv).
cargo run             # debug
cargo run --release   # release
cargo runf            # fast debug iteration (Bevy dynamic_linking; never for release)
```

See `CLAUDE.md` for an architectural tour.
