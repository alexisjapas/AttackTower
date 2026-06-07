use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

use avian3d::prelude::*;
use bevy::animation::RepeatAnimation;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;

use crate::common::*;

pub fn spawn_soldier(
    commands: &mut Commands,
    models: &UnitModels,
    slot: PlayerSlot,
    mode: GameMode,
    lane: usize,
) {
    let _ = spawn_unit(
        commands,
        models,
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
    models: &UnitModels,
    slot: PlayerSlot,
    mode: GameMode,
    ring_slot: usize,
) {
    let entity = spawn_unit(commands, models, slot, mode, UnitKind::Miner, None);
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
    models: Res<UnitModels>,
    units: Query<Entity, With<Unit>>,
) {
    if !state.is_changed() || *state != GameState::Playing {
        return;
    }
    if units.iter().next().is_some() {
        return;
    }
    for &slot in mode.active_slots() {
        spawn_miner(&mut commands, &models, slot, *mode, 0);
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
    models: &UnitModels,
    slot: PlayerSlot,
    mode: GameMode,
    lane: usize,
) {
    let _ = spawn_unit(
        commands,
        models,
        slot,
        mode,
        UnitKind::Archer,
        Some(lane_z(lane, mode)),
    );
}

pub fn spawn_priest(
    commands: &mut Commands,
    models: &UnitModels,
    slot: PlayerSlot,
    mode: GameMode,
    lane: usize,
) {
    let _ = spawn_unit(
        commands,
        models,
        slot,
        mode,
        UnitKind::Priest,
        Some(lane_z(lane, mode)),
    );
}

fn spawn_unit(
    commands: &mut Commands,
    models: &UnitModels,
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
        // The priest is support-only: no attack damage.
        UnitKind::Priest => (
            base_x + side.forward() * PRIEST_SPAWN_OFFSET,
            PRIEST_HP,
            0,
            PRIEST_SPEED,
            PRIEST_COOLDOWN,
        ),
    };
    let z = base_z
        + match fixed_z {
            Some(z) => z + (rand_jitter() - 0.5) * 0.25,
            None => (rand_jitter() - 0.5) * SPAWN_Z_JITTER * 2.0,
        };

    // Every unit is a rigged glTF model: spawn the scene as a child (scale +
    // model-forward→facing yaw correction) and drive it via `animate_unit_model`
    // through the async-instanced `AnimationPlayer`. The weapon is attached to a
    // hand bone later by `bind_unit_weapon_hand`.
    let model = models.get(kind);
    let rotation = unit_base_rotation(side, kind);
    let scene_child = commands
        .spawn((
            SceneRoot(model.scene.clone()),
            Transform {
                translation: Vec3::ZERO,
                rotation: Quat::from_rotation_y(model.yaw_offset),
                scale: Vec3::splat(model.scale),
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
            Armor::default(),
            ModeledUnit,
            // Nested so the outer bundle tuple stays within Bevy's element limit.
            (UnitAnimState::default(), CombatTarget::default()),
            // Physics: a dynamic capsule driven by LinearVelocity in `combat_tick`.
            // Rotation + Y are locked so it stays upright on the plane; Avian
            // resolves all unit↔unit separation and structure blocking.
            (
                RigidBody::Dynamic,
                Collider::capsule(UNIT_RADIUS, UNIT_CAPSULE_LENGTH),
                LockedAxes::ROTATION_LOCKED.lock_translation_y(),
                LinearVelocity::default(),
                Friction::new(0.0),
                // Units frequently stop (combat/mining) then resume; never let a
                // resting body sleep, or a re-set velocity could be ignored.
                SleepingDisabled,
                side.unit_layers(),
            ),
        ))
        .add_children(&[scene_child])
        .id();
    crate::healthbar::spawn_health_bar_for_unit(commands, entity);
    entity
}

fn unit_base_rotation(side: Side, kind: UnitKind) -> Quat {
    let face_forward_world = match kind {
        UnitKind::Soldier | UnitKind::Archer | UnitKind::Priest => side.forward(),
        UnitKind::Miner => -side.forward(),
    };
    if face_forward_world > 0.0 {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_y(PI)
    }
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
    Priest,
    Base,
    Rock,
    Tower,
}

impl CombatantKind {
    /// True for the mobile fighting/support units (everything that targets or is
    /// targeted as an enemy unit, excluding structures and rocks).
    fn is_unit(self) -> bool {
        matches!(
            self,
            CombatantKind::Soldier
                | CombatantKind::Miner
                | CombatantKind::Archer
                | CombatantKind::Priest
        )
    }
}

pub fn combat_tick(
    time: Res<Time>,
    state: Res<GameState>,
    mut gold: ResMut<Gold>,
    // Reused across frames to avoid per-frame Vec allocations.
    mut combatants: Local<Vec<Combatant>>,
    // (target, damage, source). `source` lets the victim remember who hit it.
    mut damage_events: Local<Vec<(Entity, i32, Option<Entity>)>>,
    // (victim, attacker) → CombatTarget.last_attacker, applied after damage.
    mut attacker_events: Local<Vec<(Entity, Entity)>>,
    mut gold_events: Local<Vec<(PlayerSlot, u32)>>,
    // Priest support: HP to restore (clamped to max) and (armor, duration) to grant.
    mut heal_events: Local<Vec<(Entity, i32)>>,
    mut buff_events: Local<Vec<(Entity, i32, f32)>>,
    mut sets: ParamSet<(
        Query<
            (
                Entity,
                &Side,
                &PlayerSlot,
                &UnitKind,
                &mut Transform,
                &mut LinearVelocity,
                &Damage,
                &mut AttackCooldown,
                &MoveSpeed,
                &mut UnitAnim,
                Option<&mut MinerCarry>,
                Option<&mut MinerPhase>,
                Option<&MinerSlot>,
                Option<&mut UnitAnimState>,
            ),
            With<Unit>,
        >,
        Query<(Entity, &Side, &PlayerSlot, &Transform), (With<Base>, Without<BaseDestroyed>)>,
        Query<(Entity, &Side, &PlayerSlot, &Transform), With<Rock>>,
        Query<(&mut Health, Option<&mut Armor>)>,
        Query<(Entity, &Side, &Transform), (With<Tower>, Without<TowerDying>)>,
    )>,
    // Targeting memory, kept out of the ParamSet (disjoint component) so it can be
    // read/written during the decision loop and again after damage is applied.
    mut targets: Query<&mut CombatTarget>,
) {
    if *state != GameState::Playing {
        // Freeze movement so nothing drifts while paused/ended (physics keeps
        // stepping regardless of GameState).
        for (_, _, _, _, _, mut lin_vel, _, _, _, mut anim, _, _, _, _) in sets.p0().iter_mut() {
            lin_vel.0 = Vec3::ZERO;
            anim.walking = false;
            anim.attacking = false;
        }
        return;
    }

    // 1. Snapshot every combatant's position.
    combatants.clear();
    damage_events.clear();
    attacker_events.clear();
    gold_events.clear();
    heal_events.clear();
    buff_events.clear();
    for (entity, side, slot, kind, transform, _, _, _, _, _, _, _, _, _) in sets.p0().iter() {
        let ckind = match *kind {
            UnitKind::Soldier => CombatantKind::Soldier,
            UnitKind::Miner => CombatantKind::Miner,
            UnitKind::Archer => CombatantKind::Archer,
            UnitKind::Priest => CombatantKind::Priest,
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

    // 2. Per-unit decision. Movement is expressed as LinearVelocity; Avian
    // integrates it and resolves separation/blocking.
    for (
        entity,
        side,
        slot,
        kind,
        mut transform,
        mut lin_vel,
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
            lin_vel.0 = Vec3::ZERO;
            anim.walking = false;
            anim.attacking = false;
            continue;
        }

        let pos = transform.translation;

        match *kind {
            UnitKind::Soldier => {
                let walk_sign = side.forward();
                let mut ct = targets.get_mut(entity).expect("unit has CombatTarget");
                let committed_id = ct.current;
                let last_attacker = ct.last_attacker;

                // Acquire / switch: nearest enemy unit or tower within the short
                // aggro radius wins, re-evaluated every frame, so the soldier
                // redirects to a closer threat that crosses its path.
                let aggro = nearest_enemy_combatant(&combatants, *side, pos, AGGRO_RADIUS);
                // Keep chasing the committed target while it lives and stays
                // within the larger leash, even after it leaves the aggro ring.
                let committed = committed_id.and_then(|e| {
                    combatants.iter().find(|c| {
                        c.entity == e
                            && c.side != *side
                            && (c.kind.is_unit() || c.kind == CombatantKind::Tower)
                            && xz_distance(c.pos, pos) <= TARGET_LEASH
                    })
                });
                // Retaliate: with no target in sight, charge the last attacker if
                // it is alive and within reach (answers ranged fire too).
                let retaliation = || {
                    last_attacker.and_then(|e| {
                        combatants.iter().find(|c| {
                            c.entity == e
                                && c.side != *side
                                && (c.kind.is_unit() || c.kind == CombatantKind::Tower)
                                && xz_distance(c.pos, pos) <= RETALIATE_LEASH
                        })
                    })
                };
                let target = aggro.or(committed).or_else(retaliation);
                ct.current = target.map(|c| c.entity);

                // Fallback objective: the enemy base. Marched toward straight,
                // then attacked on arrival.
                let enemy_base = combatants
                    .iter()
                    .filter(|c| c.kind == CombatantKind::Base && c.side != *side)
                    .map(|c| (c, xz_distance(c.pos, pos)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(c, _)| c);

                // What to engage (entity + pos) and which way to move this frame.
                let (engage, move_dir) = match target {
                    Some(t) => (Some((t.entity, t.pos)), t.pos - pos),
                    None => match enemy_base {
                        Some(base) => (
                            Some((base.entity, base.pos)),
                            march_dir(pos, base.pos, walk_sign),
                        ),
                        None => (None, Vec3::new(walk_sign, 0.0, 0.0)),
                    },
                };

                if let Some((target_entity, target_pos)) = engage
                    && xz_distance(target_pos, pos) <= ENGAGE_RANGE
                {
                    if !anim.attacking {
                        // First frame in melee: restart the swing so it doesn't
                        // pick up at a random fraction from a previous fight.
                        cooldown.0.reset();
                    }
                    cooldown.0.tick(time.delta());
                    anim.attacking = true;
                    if cooldown.0.just_finished() {
                        damage_events.push((target_entity, damage.0, Some(entity)));
                    }
                    lin_vel.0 = Vec3::ZERO;
                    face_dir(&mut transform, target_pos - pos);
                    anim.walking = false;
                    continue;
                }

                anim.attacking = false;
                drive(&mut lin_vel, &mut transform, move_dir, speed.0, true);
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
                            lin_vel.0 = Vec3::ZERO;
                            anim.walking = false;
                            continue;
                        };
                        let target = rock.pos + miner_slot_offset(ring_slot, *side);
                        let target_xz = Vec3::new(target.x, 0.0, target.z);
                        let dist = xz_distance(target_xz, pos);
                        if dist <= MINER_ARRIVE_RANGE {
                            // Arrived at the mining slot: stop, face the rock so
                            // the swing reads toward what's being hit, and mine.
                            lin_vel.0 = Vec3::ZERO;
                            face_dir(&mut transform, rock.pos - pos);
                            if let Some(phase) = phase_opt.as_deref_mut() {
                                *phase = MinerPhase::Mining;
                            }
                            // Restart the swing timer so the first resource is
                            // gained at the END of a full mining cycle, not
                            // instantly on arrival (the cooldown spawns "ready").
                            cooldown.0.reset();
                            anim.walking = false;
                            continue;
                        }
                        drive(&mut lin_vel, &mut transform, target_xz - pos, speed.0, true);
                        anim.walking = true;
                    }
                    MinerPhase::Mining => {
                        lin_vel.0 = Vec3::ZERO;
                        cooldown.0.tick(time.delta());
                        anim.attacking = true;
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
                            lin_vel.0 = Vec3::ZERO;
                            anim.walking = false;
                            continue;
                        };
                        let target_xz = Vec3::new(base.pos.x, 0.0, base.pos.z);
                        let dist = xz_distance(target_xz, pos);
                        if dist <= MINER_DEPOSIT_RANGE {
                            lin_vel.0 = Vec3::ZERO;
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
                        drive(&mut lin_vel, &mut transform, target_xz - pos, speed.0, true);
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
                            && (c.kind.is_unit()
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
                    // straight at the target. animate_unit_model smoothly turns toward
                    // this yaw.
                    anim.face_yaw = face_angle(target.pos.x - pos.x, target.pos.z - pos.z)
                        + ARCHER_SHOT_YAW_OFFSET;
                    anim.attacking = true;
                    // Only queue a shot once the pivot is done (facing error
                    // within ARCHER_TURN_EPS), so the first arrow waits for the
                    // turn to finish. `animate_unit_model` releases the queued shot at
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
                        // Kite: retreat (slower than a normal walk) while shooting.
                        lin_vel.0 = Vec3::new(-walk_sign * speed.0 * 0.7, 0.0, 0.0);
                        anim.walking = true;
                    } else {
                        lin_vel.0 = Vec3::ZERO;
                        anim.walking = false;
                    }
                    continue;
                }

                anim.attacking = false;
                if let Some(arch_state) = arch_state.as_deref_mut() {
                    arch_state.pending_shot = None;
                }
                // No target: face the advance direction so it turns back to the
                // front (animate_unit_model smoothly pivots toward this yaw).
                anim.face_yaw = face_angle(walk_sign, 0.0);

                let enemy_base = combatants
                    .iter()
                    .filter(|c| c.kind == CombatantKind::Base && c.side != *side)
                    .map(|c| (c, xz_distance(c.pos, pos)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(c, _)| c);
                let dir = match enemy_base {
                    Some(base) => march_dir(pos, base.pos, walk_sign),
                    None => Vec3::new(walk_sign, 0.0, 0.0),
                };
                anim.face_yaw = face_angle(dir.x, dir.z);
                // face_yaw owns the archer's rotation (smoothed in animate_unit_model).
                drive(&mut lin_vel, &mut transform, dir, speed.0, false);
                anim.walking = true;
            }
            UnitKind::Priest => {
                let walk_sign = side.forward();
                // Support the nearest ally ahead (toward the front) within range.
                let ally = combatants
                    .iter()
                    .filter(|c| {
                        c.side == *side
                            && c.entity != entity
                            && c.kind.is_unit()
                            && (c.pos.x - pos.x) * walk_sign >= 0.0
                    })
                    .map(|c| (c, xz_distance(c.pos, pos)))
                    .filter(|(_, d)| *d <= PRIEST_RANGE)
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(c, _)| c);

                if let Some(ally) = ally {
                    if !anim.attacking {
                        cooldown.0.reset();
                    }
                    cooldown.0.tick(time.delta());
                    anim.attacking = true;
                    anim.walking = false;
                    lin_vel.0 = Vec3::ZERO;
                    // Face the ally being supported (animate_unit_model smooths to it).
                    anim.face_yaw = face_angle(ally.pos.x - pos.x, ally.pos.z - pos.z);
                    if cooldown.0.just_finished() {
                        heal_events.push((ally.entity, PRIEST_HEAL));
                        buff_events.push((ally.entity, PRIEST_ARMOR, PRIEST_ARMOR_DURATION));
                    }
                    continue;
                }

                anim.attacking = false;
                anim.face_yaw = face_angle(walk_sign, 0.0);

                let enemy_base = combatants
                    .iter()
                    .filter(|c| c.kind == CombatantKind::Base && c.side != *side)
                    .map(|c| (c, xz_distance(c.pos, pos)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(c, _)| c);
                let dir = match enemy_base {
                    Some(base) => march_dir(pos, base.pos, walk_sign),
                    None => Vec3::new(walk_sign, 0.0, 0.0),
                };
                anim.face_yaw = face_angle(dir.x, dir.z);
                // face_yaw owns the priest's rotation (smoothed in animate_unit_model).
                drive(&mut lin_vel, &mut transform, dir, speed.0, false);
                anim.walking = true;
            }
        }
    }

    // 3. Apply damage, heals, armor buffs and gold.
    let mut healths = sets.p3();
    for (target, dmg, source) in damage_events.drain(..) {
        if let Ok((mut hp, armor)) = healths.get_mut(target) {
            let reduction = armor.as_deref().map(Armor::active).unwrap_or(0);
            hp.current -= (dmg - reduction).max(MIN_DAMAGE);
            if let Some(source) = source {
                attacker_events.push((target, source));
            }
        }
    }
    for (target, amount) in heal_events.drain(..) {
        if let Ok((mut hp, _)) = healths.get_mut(target) {
            hp.current = (hp.current + amount).min(hp.max);
        }
    }
    for (target, amount, duration) in buff_events.drain(..) {
        if let Ok((_, Some(mut armor))) = healths.get_mut(target) {
            armor.amount = amount;
            armor.timer = Timer::from_seconds(duration, TimerMode::Once);
        }
    }
    for (slot, amount) in gold_events.drain(..) {
        gold.add(slot, amount);
    }

    // Record who hit whom so idle victims retaliate. CombatTarget is disjoint
    // from the ParamSet, so it's safe to write here, after the damage pass.
    for (victim, attacker) in attacker_events.drain(..) {
        if let Ok(mut ct) = targets.get_mut(victim) {
            ct.last_attacker = Some(attacker);
        }
    }
}

/// Set a unit's planar (XZ) velocity toward `dir` (need not be normalized) at
/// `speed` m/s; Y is left to the locked axis. When `face` is true, also turn the
/// unit to look along the motion (soldier/miner). Aiming kinds (archer/priest)
/// pass `false` and drive their own rotation via `UnitAnim.face_yaw`.
fn drive(
    lin_vel: &mut LinearVelocity,
    transform: &mut Transform,
    dir: Vec3,
    speed: f32,
    face: bool,
) {
    let v = Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero() * speed;
    lin_vel.0 = v;
    if face {
        face_dir(transform, v);
    }
}

/// Turn the unit to face the world XZ direction `dir` (no-op if it's ~zero).
fn face_dir(transform: &mut Transform, dir: Vec3) {
    if dir.x.hypot(dir.z) > 1e-4 {
        let yaw = dir.z.atan2(dir.x);
        transform.rotation = Quat::from_rotation_y(-yaw);
    }
}

/// Nearest enemy unit or tower within `radius` (XZ) of `pos`, if any. Drives the
/// soldier's short-range aggro acquisition / target switching.
fn nearest_enemy_combatant(
    combatants: &[Combatant],
    self_side: Side,
    pos: Vec3,
    radius: f32,
) -> Option<&Combatant> {
    combatants
        .iter()
        .filter(|c| c.side != self_side && (c.kind.is_unit() || c.kind == CombatantKind::Tower))
        .map(|c| (c, xz_distance(c.pos, pos)))
        .filter(|(_, d)| *d <= radius)
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(c, _)| c)
}

/// Direction for a unit marching with no enemy target: dead-straight along its
/// forward axis until within `BASE_SEEK_RANGE` of the enemy base, then steer onto
/// the base so it actually reaches and hits it (instead of converging on the
/// centre line the whole way, which is what made units pile up).
fn march_dir(pos: Vec3, base_pos: Vec3, walk_sign: f32) -> Vec3 {
    if xz_distance(base_pos, pos) <= BASE_SEEK_RANGE {
        base_pos - pos
    } else {
        Vec3::new(walk_sign, 0.0, 0.0)
    }
}

fn xz_distance(a: Vec3, b: Vec3) -> f32 {
    (a.x - b.x).hypot(a.z - b.z)
}

/// Entity yaw (rotation around Y) that faces the world XZ direction `(dx, dz)`,
/// matching `face_dir`'s `Quat::from_rotation_y` convention: `dx>0, dz=0`
/// gives 0 (faces +X).
fn face_angle(dx: f32, dz: f32) -> f32 {
    -dz.atan2(dx)
}

/// Shortest signed angular difference `target - current`, wrapped to
/// `(-PI, PI]`. Shared by `combat_tick` (to know when the archer is aimed) and
/// `animate_unit_model` (to step the pivot).
fn shortest_yaw_diff(target: f32, current: f32) -> f32 {
    let mut diff = (target - current).rem_euclid(std::f32::consts::TAU);
    if diff > PI {
        diff -= std::f32::consts::TAU;
    }
    diff
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

/// Tick down the priest's flat-armor buffs; the reduction drops to 0 the moment
/// a buff's timer expires. Unbuffed units start with an already-finished timer.
pub fn tick_armor_buffs(time: Res<Time>, mut armors: Query<&mut Armor>) {
    for mut armor in &mut armors {
        if !armor.timer.is_finished() {
            armor.timer.tick(time.delta());
            if armor.timer.just_finished() {
                armor.amount = 0;
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
    mut healths: Query<(&mut Health, Option<&Armor>)>,
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
            if let Ok((mut hp, armor)) = healths.get_mut(arrow.target_entity) {
                let reduction = armor.map(Armor::active).unwrap_or(0);
                hp.current -= (arrow.damage - reduction).max(MIN_DAMAGE);
            }
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_dead_units(
    mut commands: Commands,
    models: Res<UnitModels>,
    query: Query<(Entity, &UnitAnim, &UnitKind), With<Unit>>,
) {
    for (entity, anim, kind) in &query {
        // Hold the corpse for the kind's fall-clip duration (miner has no death
        // clip → the generic `DEATH_DURATION` stored on its model) before despawn.
        let duration = models.get(*kind).death_duration;
        if anim.dying && anim.death_t >= duration {
            commands.entity(entity).despawn();
        }
    }
}

/// glTF scenes instance their `AnimationPlayer` asynchronously on a descendant
/// (the Armature). Once the player exists and the owning unit's graph is ready,
/// attach that graph + a transition mixer and record the link back on the unit
/// root so `animate_unit_model` can drive it.
pub fn bind_unit_animation_player(
    mut commands: Commands,
    models: Res<UnitModels>,
    players: Query<Entity, (With<AnimationPlayer>, Without<AnimationGraphHandle>)>,
    parents: Query<&ChildOf>,
    mut units: Query<(&UnitKind, &mut UnitAnimState), With<ModeledUnit>>,
) {
    for player in &players {
        // Walk up the hierarchy to the modeled unit that owns this player.
        let mut current = player;
        let owner = loop {
            if units.contains(current) {
                break Some(current);
            }
            match parents.get(current) {
                Ok(child_of) => current = child_of.parent(),
                Err(_) => break None,
            }
        };
        let Some(owner) = owner else { continue };
        let Ok((kind, mut state)) = units.get_mut(owner) else {
            continue;
        };
        // Graph not built yet → leave the player unbound; retried next frame.
        let Some(graph) = models.get(*kind).graph.clone() else {
            continue;
        };
        commands
            .entity(player)
            .insert((AnimationGraphHandle(graph), AnimationTransitions::new()));
        state.player = Some(player);
    }
}

/// The skeleton's bone entities (with their glTF `Name`s) are instanced
/// asynchronously with the scene. When a unit's weapon hand bone appears, attach
/// that kind's weapon scene and record the bone (the archer reads its world
/// position as the arrow muzzle).
pub fn bind_unit_weapon_hand(
    mut commands: Commands,
    models: Res<UnitModels>,
    bones: Query<(Entity, &Name), Added<Name>>,
    parents: Query<&ChildOf>,
    mut units: Query<(&UnitKind, &mut UnitAnimState), With<ModeledUnit>>,
) {
    for (bone, name) in &bones {
        // Walk up to the owning modeled unit.
        let mut current = bone;
        let owner = loop {
            if units.contains(current) {
                break Some(current);
            }
            match parents.get(current) {
                Ok(child_of) => current = child_of.parent(),
                Err(_) => break None,
            }
        };
        let Some(owner) = owner else { continue };
        let Ok((kind, mut state)) = units.get_mut(owner) else {
            continue;
        };
        let Some(weapon) = models.get(*kind).weapon.as_ref() else {
            continue;
        };
        if name.as_str() != weapon.bone || state.weapon_hand.is_some() {
            continue;
        }
        state.weapon_hand = Some(bone);
        // Placement in the bone frame, then a spin about the weapon's own long
        // axis (right-multiply), same semantics as the bow.
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            weapon.rotation.x,
            weapon.rotation.y,
            weapon.rotation.z,
        ) * Quat::from_rotation_y(weapon.self_flip);
        // Slide the weapon along its (post-rotation) long axis so the hand grips
        // an end rather than the middle. The mesh's local Y is its long axis; one
        // unit there is `scale` bone-local units after scaling.
        let grip_shift = rotation * Vec3::new(0.0, weapon.grip * weapon.scale, 0.0);
        commands.entity(bone).with_child((
            SceneRoot(weapon.scene.clone()),
            Transform::from_translation(weapon.offset + grip_shift)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(weapon.scale)),
            UnitWeapon,
        ));
    }
}

/// True for kinds that turn to face a target via `UnitAnim.face_yaw` (archer
/// aims, priest faces the ally it supports). Soldier/miner keep the rotation set
/// at spawn / in `combat_tick`, so the animator leaves it alone.
fn uses_face_yaw(kind: UnitKind) -> bool {
    matches!(kind, UnitKind::Archer | UnitKind::Priest)
}

/// Drives every modeled unit's `AnimationPlayer` from the `UnitAnim` flags set by
/// `combat_tick` (walk / attack / hurt / death), per kind via `UnitModels`. Owns
/// the `hurt_t`/`death_t` bookkeeping and, for the archer, the per-cycle arrow
/// release. Tolerates kinds with no attack/hurt/death clip (the miner).
pub fn animate_unit_model(
    mut commands: Commands,
    time: Res<Time>,
    models: Res<UnitModels>,
    lib: Res<MatLibrary>,
    mut units: Query<
        (
            &UnitKind,
            &Side,
            &mut UnitAnim,
            &mut UnitAnimState,
            &mut Transform,
        ),
        With<ModeledUnit>,
    >,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    // World transforms of skeleton bones (the hand the arrow leaves from).
    bones: Query<&GlobalTransform>,
) {
    let dt = time.delta_secs();
    let blend = Duration::from_secs_f32(0.15);

    for (kind, side, mut anim, mut state, mut transform) in units.iter_mut() {
        let model = models.get(*kind);
        let Some(nodes) = model.nodes.as_ref() else {
            continue;
        };
        let hurt_count = nodes.hurts.len();
        if anim.hurt_t > 0.0 {
            anim.hurt_t = (anim.hurt_t - dt).max(0.0);
        }
        if anim.dying {
            anim.death_t += dt;
        }

        // Smoothly rotate aiming kinds toward the facing combat_tick asked for
        // (no dedicated turn clip; in-place Mixamo turn clips bake a fixed
        // rotation that fights this). Frozen while dying so the fall clip owns it.
        if !anim.dying && uses_face_yaw(*kind) {
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
            // Kinds with no death clip (miner) just hold; cleanup despawns them.
            if let Some(death) = nodes.death
                && state.current != ModelClip::Death
            {
                transitions
                    .play(&mut player, death, blend)
                    .set_repeat(RepeatAnimation::Never)
                    .set_speed(nodes.death_speed);
                state.current = ModelClip::Death;
                state.oneshot_active = false;
            }
            continue;
        }

        if fresh_hit && !anim.walking && hurt_count > 0 {
            // Reactions hard-cut in (Duration::ZERO) so the flinch replaces the
            // action pose instead of blending over it.
            let node = nodes.hurts[state.hurt_index % hurt_count];
            state.hurt_index = (state.hurt_index + 1) % hurt_count;
            transitions
                .play(&mut player, node, Duration::ZERO)
                .set_repeat(RepeatAnimation::Never);
            state.current = ModelClip::Hurt;
            state.oneshot_active = true;
            continue;
        }

        if state.oneshot_active {
            continue;
        }

        // Hard cut when leaving a reaction; short crossfade otherwise.
        let enter = if snap { Duration::ZERO } else { blend };

        if attacking && let Some(attack_node) = nodes.attack {
            if state.current != ModelClip::Attack {
                // Loop the action clip at a speed that fits one cycle into the
                // cooldown (the clip length sets the cadence).
                transitions
                    .play(&mut player, attack_node, enter)
                    .repeat()
                    .set_speed(nodes.attack_speed);
                state.current = ModelClip::Attack;
                state.last_attack_seek = 0.0;
            }
            // Archer only: release one arrow per cycle the moment playback crosses
            // the release point, from the bow hand's world position. combat_tick
            // leaves `pending_shot` set only while aimed.
            if *kind == UnitKind::Archer {
                let seek = player
                    .animation(attack_node)
                    .map(|a| a.seek_time())
                    .unwrap_or(0.0);
                // Lead is in real seconds; the clip plays at `attack_speed`, so
                // convert into clip-time before subtracting from the release point.
                let release_t = (ARCHER_SHOT_RELEASE_FRACTION * nodes.attack_len
                    - ARCHER_SHOT_RELEASE_LEAD * nodes.attack_speed)
                    .max(0.0);
                let crossed = state.last_attack_seek < release_t && seek >= release_t;
                state.last_attack_seek = seek;
                if crossed && let Some(shot) = state.pending_shot {
                    let start = state
                        .weapon_hand
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
            }
        } else if anim.walking {
            if state.current != ModelClip::Walk {
                transitions
                    .play(&mut player, nodes.walk, enter)
                    .repeat()
                    .set_speed(1.0);
                state.current = ModelClip::Walk;
            }
        } else if state.current != ModelClip::Idle {
            // No dedicated idle clip: hold the walk clip's first frame.
            transitions
                .play(&mut player, nodes.walk, enter)
                .set_repeat(RepeatAnimation::Never)
                .set_speed(0.0)
                .seek_to(0.0);
            state.current = ModelClip::Idle;
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
