use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

use bevy::animation::RepeatAnimation;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;

use crate::common::*;

pub fn spawn_soldier(
    commands: &mut Commands,
    lib: &MatLibrary,
    slot: PlayerSlot,
    mode: GameMode,
    lane: usize,
) {
    let _ = spawn_unit(
        commands,
        lib,
        None,
        slot,
        mode,
        UnitKind::Soldier,
        Some(lane_z(lane, mode)),
    );
}

pub fn lane_z(lane: usize, mode: GameMode) -> f32 {
    if LANE_COUNT <= 1 {
        return 0.0;
    }
    let half = mode.lane_half_width();
    let step = (half * 2.0) / (LANE_COUNT as f32 - 1.0);
    -half + (lane.min(LANE_COUNT - 1) as f32) * step
}

pub fn spawn_miner(
    commands: &mut Commands,
    lib: &MatLibrary,
    slot: PlayerSlot,
    mode: GameMode,
    ring_slot: usize,
) {
    let entity = spawn_unit(commands, lib, None, slot, mode, UnitKind::Miner, None);
    commands.entity(entity).insert((
        MinerCarry::default(),
        MinerPhase::ToRock,
        MinerSlot(ring_slot),
    ));
}

/// Each active player slot starts with one miner so the economy works without
/// the player having to spend gold first. Runs on the Menu→Playing transition;
/// skips when units already exist (Paused→Playing resume).
pub fn spawn_initial_miners(
    state: Res<GameState>,
    mode: Res<GameMode>,
    mut commands: Commands,
    lib: Res<MatLibrary>,
    units: Query<Entity, With<Unit>>,
) {
    if !state.is_changed() || *state != GameState::Playing {
        return;
    }
    if units.iter().next().is_some() {
        return;
    }
    for &slot in mode.active_slots() {
        spawn_miner(&mut commands, &lib, slot, *mode, 0);
    }
}

pub fn miner_slot_offset(slot: usize, side: Side) -> Vec3 {
    // Slots spread across a 180° arc on the base-facing side of the rock, so
    // miners never need to cross the rock to reach their position. Slot 0 is
    // the leftmost arc position from the base's POV; the formula is
    // side-mirrored so both players see the same layout.
    let n = MAX_MINERS_PER_PLAYER as f32;
    let step = std::f32::consts::PI / n;
    let angle = (slot as f32 - (n - 1.0) / 2.0) * step;
    let x_local = angle.cos() * MINER_RING_RADIUS;
    let z_local = angle.sin() * MINER_RING_RADIUS;
    Vec3::new(x_local * side.forward(), 0.0, z_local)
}

pub fn spawn_archer(
    commands: &mut Commands,
    lib: &MatLibrary,
    assets: &ArcherAssets,
    slot: PlayerSlot,
    mode: GameMode,
    lane: usize,
) {
    let _ = spawn_unit(
        commands,
        lib,
        Some(assets),
        slot,
        mode,
        UnitKind::Archer,
        Some(lane_z(lane, mode)),
    );
}

fn spawn_unit(
    commands: &mut Commands,
    lib: &MatLibrary,
    assets: Option<&ArcherAssets>,
    slot: PlayerSlot,
    mode: GameMode,
    kind: UnitKind,
    fixed_z: Option<f32>,
) -> Entity {
    let side = slot.side();
    let base_x = match side {
        Side::Left => LEFT_BASE_X,
        Side::Right => RIGHT_BASE_X,
    };
    let base_z = slot.base_z(mode);
    let (spawn_x, hp, dmg, speed, cooldown) = match kind {
        UnitKind::Soldier => (
            base_x + side.forward() * SOLDIER_SPAWN_OFFSET,
            SOLDIER_HP,
            SOLDIER_DAMAGE,
            SOLDIER_SPEED,
            SOLDIER_COOLDOWN,
        ),
        UnitKind::Miner => (
            base_x - side.forward() * MINER_SPAWN_OFFSET,
            MINER_HP,
            0,
            MINER_SPEED,
            MINER_COOLDOWN,
        ),
        UnitKind::Archer => (
            base_x + side.forward() * ARCHER_SPAWN_OFFSET,
            ARCHER_HP,
            ARCHER_DAMAGE,
            ARCHER_SPEED,
            ARCHER_COOLDOWN,
        ),
    };
    let z = base_z
        + match fixed_z {
            Some(z) => z + (rand_jitter() - 0.5) * 0.25,
            None => (rand_jitter() - 0.5) * SPAWN_Z_JITTER * 2.0,
        };

    // The archer is a rigged glTF model, not the procedural capsule rig. Spawn
    // the scene as a child (with the scale + facing correction the model needs)
    // and tag the root so `animate_archer` drives it via an AnimationPlayer. No
    // `UnitRig`, so `animate_units` skips it.
    if matches!(kind, UnitKind::Archer) {
        let assets = assets.expect("archer spawn requires ArcherAssets");
        let rotation = unit_base_rotation(side, kind);
        let model = commands
            .spawn((
                SceneRoot(assets.scene.clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_y(ARCHER_MODEL_YAW_OFFSET),
                    scale: Vec3::splat(ARCHER_MODEL_SCALE),
                },
                Visibility::default(),
            ))
            .id();
        let entity = commands
            .spawn((
                Transform {
                    translation: Vec3::new(spawn_x, 0.0, z),
                    rotation,
                    scale: Vec3::ONE,
                },
                Visibility::default(),
                Unit,
                kind,
                side,
                slot,
                Health::new(hp),
                Damage(dmg),
                MoveSpeed(speed),
                AttackCooldown::ready(cooldown),
                UnitAnim::default(),
                ArcherModel,
                ArcherAnimState::default(),
            ))
            .add_children(&[model])
            .id();
        crate::healthbar::spawn_health_bar_for_unit(commands, entity);
        return entity;
    }

    let main_mat = match side {
        Side::Left => lib.left.clone(),
        Side::Right => lib.right.clone(),
    };
    let dark_mat = match side {
        Side::Left => lib.left_dark.clone(),
        Side::Right => lib.right_dark.clone(),
    };
    let rotation = unit_base_rotation(side, kind);

    let leg_left = spawn_limb_pivot(
        commands,
        Vec3::new(0.0, HIP_Y, LEG_SPREAD_Z),
        LEG_PIVOT_OFFSET,
        &lib.limb_mesh,
        &dark_mat,
    );
    let leg_right = spawn_limb_pivot(
        commands,
        Vec3::new(0.0, HIP_Y, -LEG_SPREAD_Z),
        LEG_PIVOT_OFFSET,
        &lib.limb_mesh,
        &dark_mat,
    );
    let arm_left = spawn_limb_pivot(
        commands,
        Vec3::new(0.0, ARM_SHOULDER_Y, ARM_SPREAD_Z),
        ARM_PIVOT_OFFSET,
        &lib.limb_mesh,
        &dark_mat,
    );
    let arm_right = spawn_limb_pivot(
        commands,
        Vec3::new(0.0, ARM_SHOULDER_Y, -ARM_SPREAD_Z),
        ARM_PIVOT_OFFSET,
        &lib.limb_mesh,
        &dark_mat,
    );

    let bob = commands
        .spawn((
            Transform::from_xyz(0.0, BOB_BASE_Y, 0.0),
            Visibility::default(),
        ))
        .with_children(|b| {
            b.spawn((
                Mesh3d(lib.body_mesh.clone()),
                MeshMaterial3d(main_mat.clone()),
                Transform::default(),
            ));
            b.spawn((
                Mesh3d(lib.head_mesh.clone()),
                MeshMaterial3d(main_mat.clone()),
                Transform::from_xyz(0.0, 0.32, 0.0),
            ))
            .with_children(|h| {
                h.spawn((
                    Mesh3d(lib.eye_mesh.clone()),
                    MeshMaterial3d(lib.eye_mat.clone()),
                    Transform::from_xyz(0.13, 0.03, 0.07),
                ));
                h.spawn((
                    Mesh3d(lib.eye_mesh.clone()),
                    MeshMaterial3d(lib.eye_mat.clone()),
                    Transform::from_xyz(0.13, 0.03, -0.07),
                ));
            });
        })
        .id();

    commands.entity(bob).add_children(&[arm_left, arm_right]);

    // Weapon attachment. The archer returns early above (glTF path) and never
    // reaches this procedural rig.
    match kind {
        UnitKind::Soldier => attach_spear(commands, arm_right, lib),
        UnitKind::Miner => attach_pickaxe(commands, arm_right, lib),
        UnitKind::Archer => {}
    }

    let entity = commands
        .spawn((
            Transform {
                translation: Vec3::new(spawn_x, 0.0, z),
                rotation,
                scale: Vec3::ONE,
            },
            Visibility::default(),
            Unit,
            kind,
            side,
            slot,
            Health::new(hp),
            Damage(dmg),
            MoveSpeed(speed),
            AttackCooldown::ready(cooldown),
            UnitAnim::default(),
            UnitRig {
                bob,
                leg_left,
                leg_right,
                arm_left,
                arm_right,
            },
        ))
        .add_children(&[bob, leg_left, leg_right])
        .id();
    crate::healthbar::spawn_health_bar_for_unit(commands, entity);
    entity
}

