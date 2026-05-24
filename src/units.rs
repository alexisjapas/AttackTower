use std::f32::consts::{FRAC_PI_2, PI};

use bevy::ecs::system::ParamSet;
use bevy::prelude::*;

use crate::common::*;

pub fn spawn_soldier(commands: &mut Commands, lib: &MatLibrary, side: Side) {
    spawn_unit(commands, lib, side, UnitKind::Soldier);
}

pub fn spawn_miner(commands: &mut Commands, lib: &MatLibrary, side: Side) {
    spawn_unit(commands, lib, side, UnitKind::Miner);
}

pub fn spawn_archer(commands: &mut Commands, lib: &MatLibrary, side: Side) {
    spawn_unit(commands, lib, side, UnitKind::Archer);
}

fn spawn_unit(commands: &mut Commands, lib: &MatLibrary, side: Side, kind: UnitKind) {
    let base_x = match side {
        Side::Left => LEFT_BASE_X,
        Side::Right => RIGHT_BASE_X,
    };
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
    let z = (rand_jitter() - 0.5) * SPAWN_Z_JITTER * 2.0;
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

    // Weapon attachment. The archer's bow is parented to the left arm; the
    // left arm's rotation is overridden in animate_units to keep it raised
    // forward (holding the bow at chest height).
    match kind {
        UnitKind::Soldier => attach_spear(commands, arm_right, lib),
        UnitKind::Miner => attach_pickaxe(commands, arm_right, lib),
        UnitKind::Archer => attach_bow_and_arrow(commands, arm_left, arm_right, lib),
    }

    commands
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
        .add_children(&[bob, leg_left, leg_right]);
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

fn attach_bow_and_arrow(
    commands: &mut Commands,
    arm_left: Entity,
    arm_right: Entity,
    lib: &MatLibrary,
) {
    // Bow: attached to the left arm at the hand. The left arm is held raised
    // forward (animate_units overrides it to +π/2 around Z for archers), which
    // brings the hand up to chest height in front of the body. The bow has a
    // -π/2 rotation that cancels the arm rotation, so its internal D-shape
    // stays in the world XY plane (visible to the 3/4 camera) regardless.
    let bow = commands
        .spawn((
            Transform {
                translation: Vec3::new(0.0, -0.36, 0.0),
                rotation: Quat::from_rotation_z(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
            Visibility::default(),
        ))
        .with_children(|b| {
            let arc: [((f32, f32), (f32, f32)); 4] = [
                ((0.0, 0.30), (0.09, 0.18)),
                ((0.09, 0.18), (0.13, 0.0)),
                ((0.13, 0.0), (0.09, -0.18)),
                ((0.09, -0.18), (0.0, -0.30)),
            ];
            for (start, end) in arc {
                let a = Vec3::new(start.0, start.1, 0.0);
                let c = Vec3::new(end.0, end.1, 0.0);
                let mid = (a + c) * 0.5;
                let dir = c - a;
                let length = dir.length();
                let rotation = Quat::from_rotation_arc(Vec3::Y, dir.normalize());
                b.spawn((
                    Mesh3d(lib.bow_limb.clone()),
                    MeshMaterial3d(lib.wood_mat.clone()),
                    Transform {
                        translation: mid,
                        rotation,
                        scale: Vec3::new(1.0, length / 0.36, 1.0),
                    },
                ));
            }
            // Bowstring: straight vertical chord on the back side of the arc.
            b.spawn((
                Mesh3d(lib.bow_string.clone()),
                MeshMaterial3d(lib.eye_mat.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        })
        .id();
    commands.entity(arm_left).add_children(&[bow]);

    // Nocked arrow: held in the right hand, shaft along +X (forward). The shaft
    // and tip cylinders/cone are Y-aligned by default; rotating -90° around Z
    // aligns them with +X.
    let arrow = commands
        .spawn((
            Transform {
                translation: Vec3::new(0.0, -0.36, 0.0),
                rotation: Quat::from_rotation_z(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
            Visibility::default(),
        ))
        .with_children(|a| {
            a.spawn((
                Mesh3d(lib.arrow_shaft.clone()),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform::default(),
            ));
            a.spawn((
                Mesh3d(lib.arrow_tip.clone()),
                MeshMaterial3d(lib.metal_mat.clone()),
                Transform::from_xyz(0.0, 0.32, 0.0),
            ));
            a.spawn((
                Mesh3d(lib.arrow_fletch.clone()),
                MeshMaterial3d(lib.eye_mat.clone()),
                Transform::from_xyz(0.0, -0.25, 0.0),
            ));
        })
        .id();
    commands.entity(arm_right).add_children(&[arrow]);
}

fn rand_jitter() -> f32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEED: AtomicU32 = AtomicU32::new(0x1234_5678);
    let mut x = SEED.load(Ordering::Relaxed).wrapping_add(0x9E37_79B9);
    SEED.store(x, Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    (x & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
}

#[derive(Clone, Copy)]
struct Combatant {
    entity: Entity,
    side: Side,
    pos: Vec3,
    kind: CombatantKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CombatantKind {
    Soldier,
    Miner,
    Archer,
    Base,
    Rock,
    Tower,
}

pub fn combat_tick(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<GameState>,
    lib: Res<MatLibrary>,
    mut gold: ResMut<Gold>,
    mut sets: ParamSet<(
        Query<
            (
                Entity,
                &Side,
                &UnitKind,
                &mut Transform,
                &Damage,
                &mut AttackCooldown,
                &MoveSpeed,
                &mut UnitAnim,
            ),
            With<Unit>,
        >,
        Query<(Entity, &Side, &Transform), With<Base>>,
        Query<(Entity, &Side, &Transform), With<Rock>>,
        Query<&mut Health>,
        Query<(Entity, &Side, &Transform), With<Tower>>,
    )>,
) {
    if *state != GameState::Playing {
        for (_, _, _, _, _, _, _, mut anim) in sets.p0().iter_mut() {
            anim.walking = false;
            anim.attacking = false;
        }
        return;
    }

    // 1. Snapshot every combatant's position.
    let mut combatants: Vec<Combatant> = Vec::new();
    for (entity, side, kind, transform, _, _, _, _) in sets.p0().iter() {
        let ckind = match *kind {
            UnitKind::Soldier => CombatantKind::Soldier,
            UnitKind::Miner => CombatantKind::Miner,
            UnitKind::Archer => CombatantKind::Archer,
        };
        combatants.push(Combatant {
            entity,
            side: *side,
            pos: transform.translation,
            kind: ckind,
        });
    }
    for (entity, side, transform) in sets.p1().iter() {
        combatants.push(Combatant {
            entity,
            side: *side,
            pos: transform.translation,
            kind: CombatantKind::Base,
        });
    }
    for (entity, side, transform) in sets.p2().iter() {
        combatants.push(Combatant {
            entity,
            side: *side,
            pos: transform.translation,
            kind: CombatantKind::Rock,
        });
    }
    for (entity, side, transform) in sets.p4().iter() {
        combatants.push(Combatant {
            entity,
            side: *side,
            pos: transform.translation,
            kind: CombatantKind::Tower,
        });
    }

    let dt = time.delta_secs();
    let mut damage_events: Vec<(Entity, i32)> = Vec::new();
    let mut gold_events: Vec<(Side, u32)> = Vec::new();

    // 2. Per-unit decision.
    for (entity, side, kind, mut transform, damage, mut cooldown, speed, mut anim) in
        sets.p0().iter_mut()
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

                if let Some((target, dist)) = enemy {
                    if dist <= ENGAGE_RANGE {
                        cooldown.0.tick(time.delta());
                        anim.attacking = true;
                        anim.attack_phase = cooldown.0.fraction();
                        if cooldown.0.just_finished() {
                            damage_events.push((target.entity, damage.0));
                        }
                        anim.walking = false;
                        continue;
                    }
                }

                anim.attacking = false;

                if ally_blocking(
                    &combatants,
                    entity,
                    *side,
                    pos,
                    walk_sign,
                    CombatantKind::Soldier,
                ) {
                    anim.walking = false;
                    continue;
                }
                transform.translation.x += walk_sign * speed.0 * dt;
                anim.walking = true;
            }
            UnitKind::Miner => {
                let walk_sign = -side.forward();
                let own_rock = combatants
                    .iter()
                    .find(|c| c.side == *side && c.kind == CombatantKind::Rock);

                if let Some(rock) = own_rock {
                    let dist = xz_distance(rock.pos, pos);
                    if dist <= MINE_RANGE {
                        cooldown.0.tick(time.delta());
                        anim.attacking = true;
                        anim.attack_phase = cooldown.0.fraction();
                        if cooldown.0.just_finished() {
                            gold_events.push((*side, MINER_GOLD_PER_HIT));
                        }
                        anim.walking = false;
                        continue;
                    }
                }

                anim.attacking = false;

                if ally_blocking(
                    &combatants,
                    entity,
                    *side,
                    pos,
                    walk_sign,
                    CombatantKind::Miner,
                ) {
                    anim.walking = false;
                    continue;
                }
                transform.translation.x += walk_sign * speed.0 * dt;
                anim.walking = true;
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

                if let Some((target, dist)) = enemy {
                    if dist <= ARCHER_RANGE {
                        cooldown.0.tick(time.delta());
                        anim.attacking = true;
                        anim.attack_phase = cooldown.0.fraction();
                        if cooldown.0.just_finished() {
                            // Arrow leaves the bow (hand-raised left arm), which sits
                            // ~0.36 in front of the body at chest height (0.65), offset
                            // toward the left-arm side (Z = +ARM_SPREAD_Z) before the
                            // unit's facing rotation. side.forward() flips X and Z for
                            // the Right side (rotated by Y(PI)).
                            let start = pos
                                + Vec3::new(
                                    side.forward() * 0.36,
                                    0.65,
                                    side.forward() * ARM_SPREAD_Z,
                                );
                            spawn_arrow(
                                &mut commands,
                                &lib,
                                *side,
                                start,
                                target.entity,
                                target.pos,
                                damage.0,
                            );
                        }
                        anim.walking = false;
                        continue;
                    }
                }

                anim.attacking = false;

                if ally_blocking(
                    &combatants,
                    entity,
                    *side,
                    pos,
                    walk_sign,
                    CombatantKind::Archer,
                ) {
                    anim.walking = false;
                    continue;
                }
                transform.translation.x += walk_sign * speed.0 * dt;
                anim.walking = true;
            }
        }
    }

    // 3. Apply damage and gold.
    let mut healths = sets.p3();
    for (target, dmg) in damage_events {
        if let Ok(mut hp) = healths.get_mut(target) {
            hp.current -= dmg;
        }
    }
    for (side, amount) in gold_events {
        gold.add(side, amount);
    }
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

fn xz_distance(a: Vec3, b: Vec3) -> f32 {
    (a.x - b.x).hypot(a.z - b.z)
}

pub fn process_damage_effects(
    mut query: Query<(&Health, &mut UnitAnim), (With<Unit>, Changed<Health>)>,
) {
    for (hp, mut anim) in query.iter_mut() {
        if hp.current <= 0 {
            if !anim.dying {
                anim.dying = true;
                anim.death_t = 0.0;
            }
        } else if hp.current < hp.max {
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
            apply_pose(&mut transforms, rig, 0.0, 0.0, 0.0, 0.0, 0.0);
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
            t.rotation = if matches!(*kind, UnitKind::Archer) {
                // Archer holds the bow steady: left arm raised forward at chest height.
                Quat::from_rotation_z(FRAC_PI_2)
            } else {
                Quat::from_rotation_z(-arm_angle)
            };
        }
        if let Ok(mut t) = transforms.get_mut(rig.arm_right) {
            t.rotation = Quat::from_rotation_z(right_arm_angle);
        }
    }
}

fn apply_pose(
    transforms: &mut Query<&mut Transform, Without<Unit>>,
    rig: &UnitRig,
    bob_y: f32,
    leg_angle: f32,
    arm_angle: f32,
    right_arm_angle: f32,
    hurt_tilt: f32,
) {
    if let Ok(mut t) = transforms.get_mut(rig.bob) {
        t.translation.y = BOB_BASE_Y + bob_y;
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

pub fn cleanup_dead_units(mut commands: Commands, query: Query<(Entity, &UnitAnim), With<Unit>>) {
    for (entity, anim) in &query {
        if anim.dying && anim.death_t >= DEATH_DURATION {
            commands.entity(entity).despawn();
        }
    }
}
