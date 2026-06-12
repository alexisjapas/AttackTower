//! Tower placement: the per-player placement seat (armed from the HUD),
//! the gamepad/mouse-driven ghost cursor, validity checks and the actual
//! tower purchase. Cleared on OnExit(InMatch).

use bevy::prelude::*;

use crate::common::*;
use crate::towers::{collides_with_existing_tower, is_valid_tower_zone, spawn_tower};

/// OnExit(InMatch): cancel every in-flight tower placement and its preview
/// ghost. While merely paused (still InMatch) the ghosts stay visible in
/// place — `placement_system` simply doesn't run outside Playing.
pub fn clear_placement(
    mut commands: Commands,
    mut placement: ResMut<PlacementMode>,
    ghosts: Query<Entity, With<TowerGhost>>,
) {
    *placement = PlacementMode::default();
    for e in &ghosts {
        commands.entity(e).despawn();
    }
}

pub fn placement_system(
    mut commands: Commands,
    time: Res<Time>,
    mode: Res<GameMode>,
    lib: Res<MatLibrary>,
    env: Res<EnvAssets>,
    mut placement: ResMut<PlacementMode>,
    mut gold: ResMut<Gold>,
    players: Res<PlayerControllers>,
    gamepads: Query<&Gamepad>,
    ghosts: Query<(Entity, &PlayerSlot), With<TowerGhost>>,
    existing_towers: Query<&Transform, With<Tower>>,
    alive_bases: Query<&PlayerSlot, (With<Base>, Without<BaseDestroyed>)>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    windows: Query<&Window>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    let mut alive = [false; 4];
    for slot in &alive_bases {
        alive[slot.index()] = true;
    }

    for &slot in mode.active_slots() {
        // Despawn any existing ghost for this slot (will respawn below if still placing).
        for (e, ghost_slot) in &ghosts {
            if *ghost_slot == slot {
                commands.entity(e).despawn();
            }
        }

        if !alive[slot.index()] {
            placement.clear(slot);
            continue;
        }

        let Some(seat) = placement.get(slot) else {
            continue;
        };

        // Swallow the frame that armed placement so the activating press/click
        // isn't also read as a confirm.
        if !seat.armed {
            placement.set(
                slot,
                PlacementSeat {
                    world_pos: seat.world_pos,
                    armed: true,
                },
            );
            continue;
        }

        let tower_positions: Vec<Vec3> = existing_towers.iter().map(|t| t.translation).collect();

        match players.get(slot) {
            // Gamepad-driven: left stick moves the cursor, A places, B cancels.
            Some(pad_entity) => {
                let Ok(pad) = gamepads.get(pad_entity) else {
                    placement.clear(slot);
                    continue;
                };
                if pad.just_pressed(GamepadButton::East) {
                    placement.clear(slot);
                    continue;
                }
                let stick = pad.left_stick();
                let dt = time.delta_secs();
                let dx = if stick.x.abs() > GAMEPAD_STICK_DEADZONE {
                    stick.x
                } else {
                    0.0
                };
                let dz = if stick.y.abs() > GAMEPAD_STICK_DEADZONE {
                    stick.y
                } else {
                    0.0
                };
                let mut pos = seat.world_pos;
                pos.x += dx * GAMEPAD_CURSOR_SPEED * dt;
                // Stick Y positive = up on screen → -Z in world.
                pos.z -= dz * GAMEPAD_CURSOR_SPEED * dt;
                let confirm = pad.just_pressed(GamepadButton::South);
                place_tower_at(
                    &mut commands,
                    &lib,
                    &env,
                    &mut gold,
                    &mut placement,
                    &tower_positions,
                    slot,
                    *mode,
                    pos,
                    confirm,
                );
            }
            // Mouse-driven (controller-less debug): the ghost tracks the cursor's
            // ground projection, left-click places, right-click cancels.
            None => {
                if mouse_buttons.just_pressed(MouseButton::Right) {
                    placement.clear(slot);
                    continue;
                }
                let pos = windows
                    .single()
                    .ok()
                    .and_then(|w| w.cursor_position())
                    .zip(camera.single().ok())
                    .and_then(|(cursor, (cam, cam_tf))| cursor_ground_pos(cam, cam_tf, cursor))
                    .map(|p| Vec3::new(p.x, 0.0, p.z))
                    .unwrap_or(seat.world_pos);
                let confirm = mouse_buttons.just_pressed(MouseButton::Left);
                place_tower_at(
                    &mut commands,
                    &lib,
                    &env,
                    &mut gold,
                    &mut placement,
                    &tower_positions,
                    slot,
                    *mode,
                    pos,
                    confirm,
                );
            }
        }
    }
}

/// Shared tail of tower placement: validate `pos`, and either spend gold + spawn
/// the tower (when `confirm` and the spot is valid) or (re)spawn the placement
/// ghost tinted by validity. Returns true if a tower was placed. Used by both
/// the gamepad and mouse placement paths in [`placement_system`].
fn place_tower_at(
    commands: &mut Commands,
    lib: &MatLibrary,
    env: &EnvAssets,
    gold: &mut Gold,
    placement: &mut PlacementMode,
    tower_positions: &[Vec3],
    slot: PlayerSlot,
    mode: GameMode,
    pos: Vec3,
    confirm: bool,
) -> bool {
    let valid = is_valid_tower_zone(slot.side(), pos, mode)
        && !collides_with_existing_tower(pos, tower_positions)
        && gold.get(slot) >= TOWER_COST;
    if confirm && valid && gold.try_spend(slot, TOWER_COST) {
        spawn_tower(commands, lib, env, slot, Vec3::new(pos.x, 0.0, pos.z));
        placement.clear(slot);
        return true;
    }
    placement.set(
        slot,
        PlacementSeat {
            world_pos: pos,
            armed: true,
        },
    );
    let mat = if valid {
        lib.ghost_valid_mat.clone()
    } else {
        lib.ghost_invalid_mat.clone()
    };
    commands.spawn((
        Mesh3d(lib.tower_ghost_mesh.clone()),
        MeshMaterial3d(mat),
        Transform::from_xyz(pos.x, TOWER_HEIGHT * 0.5, pos.z),
        TowerGhost,
        slot,
    ));
    false
}

/// Project a screen-space cursor position onto the world ground plane (y = 0)
/// through `camera`, for mouse tower placement.
fn cursor_ground_pos(camera: &Camera, cam_tf: &GlobalTransform, cursor: Vec2) -> Option<Vec3> {
    let ray = camera.viewport_to_world(cam_tf, cursor).ok()?;
    let dist = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    Some(ray.get_point(dist))
}

pub fn arm_placement(placement: &mut PlacementMode, slot: PlayerSlot, mode: GameMode) {
    placement.set(
        slot,
        PlacementSeat {
            world_pos: default_placement_pos(slot, mode),
            armed: false,
        },
    );
}

fn default_placement_pos(slot: PlayerSlot, mode: GameMode) -> Vec3 {
    let x = match slot.side() {
        Side::Left => (LEFT_BASE_X + TOWER_PLACEMENT_MARGIN - ZONE_BOUNDARY) * 0.5,
        Side::Right => (ZONE_BOUNDARY + RIGHT_BASE_X - TOWER_PLACEMENT_MARGIN) * 0.5,
    };
    Vec3::new(x, 0.0, slot.base_z(mode))
}
