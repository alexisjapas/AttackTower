use bevy::prelude::*;

use crate::common::*;

pub fn init_mat_library(
    mut lib: ResMut<MatLibrary>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    lib.left = materials.add(StandardMaterial {
        base_color: Side::Left.color(),
        perceptual_roughness: 0.7,
        ..default()
    });
    lib.right = materials.add(StandardMaterial {
        base_color: Side::Right.color(),
        perceptual_roughness: 0.7,
        ..default()
    });
    lib.left_dark = materials.add(StandardMaterial {
        base_color: Side::Left.color_dark(),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.right_dark = materials.add(StandardMaterial {
        base_color: Side::Right.color_dark(),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.eye_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.04, 0.04, 0.06),
        perceptual_roughness: 0.4,
        ..default()
    });
    lib.ground = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.45, 0.20),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.wood_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.26, 0.13),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.metal_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.74, 0.78),
        metallic: 0.4,
        perceptual_roughness: 0.4,
        ..default()
    });
    lib.stone_light = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.76, 0.70),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.stone_dark = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.52, 0.48),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.43, 0.40),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.grass_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.62, 0.24),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.bush_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.48, 0.20),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.flower_red_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.30, 0.30),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.flower_yellow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.98, 0.85, 0.25),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.flower_violet_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.40, 0.85),
        perceptual_roughness: 0.85,
        ..default()
    });

    lib.body_mesh = meshes.add(Capsule3d::new(0.20, 0.28));
    lib.head_mesh = meshes.add(Sphere::new(0.17));
    lib.limb_mesh = meshes.add(Cylinder::new(0.085, 0.36));
    lib.eye_mesh = meshes.add(Sphere::new(0.035));

    lib.spear_shaft = meshes.add(Cylinder::new(0.025, 0.85));
    lib.spear_tip = meshes.add(Cone::new(0.06, 0.18));
    lib.pickaxe_handle = meshes.add(Cylinder::new(0.025, 0.55));
    lib.pickaxe_head = meshes.add(Cuboid::new(0.34, 0.07, 0.07));
    lib.bow_limb = meshes.add(Cylinder::new(0.035, 0.36));
    lib.bow_string = meshes.add(Cylinder::new(0.010, 0.66));
    lib.arrow_shaft = meshes.add(Cylinder::new(0.014, 0.55));
    lib.arrow_tip = meshes.add(Cone::new(0.040, 0.10));
    lib.arrow_fletch = meshes.add(Cuboid::new(0.01, 0.08, 0.07));

    lib.grass_blade = meshes.add(Cone::new(0.045, 0.22));
    lib.bush_mesh = meshes.add(Sphere::new(0.22));
    lib.plant_stem = meshes.add(Cylinder::new(0.012, 0.28));
    lib.plant_flower = meshes.add(Sphere::new(0.065));

    // Tower (assembled from stacked stone primitives).
    lib.tower_foundation = meshes.add(Cuboid::new(1.05, 0.3, 1.05));
    lib.tower_shaft = meshes.add(Cylinder::new(0.42, 1.6));
    lib.tower_top_slab = meshes.add(Cuboid::new(1.15, 0.16, 1.15));
    lib.tower_crenel = meshes.add(Cuboid::new(0.2, 0.22, 0.2));
    lib.tower_roof = meshes.add(Cone::new(0.55, 0.55));

    // Ghost preview: vertical cylinder shown at the cursor during placement.
    lib.tower_ghost_mesh = meshes.add(Cylinder::new(0.55, TOWER_HEIGHT));
    lib.ghost_valid_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.30, 1.0, 0.45, 0.35),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    lib.ghost_invalid_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.30, 0.30, 0.35),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    // Thin marker strip painted on the ground at each zone boundary.
    lib.zone_marker_mesh = meshes.add(Cuboid::new(0.12, 0.02, 12.0));
    lib.zone_marker_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.95, 0.95, 0.55),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    lib: Res<MatLibrary>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(80.0, 0.2, 40.0))),
        MeshMaterial3d(lib.ground.clone()),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    spawn_sky(&mut commands, &mut meshes, &mut materials);

    spawn_castle(&mut commands, &mut meshes, &lib, Side::Left);
    spawn_castle(&mut commands, &mut meshes, &lib, Side::Right);
    spawn_rock(&mut commands, &mut meshes, &lib, Side::Left);
    spawn_rock(&mut commands, &mut meshes, &lib, Side::Right);

    spawn_zone_markers(&mut commands, &lib);
    spawn_scenery(&mut commands, &lib);
}

