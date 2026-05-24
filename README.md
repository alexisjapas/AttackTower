# AttackTower

POC of a bilateral 3D tower defense game, written in Rust with Bevy and Avian3d.

## Tech stack

- **Language**: Rust
- **Engine**: Bevy (latest version, picked by cargo)
- **Physics**: Avian3d
- **Dev environment**: Nix flake (Vulkan, Wayland/X11, mold linker)

## Concept

Real-time bilateral tower defense: one player on the left, one on the right. Each player buys units that walk straight toward the opposing base. First to destroy the opposing base wins.

For this POC, both players are controlled from the same machine (local testing).

## Specifications

### Bases
- 1 base per player (left / right)
- **20 HP** each
- Representation: colored cube

### Units
- **10 HP**, **3 attack**
- Cost: **1 gold**
- Walk straight toward the opposing base
- Representation: colored cylinder

### Combat
- **Unit melee**: two opposing units that meet stop and attack each other until one dies
- **Base attack**: on contact, a unit deals its damage in a loop until it dies or destroys the base

### Economy
- **10 gold** starting per player
- **Steady passive income**: +1 gold every X seconds per player

### Camera & map
- Horizontal map, bases aligned on the left/right axis
- **Fixed** camera in the middle, 3/4 view from 45° above
- Far enough to see both bases simultaneously
- Designed for two players sharing the same screen (no split-screen)

### UI
- **Two unit purchase buttons** (one per player) at the bottom of the screen
- Gold counter visible next to each button

### Endgame
- When a base reaches 0 HP: **"Player X wins"** text displayed
- **Restart button** to start a new game

### Art direction (POC)
- Primitive geometric shapes (cubes, cylinders)
- Solid colors (uniform green ground, distinct colors per side)
- No textures or imported models

## Running the project

```sh
# Inside the Nix dev shell (auto via direnv)
cargo run
```
