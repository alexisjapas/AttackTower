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

    lib.body_mesh = meshes.add(Capsule3d::new(0.20, 0.28));
    lib.head_mesh = meshes.add(Sphere::new(0.17));
    lib.limb_mesh = meshes.add(Cylinder::new(0.085, 0.36));
    lib.eye_mesh = meshes.add(Sphere::new(0.035));

    lib.spear_shaft = meshes.add(Cylinder::new(0.025, 0.85));
    lib.spear_tip = meshes.add(Cone::new(0.06, 0.18));
    lib.pickaxe_handle = meshes.add(Cylinder::new(0.025, 0.55));
    lib.pickaxe_head = meshes.add(Cuboid::new(0.34, 0.07, 0.07));
}

pub fn setup_world(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, lib: Res<MatLibrary>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 11.0, 13.5).looking_at(Vec3::ZERO, Vec3::Y),
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
        Mesh3d(meshes.add(Cuboid::new(34.0, 0.2, 10.0))),
        MeshMaterial3d(lib.ground.clone()),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    spawn_castle(&mut commands, &mut meshes, &lib, Side::Left);
    spawn_castle(&mut commands, &mut meshes, &lib, Side::Right);
    spawn_rock(&mut commands, &mut meshes, &lib, Side::Left);
    spawn_rock(&mut commands, &mut meshes, &lib, Side::Right);
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