fn spawn_zone_markers(commands: &mut Commands, lib: &MatLibrary) {
    for x in [-ZONE_BOUNDARY, ZONE_BOUNDARY] {
        commands.spawn((
            Mesh3d(lib.zone_marker_mesh.clone()),
            MeshMaterial3d(lib.zone_marker_mat.clone()),
            Transform::from_xyz(x, 0.02, 0.0),
        ));
    }
}

fn spawn_sky(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Large inverted sphere acts as a sky dome: cull_mode None so the inside faces
    // render, unlit so it ignores the directional light.
    let sky_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.74, 0.95),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(80.0))),
        MeshMaterial3d(sky_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // A few cloud puffs scattered high overhead.
    let cloud_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.98, 0.98, 0.98),
        unlit: true,
        ..default()
    });
    let cloud_mesh = meshes.add(Sphere::new(1.0));
    for &(x, y, z, sx, sy, sz) in &[
        (-12.0, 14.0, -18.0, 2.4, 1.2, 1.6),
        (8.0, 16.0, -22.0, 3.0, 1.4, 2.0),
        (18.0, 13.0, -12.0, 2.2, 1.0, 1.5),
        (-22.0, 15.0, -8.0, 2.8, 1.3, 1.7),
        (2.0, 18.0, -28.0, 3.4, 1.5, 2.1),
    ] {
        commands.spawn((
            Mesh3d(cloud_mesh.clone()),
            MeshMaterial3d(cloud_mat.clone()),
            Transform {
                translation: Vec3::new(x, y, z),
                scale: Vec3::new(sx, sy, sz),
                ..default()
            },
        ));
    }
}

fn spawn_scenery(commands: &mut Commands, lib: &MatLibrary) {
    // Grass tufts: cone clusters. Placed outside the central walking strip
    // (|z| > 1.4) and away from base/rock footprints.
    let grass_spots: &[(f32, f32)] = &[
        (-13.0, 2.0),
        (-11.5, 3.4),
        (-9.0, -2.6),
        (-6.5, 2.8),
        (-4.0, -3.2),
        (-2.0, 3.1),
        (0.5, -2.4),
        (3.0, 2.7),
        (5.5, -3.5),
        (7.0, 2.2),
        (9.5, -2.5),
        (11.5, 3.0),
        (13.5, -3.2),
        (-15.5, -4.5),
        (-7.5, -4.8),
        (1.5, -5.0),
        (10.0, 4.6),
        (-3.5, 5.4),
        (6.0, 5.2),
        (15.0, 4.0),
    ];
    for &(x, z) in grass_spots {
        spawn_grass_tuft(commands, lib, x, z);
    }

    // Bushes (slightly larger filler).
    let bush_spots: &[(f32, f32)] = &[
        (-14.5, -2.6),
        (-10.0, 4.2),
        (-5.0, -4.5),
        (4.5, 4.4),
        (12.5, -4.5),
        (16.0, 2.5),
        (-16.5, 3.5),
        (8.5, -5.2),
    ];
    for &(x, z) in bush_spots {
        commands.spawn((
            Mesh3d(lib.bush_mesh.clone()),
            MeshMaterial3d(lib.bush_mat.clone()),
            Transform {
                translation: Vec3::new(x, 0.18, z),
                scale: Vec3::new(1.0, 0.85, 1.0),
                ..default()
            },
        ));
    }

    // Flowers: stem + colored top.
    let flower_spots: &[(f32, f32, u8)] = &[
        (-12.5, -3.4, 0),
        (-8.0, 3.6, 1),
        (-2.5, -2.2, 2),
        (2.5, 3.6, 0),
        (5.0, -2.2, 1),
        (11.0, 2.5, 2),
        (14.0, -2.0, 0),
        (-5.5, 3.8, 2),
        (-15.0, 2.4, 1),
        (15.5, 3.2, 0),
    ];
    for &(x, z, color_idx) in flower_spots {
        let petal_mat = match color_idx {
            0 => lib.flower_red_mat.clone(),
            1 => lib.flower_yellow_mat.clone(),
            _ => lib.flower_violet_mat.clone(),
        };
        commands
            .spawn((Transform::from_xyz(x, 0.0, z), Visibility::default()))
            .with_children(|f| {
                f.spawn((
                    Mesh3d(lib.plant_stem.clone()),
                    MeshMaterial3d(lib.bush_mat.clone()),
                    Transform::from_xyz(0.0, 0.14, 0.0),
                ));
                f.spawn((
                    Mesh3d(lib.plant_flower.clone()),
                    MeshMaterial3d(petal_mat),
                    Transform::from_xyz(0.0, 0.30, 0.0),
                ));
            });
    }
}