fn unit_base_rotation(side: Side, kind: UnitKind) -> Quat {
    let face_forward_world = match kind {
        UnitKind::Soldier | UnitKind::Archer => side.forward(),
        UnitKind::Miner => -side.forward(),
    };
    if face_forward_world > 0.0 {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_y(PI)
    }
}

fn spawn_limb_pivot(
    commands: &mut Commands,
    pos: Vec3,
    pivot_offset: f32,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) -> Entity {
    commands
        .spawn((Transform::from_translation(pos), Visibility::default()))
        .with_child((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, -pivot_offset, 0.0),
        ))
        .id()
}

fn attach_spear(commands: &mut Commands, arm: Entity, lib: &MatLibrary) {
    // Spear is held horizontally pointing in +X (forward). Original cylinder/cone
    // are Y-aligned; rotating -90° around Z aligns them with +X.
    let spear = commands
        .spawn((
            Transform {
                translation: Vec3::new(0.42, -0.34, 0.0),
                rotation: Quat::from_rotation_z(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
            Visibility::default(),
        ))
        .with_children(|s| {
            s.spawn((
                Mesh3d(lib.spear_shaft.clone()),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform::default(),
            ));
            // Cone apex points along its +Y, so the tip ends up at +X after the parent rotation.
            s.spawn((
                Mesh3d(lib.spear_tip.clone()),
                MeshMaterial3d(lib.metal_mat.clone()),
                Transform::from_xyz(0.0, 0.45, 0.0),
            ));
        })
        .id();
    commands.entity(arm).add_children(&[spear]);
}

fn attach_pickaxe(commands: &mut Commands, arm: Entity, lib: &MatLibrary) {
    // Pickaxe held by the hand: pivot at the hand, tilted 30° forward so the
    // handle clears the arm cylinder instead of intersecting it. Head sits
    // forward-and-up at the top of the handle.
    let pick = commands
        .spawn((
            Transform {
                translation: Vec3::new(0.0, -0.36, 0.0),
                rotation: Quat::from_rotation_z(-0.55),
                scale: Vec3::ONE,
            },
            Visibility::default(),
        ))
        .with_children(|p| {
            p.spawn((
                Mesh3d(lib.pickaxe_handle.clone()),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform::from_xyz(0.0, 0.27, 0.0),
            ));
            p.spawn((
                Mesh3d(lib.pickaxe_head.clone()),
                MeshMaterial3d(lib.metal_mat.clone()),
                Transform::from_xyz(0.10, 0.52, 0.0),
            ));
        })
        .id();
    commands.entity(arm).add_children(&[pick]);
}

pub fn rand_jitter() -> f32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEED: AtomicU32 = AtomicU32::new(0x1234_5678);
    // fetch_add then hash, so concurrent callers get distinct seeds even on
    // platforms with weak memory ordering. The previous load+store had a TOCTOU
    // race that could produce duplicate jitter values.
    let mut x = SEED
        .fetch_add(0x9E37_79B9, Ordering::Relaxed)
        .wrapping_add(0x9E37_79B9);
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    (x & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
}

#[derive(Clone, Copy)]
pub struct Combatant {
    entity: Entity,
    side: Side,
    slot: Option<PlayerSlot>,
    pos: Vec3,
    kind: CombatantKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CombatantKind {
    Soldier,
    Miner,
    Archer,
    Base,
    Rock,
    Tower,
}

pub fn combat_tick(
    time: Res<Time>,
    state: Res<GameState>,
    mut gold: ResMut<Gold>,
    // Reused across frames to avoid per-frame Vec allocations.
    mut combatants: Local<Vec<Combatant>>,
    mut damage_events: Local<Vec<(Entity, i32)>>,
    mut gold_events: Local<Vec<(PlayerSlot, u32)>>,
    mut sets: ParamSet<(
        Query<
            (
                Entity,
                &Side,
                &PlayerSlot,
                &UnitKind,
                &mut Transform,
                &Damage,
                &mut AttackCooldown,
                &MoveSpeed,
                &mut UnitAnim,
                Option<&mut MinerCarry>,
                Option<&mut MinerPhase>,
                Option<&MinerSlot>,
                Option<&mut ArcherAnimState>,
            ),
            With<Unit>,
        >,
        Query<(Entity, &Side, &PlayerSlot, &Transform), (With<Base>, Without<BaseDestroyed>)>,
        Query<(Entity, &Side, &PlayerSlot, &Transform), With<Rock>>,
        Query<&mut Health>,
        Query<(Entity, &Side, &Transform), (With<Tower>, Without<TowerDying>)>,
    )>,
) {
    if *state != GameState::Playing {
        for (_, _, _, _, _, _, _, _, mut anim, _, _, _, _) in sets.p0().iter_mut() {
            anim.walking = false;
            anim.attacking = false;
        }
        return;
    }

    // 1. Snapshot every combatant's position.
    combatants.clear();
    damage_events.clear();
    gold_events.clear();
    for (entity, side, slot, kind, transform, _, _, _, _, _, _, _, _) in sets.p0().iter() {
        let ckind = match *kind {
            UnitKind::Soldier => CombatantKind::Soldier,
            UnitKind::Miner => CombatantKind::Miner,
            UnitKind::Archer => CombatantKind::Archer,
        };
        combatants.push(Combatant {
            entity,
            side: *side,
            slot: Some(*slot),
            pos: transform.translation,
            kind: ckind,
        });
    }
    for (entity, side, slot, transform) in sets.p1().iter() {
        combatants.push(Combatant {
            entity,
            side: *side,
            slot: Some(*slot),
            pos: transform.translation,
            kind: CombatantKind::Base,
        });
    }
    for (entity, side, slot, transform) in sets.p2().iter() {
        combatants.push(Combatant {
            entity,
            side: *side,
            slot: Some(*slot),
            pos: transform.translation,
            kind: CombatantKind::Rock,
        });
    }
    for (entity, side, transform) in sets.p4().iter() {
        combatants.push(Combatant {
            entity,
            side: *side,
            slot: None,
            pos: transform.translation,
            kind: CombatantKind::Tower,
        });
    }

    let dt = time.delta_secs();

    // 2. Per-unit decision.
    for (
        entity,
        side,
        slot,
        kind,
        mut transform,
        damage,
        mut cooldown,
        speed,
        mut anim,
        mut carry_opt,
        mut phase_opt,
        slot_opt,
        mut arch_state,
    ) in sets.p0().iter_mut()
    {
        if anim.dying {
            anim.walking = false;
            anim.attacking = false;
            continue;
        }

        let pos = transform.translation;

        match *kind {
            UnitKind::Soldier => {
                let walk_sign = side.forward();
                let enemy = combatants
                    .iter()
                    .filter(|c| {
                        c.side != *side
                            && (c.kind == CombatantKind::Soldier
                                || c.kind == CombatantKind::Miner
                                || c.kind == CombatantKind::Archer
                                || c.kind == CombatantKind::Base
                                || c.kind == CombatantKind::Tower)
                    })
                    .map(|c| (c, xz_distance(c.pos, pos)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                if let Some((target, dist)) = enemy
                    && dist <= ENGAGE_RANGE
                {
                    if !anim.attacking {
                        // First frame in melee: restart the swing from the
                        // beginning so the animation doesn't pick up at a
                        // random fraction left over from a previous fight.
                        cooldown.0.reset();
                    }
                    cooldown.0.tick(time.delta());
                    anim.attacking = true;
                    anim.attack_phase = cooldown.0.fraction();
                    if cooldown.0.just_finished() {
                        damage_events.push((target.entity, damage.0));
                    }
                    anim.walking = false;
                    continue;
                }

                anim.attacking = false;

                if ally_blocking(
                    &combatants,
                    entity,
                    *side,
                    pos,
                    walk_sign,
                    CombatantKind::Soldier,
                ) || enemy_tower_blocking(&combatants, *side, pos, walk_sign)
                {
                    anim.walking = false;
                    continue;
                }
                let enemy_base = combatants
                    .iter()
                    .filter(|c| c.kind == CombatantKind::Base && c.side != *side)
                    .map(|c| (c, xz_distance(c.pos, pos)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(c, _)| c);
                if let Some(base) = enemy_base {
                    let target = Vec3::new(base.pos.x, 0.0, base.pos.z);
                    step_toward(&mut transform, target, speed.0 * dt);
                } else {
                    transform.translation.x += walk_sign * speed.0 * dt;
                }
                transform.translation.z +=
                    allied_tower_sidestep(&combatants, *side, pos, walk_sign, speed.0 * dt);
                anim.walking = true;
            }
            UnitKind::Miner => {
                // Filter by PlayerSlot, not Side: in 2v2 the two allied miners
                // must each return to their OWN rock and base.
                let own_rock = combatants
                    .iter()
                    .find(|c| c.slot == Some(*slot) && c.kind == CombatantKind::Rock);
                let own_base = combatants
                    .iter()
                    .find(|c| c.slot == Some(*slot) && c.kind == CombatantKind::Base);
                let ring_slot = slot_opt.map(|s| s.0).unwrap_or(0);

                let phase = phase_opt.as_deref().copied().unwrap_or(MinerPhase::ToRock);

                match phase {
                    MinerPhase::ToRock => {
                        anim.attacking = false;
                        let Some(rock) = own_rock else {
                            anim.walking = false;
                            continue;
                        };
                        let target = rock.pos + miner_slot_offset(ring_slot, *side);
                        let target_xz = Vec3::new(target.x, 0.0, target.z);
                        let pos_xz = Vec3::new(pos.x, 0.0, pos.z);
                        let dist = (target_xz - pos_xz).length();
                        // Once we're within one frame's worth of the slot, do
                        // the final step (step_toward clamps to dist, so this
                        // lands exactly on the target) and transition. No
                        // snap: the last frame of motion *is* the arrival.
                        if dist <= speed.0 * dt {
                            step_toward(&mut transform, target_xz, speed.0 * dt);
                            // Face the rock so the mining animation looks
                            // toward what's being hit, not the approach line.
                            let to_rock_x = rock.pos.x - transform.translation.x;
                            let to_rock_z = rock.pos.z - transform.translation.z;
                            let yaw = to_rock_z.atan2(to_rock_x);
                            transform.rotation = Quat::from_rotation_y(-yaw);
                            if let Some(phase) = phase_opt.as_deref_mut() {
                                *phase = MinerPhase::Mining;
                            }
                            anim.walking = false;
                            continue;
                        }
                        step_toward(&mut transform, target_xz, speed.0 * dt);
                        anim.walking = true;
                    }
                    MinerPhase::Mining => {
                        cooldown.0.tick(time.delta());
                        anim.attacking = true;
                        anim.attack_phase = cooldown.0.fraction();
                        anim.walking = false;
                        if cooldown.0.just_finished()
                            && let Some(carry) = carry_opt.as_deref_mut()
                        {
                            carry.current = carry.current.saturating_add(MINER_GOLD_PER_HIT);
                            if carry.current >= MINER_CAPACITY
                                && let Some(phase) = phase_opt.as_deref_mut()
                            {
                                *phase = MinerPhase::Returning;
                            }
                        }
                    }
                    MinerPhase::Returning => {
                        anim.attacking = false;
                        let Some(base) = own_base else {
                            anim.walking = false;
                            continue;
                        };
                        let target_xz = Vec3::new(base.pos.x, 0.0, base.pos.z);
                        let pos_xz = Vec3::new(pos.x, 0.0, pos.z);
                        let dist = (target_xz - pos_xz).length();
                        if dist <= MINER_DEPOSIT_RANGE {
                            if let Some(carry) = carry_opt.as_deref_mut()
                                && carry.current > 0
                            {
                                gold_events.push((*slot, carry.current));
                                carry.current = 0;
                            }
                            if let Some(phase) = phase_opt.as_deref_mut() {
                                *phase = MinerPhase::ToRock;
                            }
                            anim.walking = false;
                            continue;
                        }
                        step_toward(&mut transform, target_xz, speed.0 * dt);
                        anim.walking = true;
                    }
                }
            }
            UnitKind::Archer => {
                let walk_sign = side.forward();
                let enemy = combatants
                    .iter()
                    .filter(|c| {
                        c.side != *side
                            && (c.kind == CombatantKind::Soldier
                                || c.kind == CombatantKind::Miner
                                || c.kind == CombatantKind::Archer
                                || c.kind == CombatantKind::Base
                                || c.kind == CombatantKind::Tower)
                    })
                    .map(|c| (c, xz_distance(c.pos, pos)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                if let Some((target, dist)) = enemy
                    && dist <= ARCHER_RANGE
                {
                    // Pivot so the archer's left faces the target: the shot clip
                    // releases to the left, so this makes the arrow read as aimed
                    // straight at the target. animate_archer smoothly turns toward
                    // this yaw.
                    anim.face_yaw = face_angle(target.pos.x - pos.x, target.pos.z - pos.z)
                        + ARCHER_SHOT_YAW_OFFSET;
                    anim.attacking = true;
                    // Only queue a shot once the pivot is done (facing error
                    // within ARCHER_TURN_EPS), so the first arrow waits for the
                    // turn to finish. `animate_archer` releases the queued shot at
                    // the end of the shot clip's cycle; the cadence is the clip
                    // length (tuned to ARCHER_COOLDOWN). The kite/advance logic
                    // below still runs while turning so melee can't sneak in.
                    let current_yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
                    let aimed =
                        shortest_yaw_diff(anim.face_yaw, current_yaw).abs() <= ARCHER_TURN_EPS;
                    if let Some(arch_state) = arch_state.as_deref_mut() {
                        arch_state.pending_shot = aimed.then_some(PendingShot {
                            target: target.entity,
                            target_pos: target.pos,
                            damage: damage.0,
                        });
                    }
                    // Kite: an enemy that closes inside ARCHER_KITE_RANGE in
                    // front of us pushes us back so the archer stays at range
                    // instead of being slaughtered in melee. Slower than the
                    // normal walk so it reads as a careful retreat.
                    let target_ahead = (target.pos.x - pos.x) * walk_sign;
                    if dist < ARCHER_KITE_RANGE && target_ahead > 0.0 {
                        transform.translation.x -= walk_sign * speed.0 * 0.7 * dt;
                        anim.walking = true;
                    } else {
                        anim.walking = false;
                    }
                    continue;
                }

                anim.attacking = false;
                if let Some(arch_state) = arch_state.as_deref_mut() {
                    arch_state.pending_shot = None;
                }
                // No target: face the advance direction so it turns back to the
                // front (animate_archer smoothly pivots toward this yaw).
                anim.face_yaw = face_angle(walk_sign, 0.0);

                if ally_blocking(
                    &combatants,
                    entity,
                    *side,
                    pos,
                    walk_sign,
                    CombatantKind::Archer,
                ) || enemy_tower_blocking(&combatants, *side, pos, walk_sign)
                {
                    anim.walking = false;
                    continue;
                }
                let enemy_base = combatants
                    .iter()
                    .filter(|c| c.kind == CombatantKind::Base && c.side != *side)
                    .map(|c| (c, xz_distance(c.pos, pos)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(c, _)| c);
                if let Some(base) = enemy_base {
                    let target = Vec3::new(base.pos.x, 0.0, base.pos.z);
                    anim.face_yaw = face_angle(base.pos.x - pos.x, base.pos.z - pos.z);
                    move_toward_xz(&mut transform, target, speed.0 * dt);
                } else {
                    transform.translation.x += walk_sign * speed.0 * dt;
                }
                transform.translation.z +=
                    allied_tower_sidestep(&combatants, *side, pos, walk_sign, speed.0 * dt);
                anim.walking = true;
            }
        }
    }

    // 3. Apply damage and gold.
    let mut healths = sets.p3();
    for (target, dmg) in damage_events.drain(..) {
        if let Ok(mut hp) = healths.get_mut(target) {
            hp.current -= dmg;
        }
    }
    for (slot, amount) in gold_events.drain(..) {
        gold.add(slot, amount);
    }
}

fn enemy_tower_blocking(
    combatants: &[Combatant],
    self_side: Side,
    self_pos: Vec3,
    walk_sign: f32,
) -> bool {
    combatants.iter().any(|c| {
        if c.kind != CombatantKind::Tower || c.side == self_side {
            return false;
        }
        let dx_ahead = (c.pos.x - self_pos.x) * walk_sign;
        dx_ahead > 0.0
            && dx_ahead < UNIT_RADIUS + TOWER_RADIUS + 0.1
            && (c.pos.z - self_pos.z).abs() < UNIT_RADIUS + TOWER_RADIUS
    })
}

/// If an allied tower sits in the unit's forward corridor, return a lateral Z
/// step so the unit drifts around it. Returns 0 when no detour is needed.
fn allied_tower_sidestep(
    combatants: &[Combatant],
    self_side: Side,
    self_pos: Vec3,
    walk_sign: f32,
    step: f32,
) -> f32 {
    let look_ahead = UNIT_RADIUS + TOWER_RADIUS + 1.2;
    let corridor = UNIT_RADIUS + TOWER_RADIUS;
    let mut push: f32 = 0.0;
    for c in combatants {
        if c.kind != CombatantKind::Tower || c.side != self_side {
            continue;
        }
        let dx_ahead = (c.pos.x - self_pos.x) * walk_sign;
        let dz = self_pos.z - c.pos.z;
        if dx_ahead > 0.0 && dx_ahead < look_ahead && dz.abs() < corridor {
            // Push toward the freer Z side. If the unit is already centered on
            // the tower (dz≈0) use the sign of dz, defaulting to +1.
            let dir = if dz.abs() < 1e-3 { 1.0 } else { dz.signum() };
            push += dir * step;
        }
    }
    push
}

fn ally_blocking(
    combatants: &[Combatant],
    self_entity: Entity,
    self_side: Side,
    self_pos: Vec3,
    walk_sign: f32,
    kind: CombatantKind,
) -> bool {
    combatants.iter().any(|c| {
        if c.entity == self_entity || c.side != self_side || c.kind != kind {
            return false;
        }
        let dx_ahead = (c.pos.x - self_pos.x) * walk_sign;
        dx_ahead > 0.0
            && dx_ahead < UNIT_RADIUS * 2.0 + 0.05
            && (c.pos.z - self_pos.z).abs() < UNIT_RADIUS
    })
}

fn step_toward(transform: &mut Transform, target_xz: Vec3, step: f32) {
    let pos = transform.translation;
    let dx = target_xz.x - pos.x;
    let dz = target_xz.z - pos.z;
    let dist = (dx * dx + dz * dz).sqrt();
    if dist <= 1e-4 {
        return;
    }
    let s = step.min(dist);
    transform.translation.x += dx / dist * s;
    transform.translation.z += dz / dist * s;
    // Face direction of motion (rotate around Y).
    let yaw = dz.atan2(dx);
    transform.rotation = Quat::from_rotation_y(-yaw);
}

fn xz_distance(a: Vec3, b: Vec3) -> f32 {
    (a.x - b.x).hypot(a.z - b.z)
}

/// Entity yaw (rotation around Y) that faces the world XZ direction `(dx, dz)`,
/// matching `step_toward`'s `Quat::from_rotation_y` convention: `dx>0, dz=0`
/// gives 0 (faces +X).
fn face_angle(dx: f32, dz: f32) -> f32 {
    -dz.atan2(dx)
}

/// Shortest signed angular difference `target - current`, wrapped to
/// `(-PI, PI]`. Shared by `combat_tick` (to know when the archer is aimed) and
/// `animate_archer` (to step the pivot).
fn shortest_yaw_diff(target: f32, current: f32) -> f32 {
    let mut diff = (target - current).rem_euclid(std::f32::consts::TAU);
    if diff > PI {
        diff -= std::f32::consts::TAU;
    }
    diff
}

/// Translate toward an XZ target without touching rotation. The archer's facing
/// is driven separately (`UnitAnim.face_yaw` → `animate_archer`) so it can pivot
/// smoothly with the `Idle_Turn_*` clips instead of snapping like `step_toward`.
fn move_toward_xz(transform: &mut Transform, target_xz: Vec3, step: f32) {
    let pos = transform.translation;
    let dx = target_xz.x - pos.x;
    let dz = target_xz.z - pos.z;
    let dist = (dx * dx + dz * dz).sqrt();
    if dist <= 1e-4 {
        return;
    }
    let s = step.min(dist);
    transform.translation.x += dx / dist * s;
    transform.translation.z += dz / dist * s;
}

pub fn process_damage_effects(
    mut query: Query<(&Health, &mut UnitAnim), (With<Unit>, Changed<Health>)>,
) {
    for (hp, mut anim) in query.iter_mut() {
        let prev = anim.last_hp.replace(hp.current);
        if hp.current <= 0 {
            if !anim.dying {
                anim.dying = true;
                anim.death_t = 0.0;
            }
        } else if let Some(prev_hp) = prev
            && hp.current < prev_hp
        {
            anim.hurt_t = HURT_DURATION;
        }
    }
}

pub fn animate_units(
    time: Res<Time>,
    mut units: Query<(&Side, &UnitKind, &mut UnitAnim, &UnitRig, &mut Transform), With<Unit>>,
    mut transforms: Query<&mut Transform, Without<Unit>>,
) {
    let dt = time.delta_secs();
    let amp_lerp = (dt * 8.0).clamp(0.0, 1.0);

    for (side, kind, mut anim, rig, mut root_t) in units.iter_mut() {
        if anim.hurt_t > 0.0 {
            anim.hurt_t = (anim.hurt_t - dt).max(0.0);
        }

        if anim.dying {
            anim.death_t += dt;
            let progress = (anim.death_t / DEATH_DURATION).clamp(0.0, 1.0);
            // Tip forward around local Z (relative to the unit's facing direction).
            let base = unit_base_rotation(*side, *kind);
            root_t.rotation = base * Quat::from_rotation_z(-progress * FRAC_PI_2);
            anim.walking = false;
            anim.attacking = false;
            // Collapse limbs to neutral as the unit falls.
            anim.walk_amp = (anim.walk_amp - amp_lerp).max(0.0);
            reset_pose(&mut transforms, rig);
            continue;
        }

        let target_amp = if anim.walking { 1.0 } else { 0.0 };
        anim.walk_amp += (target_amp - anim.walk_amp) * amp_lerp;
        if anim.walking {
            anim.walk_phase += dt * WALK_FREQUENCY;
        }
        let walk_amp = anim.walk_amp;
        let phase = anim.walk_phase;
        let swing = phase.sin();
        let leg_angle = swing * LEG_SWING * walk_amp;
        let arm_angle = swing * ARM_SWING * walk_amp;
        let bob = (phase * 2.0).sin().abs() * BOB_AMPLITUDE * walk_amp;
        let hurt_tilt = if anim.hurt_t > 0.0 {
            (anim.hurt_t / HURT_DURATION) * HURT_TILT
        } else {
            0.0
        };

        let right_arm_angle = if anim.attacking {
            attack_arm_angle(*kind, anim.attack_phase)
        } else {
            arm_angle
        };

        if let Ok(mut t) = transforms.get_mut(rig.bob) {
            t.translation.y = BOB_BASE_Y + bob;
            t.rotation = Quat::from_rotation_z(hurt_tilt);
        }
        if let Ok(mut t) = transforms.get_mut(rig.leg_left) {
            t.rotation = Quat::from_rotation_z(leg_angle);
        }
        if let Ok(mut t) = transforms.get_mut(rig.leg_right) {
            t.rotation = Quat::from_rotation_z(-leg_angle);
        }
        if let Ok(mut t) = transforms.get_mut(rig.arm_left) {
            t.rotation = Quat::from_rotation_z(-arm_angle);
        }
        if let Ok(mut t) = transforms.get_mut(rig.arm_right) {
            t.rotation = Quat::from_rotation_z(right_arm_angle);
        }
    }
}

fn reset_pose(transforms: &mut Query<&mut Transform, Without<Unit>>, rig: &UnitRig) {
    if let Ok(mut t) = transforms.get_mut(rig.bob) {
        t.translation.y = BOB_BASE_Y;
        t.rotation = Quat::IDENTITY;
    }
    for limb in [rig.leg_left, rig.leg_right, rig.arm_left, rig.arm_right] {
        if let Ok(mut t) = transforms.get_mut(limb) {
            t.rotation = Quat::IDENTITY;
        }
    }
}

fn attack_arm_angle(kind: UnitKind, phase: f32) -> f32 {
    let p = phase.clamp(0.0, 1.0);
    match kind {
        UnitKind::Soldier => (p * PI).sin() * ATTACK_SWING_AMPLITUDE,
        UnitKind::Miner => {
            // Light wind-back, then a moderate forward tap with the head, then return.
            if p < 0.35 {
                -0.30 * (p / 0.35)
            } else if p < 0.55 {
                let k = (p - 0.35) / 0.20;
                -0.30 + 0.90 * k
            } else {
                let k = (p - 0.55) / 0.45;
                0.60 * (1.0 - k)
            }
        }
        UnitKind::Archer => {
            // Draw the bowstring back, hold briefly, then return to rest at release.
            // No forward overshoot so the motion reads as a bow shot, not a throw.
            if p < 0.75 {
                let k = (p / 0.75).powf(1.3);
                -0.80 * k
            } else if p < 0.88 {
                -0.80
            } else {
                let k = (p - 0.88) / 0.12;
                -0.80 * (1.0 - k)
            }
        }
    }
}

pub fn spawn_arrow(
    commands: &mut Commands,
    lib: &MatLibrary,
    side: Side,
    start: Vec3,
    target_entity: Entity,
    target_pos: Vec3,
    damage: i32,
) {
    // Aim at the target's chest so arrows visibly strike the body, not its feet.
    let aim = target_pos + Vec3::new(0.0, 0.55, 0.0);
    let dist = (aim - start).length();
    let total = (dist / ARROW_TRAVEL_SPEED).max(0.2);
    let apex = (dist * ARROW_ARC_FRACTION).max(ARROW_MIN_ARC);

    let main_mat = match side {
        Side::Left => lib.left.clone(),
        Side::Right => lib.right.clone(),
    };

    commands
        .spawn((
            Transform::from_translation(start),
            Visibility::default(),
            Arrow {
                start,
                target_entity,
                target_pos: aim,
                elapsed: 0.0,
                total,
                apex,
                damage,
            },
        ))
        .with_children(|a| {
            // The arrow's local +X is "forward"; the parent transform rotates so
            // +X follows the velocity vector each frame. Inside, the cylinder/cone
            // (originally Y-aligned) are rotated -90° around Z so they lie along +X.
            a.spawn((
                Mesh3d(lib.arrow_shaft.clone()),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_z(-FRAC_PI_2),
                    scale: Vec3::ONE,
                },
            ));
            a.spawn((
                Mesh3d(lib.arrow_tip.clone()),
                MeshMaterial3d(lib.metal_mat.clone()),
                Transform {
                    translation: Vec3::new(0.32, 0.0, 0.0),
                    rotation: Quat::from_rotation_z(-FRAC_PI_2),
                    scale: Vec3::ONE,
                },
            ));
            a.spawn((
                Mesh3d(lib.arrow_fletch.clone()),
                MeshMaterial3d(main_mat),
                Transform::from_xyz(-0.25, 0.0, 0.0),
            ));
        });
}

pub fn arrow_flight_system(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<GameState>,
    mut arrows: Query<(Entity, &mut Arrow, &mut Transform)>,
    targets: Query<&Transform, (Or<(With<Unit>, With<Base>, With<Tower>)>, Without<Arrow>)>,
    mut healths: Query<&mut Health>,
) {
    if *state != GameState::Playing {
        return;
    }
    let dt = time.delta_secs();
    for (entity, mut arrow, mut transform) in arrows.iter_mut() {
        arrow.elapsed += dt;
        // Light homing: keep aiming at the target's current chest position if it
        // still exists, so a slowly-moving target still gets hit.
        if let Ok(target_t) = targets.get(arrow.target_entity) {
            arrow.target_pos = target_t.translation + Vec3::new(0.0, 0.55, 0.0);
        }

        let t = (arrow.elapsed / arrow.total).clamp(0.0, 1.0);
        let start = arrow.start;
        let target = arrow.target_pos;
        let pos_y_linear = start.y + (target.y - start.y) * t;
        let arc = 4.0 * arrow.apex * t * (1.0 - t);
        let pos = Vec3::new(
            start.x + (target.x - start.x) * t,
            pos_y_linear + arc,
            start.z + (target.z - start.z) * t,
        );
        transform.translation = pos;

        // Orient the arrow along its velocity.
        let total_time = arrow.total.max(1e-3);
        let vx = (target.x - start.x) / total_time;
        let vz = (target.z - start.z) / total_time;
        let vy =
            (target.y - start.y) / total_time + (4.0 * arrow.apex / total_time) * (1.0 - 2.0 * t);
        let velocity = Vec3::new(vx, vy, vz);
        if velocity.length_squared() > 1e-6 {
            transform.rotation = Quat::from_rotation_arc(Vec3::X, velocity.normalize());
        }

        if t >= 1.0 {
            if let Ok(mut hp) = healths.get_mut(arrow.target_entity) {
                hp.current -= arrow.damage;
            }
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_dead_units(
    mut commands: Commands,
    query: Query<(Entity, &UnitAnim, &UnitKind), With<Unit>>,
) {
    for (entity, anim, kind) in &query {
        // The archer plays a longer glTF "fall" clip; hold its corpse until the
        // clip lands rather than despawning at the generic duration.
        let duration = match kind {
            UnitKind::Archer => ARCHER_DEATH_DURATION,
            _ => DEATH_DURATION,
        };
        if anim.dying && anim.death_t >= duration {
            commands.entity(entity).despawn();
        }
    }
}

/// glTF scenes instance their `AnimationPlayer` asynchronously on a descendant
/// (the Armature). Once the player exists and the shared graph is ready, attach
/// the graph + a transition mixer to the player and record the link back on the
/// owning archer root so `animate_archer` can drive it.
pub fn bind_archer_animation_player(
    mut commands: Commands,
    assets: Res<ArcherAssets>,
    players: Query<Entity, (With<AnimationPlayer>, Without<AnimationGraphHandle>)>,
    parents: Query<&ChildOf>,
    mut archers: Query<&mut ArcherAnimState, With<ArcherModel>>,
) {
    let Some(graph) = assets.graph.clone() else {
        return;
    };
    for player in &players {
        // Walk up the hierarchy to the archer root that owns this player.
        let mut current = player;
        let owner = loop {
            if archers.contains(current) {
                break Some(current);
            }
            match parents.get(current) {
                Ok(child_of) => current = child_of.parent(),
                Err(_) => break None,
            }
        };
        let Some(owner) = owner else { continue };
        commands.entity(player).insert((
            AnimationGraphHandle(graph.clone()),
            AnimationTransitions::new(),
        ));
        if let Ok(mut state) = archers.get_mut(owner) {
            state.player = Some(player);
        }
    }
}

/// The skeleton's bone entities (with their glTF `Name`s) are instanced
/// asynchronously with the scene. When the bow hand (`ARCHER_BOW_HAND_BONE`)
/// appears, walk up to the owning archer root and record it so `animate_archer`
/// can read its world position as the arrow's muzzle.
pub fn bind_archer_bow_hand(
    mut commands: Commands,
    bones: Query<(Entity, &Name), Added<Name>>,
    parents: Query<&ChildOf>,
    mut archers: Query<&mut ArcherAnimState, With<ArcherModel>>,
    assets: Res<ArcherAssets>,
) {
    for (bone, name) in &bones {
        if name.as_str() != ARCHER_BOW_HAND_BONE {
            continue;
        }
        let mut current = bone;
        let owner = loop {
            if archers.contains(current) {
                break Some(current);
            }
            match parents.get(current) {
                Ok(child_of) => current = child_of.parent(),
                Err(_) => break None,
            }
        };
        let Some(owner) = owner else { continue };
        if let Ok(mut state) = archers.get_mut(owner) {
            state.left_hand = Some(bone);
            commands.entity(bone).with_child((
                SceneRoot(assets.bow.clone()),
                Transform::from_translation(ARCHER_BOW_OFFSET)
                    .with_scale(Vec3::splat(ARCHER_BOW_SCALE)),
                ArcherBow,
            ));
        }
    }
}

/// Drives the glTF archer's `AnimationPlayer` from the same `UnitAnim` flags the
/// procedural `animate_units` reads (walk / attack / hurt / death). Archers carry
/// no `UnitRig`, so they are handled here instead of in `animate_units`; this
/// also owns the `hurt_t`/`death_t` bookkeeping for them.
pub fn animate_archer(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<ArcherAssets>,
    lib: Res<MatLibrary>,
    mut archers: Query<
        (&Side, &mut UnitAnim, &mut ArcherAnimState, &mut Transform),
        With<ArcherModel>,
    >,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    // World transforms of skeleton bones (the bow hand the arrow leaves from).
    bones: Query<&GlobalTransform>,
) {
    let dt = time.delta_secs();
    let Some(nodes) = assets.nodes else {
        return;
    };
    let hurt_count = nodes.hurts.len();
    let blend = Duration::from_secs_f32(0.15);

    for (side, mut anim, mut state, mut transform) in archers.iter_mut() {
        if anim.hurt_t > 0.0 {
            anim.hurt_t = (anim.hurt_t - dt).max(0.0);
        }
        if anim.dying {
            anim.death_t += dt;
        }

        // Smoothly rotate the whole archer toward the facing combat_tick asked
        // for. The body follows the entity, so it turns to aim at the target
        // without a dedicated turn clip (in-place Mixamo turn clips bake a fixed
        // rotation that fights this and snaps). Frozen while dying so the fall
        // clip owns the pose.
        if !anim.dying {
            let current_yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
            let mut diff = (anim.face_yaw - current_yaw).rem_euclid(std::f32::consts::TAU);
            if diff > PI {
                diff -= std::f32::consts::TAU;
            }
            if diff.abs() > ARCHER_TURN_EPS {
                let step = (ARCHER_TURN_SPEED * dt).min(diff.abs()) * diff.signum();
                transform.rotation = Quat::from_rotation_y(current_yaw + step);
            } else {
                transform.rotation = Quat::from_rotation_y(anim.face_yaw);
            }
        }

        // Rising edge of hurt_t = a fresh hit this frame.
        let fresh_hit = anim.hurt_t > state.last_hurt_t + 1e-4;
        state.last_hurt_t = anim.hurt_t;

        // Keep "attacking" latched for a moment after the target leaves range so
        // the shot pose doesn't flicker back to idle between volleys.
        if anim.attacking {
            state.attack_hold = ARCHER_ATTACK_HOLD;
        } else if state.attack_hold > 0.0 {
            state.attack_hold = (state.attack_hold - dt).max(0.0);
        }
        let attacking = anim.attacking || state.attack_hold > 0.0;

        let Some(player_entity) = state.player else {
            continue;
        };
        let Ok((mut player, mut transitions)) = players.get_mut(player_entity) else {
            continue;
        };

        // Has the current one-shot (a hurt reaction) finished playing? When it
        // does, `snap` makes the return to the next clip a hard cut so the
        // reaction never blends on top of the shot/walk pose.
        let mut snap = false;
        if state.oneshot_active {
            let finished = transitions
                .get_main_animation()
                .and_then(|n| player.animation(n))
                .map(|a| a.is_finished())
                .unwrap_or(true);
            if finished {
                state.oneshot_active = false;
                snap = true;
            }
        }

        // Priority: death > fresh hit (only when stationary, so it never breaks
        // the walk) > finish current hurt > attack > walk > idle.
        if anim.dying {
            if state.current != ArcherClip::Death {
                transitions
                    .play(&mut player, nodes.death, blend)
                    .set_repeat(RepeatAnimation::Never)
                    .set_speed(nodes.death_speed);
                state.current = ArcherClip::Death;
                state.oneshot_active = false;
            }
            continue;
        }

        if fresh_hit && !anim.walking {
            // Reactions hard-cut in (Duration::ZERO) so the flinch replaces the
            // shot pose instead of blending over it.
            let node = nodes.hurts[state.hurt_index % hurt_count];
            state.hurt_index = (state.hurt_index + 1) % hurt_count;
            transitions
                .play(&mut player, node, Duration::ZERO)
                .set_repeat(RepeatAnimation::Never);
            state.current = ArcherClip::Hurt;
            state.oneshot_active = true;
            continue;
        }

        if state.oneshot_active {
            continue;
        }

        // Hard cut when leaving a reaction; short crossfade otherwise.
        let enter = if snap { Duration::ZERO } else { blend };

        if attacking {
            if state.current != ArcherClip::Attack {
                // Loop the shot clip at a speed that fits one full draw-release
                // into the attack cooldown. The clip length sets the firing
                // cadence; the arrow leaves at the end of each cycle.
                transitions
                    .play(&mut player, nodes.attack, enter)
                    .repeat()
                    .set_speed(nodes.attack_speed);
                state.current = ArcherClip::Attack;
                state.last_attack_seek = 0.0;
            }
            // Release one arrow per cycle, the moment playback crosses the release
            // point (a bit before the clip ends, where the bow arm lowers), from
            // the bow hand's world position. combat_tick leaves `pending_shot` set
            // only while aimed, so the first arrow waits out the turn and there
            // are no stray shots.
            let seek = player
                .animation(nodes.attack)
                .map(|a| a.seek_time())
                .unwrap_or(0.0);
            let release_t = ARCHER_SHOT_RELEASE_FRACTION * nodes.attack_len;
            let crossed = state.last_attack_seek < release_t && seek >= release_t;
            state.last_attack_seek = seek;
            if crossed && let Some(shot) = state.pending_shot {
                let start = state
                    .left_hand
                    .and_then(|h| bones.get(h).ok())
                    .map(|gt| gt.translation())
                    .unwrap_or_else(|| {
                        transform.translation + transform.rotation * ARCHER_HAND_OFFSET
                    });
                spawn_arrow(
                    &mut commands,
                    &lib,
                    *side,
                    start,
                    shot.target,
                    shot.target_pos,
                    shot.damage,
                );
            }
        } else if anim.walking {
            if state.current != ArcherClip::Walk {
                transitions
                    .play(&mut player, nodes.walk, enter)
                    .repeat()
                    .set_speed(1.0);
                state.current = ArcherClip::Walk;
            }
        } else if state.current != ArcherClip::Idle {
            // No dedicated idle clip: hold the walk clip's first frame as a
            // standing pose.
            transitions
                .play(&mut player, nodes.walk, enter)
                .set_repeat(RepeatAnimation::Never)
                .set_speed(0.0)
                .seek_to(0.0);
            state.current = ArcherClip::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_z_spreads_symmetrically() {
        let half = GameMode::OneVsOne.lane_half_width();
        let first = lane_z(0, GameMode::OneVsOne);
        let last = lane_z(LANE_COUNT - 1, GameMode::OneVsOne);
        assert!((first + half).abs() < 1e-4);
        assert!((last - half).abs() < 1e-4);
    }

    #[test]
    fn lane_z_clamps_out_of_range_indices() {
        let last = lane_z(LANE_COUNT - 1, GameMode::OneVsOne);
        let beyond = lane_z(LANE_COUNT + 100, GameMode::OneVsOne);
        assert!((last - beyond).abs() < 1e-4);
    }

    #[test]
    fn lane_z_tighter_in_2v2_than_1v1() {
        let edge_1v1 = lane_z(0, GameMode::OneVsOne).abs();
        let edge_2v2 = lane_z(0, GameMode::TwoVsTwo).abs();
        assert!(edge_1v1 > edge_2v2);
    }

    #[test]
    fn miner_ring_is_on_the_correct_side() {
        let off_left = miner_slot_offset(0, Side::Left);
        let off_right = miner_slot_offset(0, Side::Right);
        // Slot 0 sits on the +X-facing arc relative to the side's forward.
        assert!(off_left.x > 0.0);
        assert!(off_right.x < 0.0);
    }

    #[test]
    fn miner_slot_offset_inside_ring_radius() {
        for slot in 0..MAX_MINERS_PER_PLAYER {
            let off = miner_slot_offset(slot, Side::Left);
            let r = (off.x * off.x + off.z * off.z).sqrt();
            assert!((r - MINER_RING_RADIUS).abs() < 1e-3);
        }
    }

    #[test]
    fn rand_jitter_in_unit_interval_and_not_constant() {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for _ in 0..256 {
            let v = rand_jitter();
            assert!((0.0..1.0).contains(&v));
            min = min.min(v);
            max = max.max(v);
        }
        // 256 samples should cover a non-trivial range, otherwise something's
        // wrong with the seed advance.
        assert!(max - min > 0.5);
    }
}