fn spawn_grass_tuft(commands: &mut Commands, lib: &MatLibrary, x: f32, z: f32) {
    // Three small cones leaning slightly outward form a grass tuft.
    let blades = [
        (0.0, 0.0, 0.0_f32),
        (0.07, 0.04, 0.15),
        (-0.06, -0.05, -0.18),
        (0.04, -0.07, 0.10),
    ];
    commands
        .spawn((Transform::from_xyz(x, 0.0, z), Visibility::default()))
        .with_children(|t| {
            for (i, &(dx, dz, tilt)) in blades.iter().enumerate() {
                let height_scale = 0.75 + 0.25 * ((i as f32) * 0.7).sin().abs();
                t.spawn((
                    Mesh3d(lib.grass_blade.clone()),
                    MeshMaterial3d(lib.grass_mat.clone()),
                    Transform {
                        translation: Vec3::new(dx, 0.10, dz),
                        rotation: Quat::from_rotation_z(tilt),
                        scale: Vec3::new(1.0, height_scale, 1.0),
                    },
                ));
            }
        });
}

fn spawn_castle(commands: &mut Commands, meshes: &mut Assets<Mesh>, lib: &MatLibrary, side: Side) {
    let x = match side {
        Side::Left => LEFT_BASE_X,
        Side::Right => RIGHT_BASE_X,
    };
    let main = match side {
        Side::Left => lib.left.clone(),
        Side::Right => lib.right.clone(),
    };

    let foundation_mesh = meshes.add(Cuboid::new(2.0, 0.4, 2.0));
    let keep_mesh = meshes.add(Cuboid::new(1.1, 1.2, 1.1));
    let top_slab_mesh = meshes.add(Cuboid::new(1.3, 0.12, 1.3));
    let crenel_mesh = meshes.add(Cuboid::new(0.22, 0.22, 0.22));
    let tower_mesh = meshes.add(Cuboid::new(0.45, 1.6, 0.45));
    let roof_mesh = meshes.add(Cone::new(0.36, 0.55));
    let door_mesh = meshes.add(Cuboid::new(0.08, 0.55, 0.36));
    let pole_mesh = meshes.add(Cylinder::new(0.03, 0.9));
    let flag_mesh = meshes.add(Cuboid::new(0.34, 0.22, 0.02));

    commands
        .spawn((
            Transform {
                translation: Vec3::new(x, 0.0, 0.0),
                rotation: side.base_rotation(),
                scale: Vec3::ONE,
            },
            Visibility::default(),
            Base,
            side,
            Health::new(BASE_HP),
        ))
        .with_children(|p| {
            // Foundation
            p.spawn((
                Mesh3d(foundation_mesh),
                MeshMaterial3d(lib.stone_dark.clone()),
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
            // Central keep
            p.spawn((
                Mesh3d(keep_mesh),
                MeshMaterial3d(lib.stone_light.clone()),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ));
            // Battlement slab
            p.spawn((
                Mesh3d(top_slab_mesh),
                MeshMaterial3d(lib.stone_dark.clone()),
                Transform::from_xyz(0.0, 1.66, 0.0),
            ));
            // Crenellations around the slab edge
            let crenel_y = 1.83;
            for &(cx, cz) in &[
                (0.55, 0.0),
                (-0.55, 0.0),
                (0.0, 0.55),
                (0.0, -0.55),
                (0.40, 0.40),
                (-0.40, 0.40),
                (0.40, -0.40),
                (-0.40, -0.40),
            ] {
                p.spawn((
                    Mesh3d(crenel_mesh.clone()),
                    MeshMaterial3d(lib.stone_light.clone()),
                    Transform::from_xyz(cx, crenel_y, cz),
                ));
            }
            // Four corner towers with cone roofs (roofs use side color).
            for &(tx, tz) in &[(0.78, 0.78), (-0.78, 0.78), (0.78, -0.78), (-0.78, -0.78)] {
                p.spawn((
                    Mesh3d(tower_mesh.clone()),
                    MeshMaterial3d(lib.stone_light.clone()),
                    Transform::from_xyz(tx, 1.2, tz),
                ));
                p.spawn((
                    Mesh3d(roof_mesh.clone()),
                    MeshMaterial3d(main.clone()),
                    Transform::from_xyz(tx, 2.28, tz),
                ));
            }
            // Door at the back (toward this side's miners).
            p.spawn((
                Mesh3d(door_mesh),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform::from_xyz(-1.0 + 0.04, 0.67, 0.0),
            ));
            // Flag pole + flag on top of the keep.
            p.spawn((
                Mesh3d(pole_mesh),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform::from_xyz(0.0, 2.3, 0.0),
            ));
            p.spawn((
                Mesh3d(flag_mesh),
                MeshMaterial3d(main.clone()),
                Transform::from_xyz(0.18, 2.65, 0.0),
            ));
        });
}

fn spawn_rock(commands: &mut Commands, meshes: &mut Assets<Mesh>, lib: &MatLibrary, side: Side) {
    let base_x = match side {
        Side::Left => LEFT_BASE_X,
        Side::Right => RIGHT_BASE_X,
    };
    // Rocks are placed behind each base (opposite of unit forward).
    let x = base_x - side.forward() * ROCK_OFFSET;

    commands
        .spawn((
            Transform::from_xyz(x, 0.0, 0.0),
            Visibility::default(),
            Rock,
            side,
        ))
        .with_children(|p| {
            p.spawn((
                Mesh3d(meshes.add(Sphere::new(0.65))),
                MeshMaterial3d(lib.rock_mat.clone()),
                Transform::from_xyz(0.0, 0.45, 0.0),
            ));
            p.spawn((
                Mesh3d(meshes.add(Sphere::new(0.42))),
                MeshMaterial3d(lib.rock_mat.clone()),
                Transform {
                    translation: Vec3::new(0.32, 0.28, 0.30),
                    rotation: Quat::from_rotation_y(0.6),
                    scale: Vec3::ONE,
                },
            ));
            p.spawn((
                Mesh3d(meshes.add(Sphere::new(0.36))),
                MeshMaterial3d(lib.rock_mat.clone()),
                Transform::from_xyz(-0.38, 0.22, -0.28),
            ));
        });
}
