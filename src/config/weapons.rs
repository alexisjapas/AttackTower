//! Hand weapon/tool placement: how each weapon scene sits in its skeleton
//! hand bone (scale, Euler placement, self-flip, grip, offset — see
//! `WeaponDef` in common.rs for the semantics). Tuned visually per weapon.

use bevy::prelude::*;

/// Name of the skeleton bone the arrow leaves from — the bow (left) hand. The
/// Meshy rig keeps standard bone names even though it scrambles clip names.
pub const ARCHER_BOW_HAND_BONE: &str = "LeftHand";
/// Fallback arrow origin used only for the frame or two before the `LeftHand`
/// bone is resolved, in the archer entity's local frame (model rigidly parented
/// at `ARCHER_MODEL_YAW_OFFSET`). Negative Z = the model's left side, where the
/// bow hand is; matches `ARCHER_SHOT_YAW_OFFSET`.
pub const ARCHER_HAND_OFFSET: Vec3 = Vec3::new(0.3, 0.95, -0.35);
/// Path of the bow mesh attached to the archer's left hand.
pub const ARCHER_BOW_PATH: &str = "models/adamar/weapons/adamar_bow.glb";
/// Local transform applied to the bow scene once parented to the `LeftHand` bone.
/// The `LeftHand` bone inherits the skeleton root's cm→m scale (`0.01`) times
/// `ARCHER_MODEL_SCALE` (`0.7`), i.e. a world scale of ~0.007. The bow mesh is
/// ~1.9 m tall in its own glTF space, so at scale 1.0 it would render at ~1.3 cm
/// (invisible). This factor cancels that shrink and sizes the bow to the archer.
/// Tune if the bow sits slightly off once the real rig is visible.
pub const ARCHER_BOW_SCALE: f32 = 63.0;
/// Local rotation of the bow within the `LeftHand` bone frame, as Euler XYZ in
/// radians (applied `Quat::from_euler(EulerRot::XYZ, x, y, z)`). The Meshy hand
/// bone's axes don't line up with the bow mesh, so without this the bow lies flat
/// against the forearm instead of standing perpendicular with the limbs vertical.
/// Tune these three angles visually — they are the bow's own orientation in hand.
pub const ARCHER_BOW_ROTATION: Vec3 = Vec3::new(
    0.0,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::FRAC_PI_2,
);
/// Extra 180°-style spin about the bow's *own* long (vertical) axis, applied as a
/// right-multiply after `ARCHER_BOW_ROTATION`. Use this — not the placement Euler
/// — to flip the bow when it reads "à l'envers" (belly/back or tips reversed),
/// since it rotates the mesh on itself rather than around the hand-bone frame.
pub const ARCHER_BOW_SELF_FLIP: f32 = std::f32::consts::PI;
pub const ARCHER_BOW_OFFSET: Vec3 = Vec3::ZERO;
/// The bow is gripped at its middle, so no shift along its long axis.
pub const ARCHER_BOW_GRIP: f32 = 0.0;

// Hand weapons/tools. Each attaches to a skeleton hand bone whose world scale is
// ~0.007 (0.7 model × 0.01 armature), so weapon `SCALE` is large like the bow.
// `ROTATION`/`SELF_FLIP` follow the bow's semantics (Euler in the bone frame,
// then a spin about the weapon's own long axis); all need visual tuning.
// `ROTATION` = `(0, π/2, π/2)` mirrors the (working) bow placement — two 90°
// axes that stand the weapon upright in the hand instead of lying horizontal.
// `SELF_FLIP` spins it about its own long axis (which face/end points out).
// `GRIP` (~0.85) slides the handle into the hand so the blade/head doesn't pass
// through the wrist. All still need visual tuning per weapon.
// `ROTATION.x = π` flips the weapon to point up/forward (it pointed down/back at
// x = 0). `ROTATION.{y,z} = π/2` is the bow-style two-axis upright placement.
// `SELF_FLIP` rolls it about its own long axis. `GRIP`: which mesh end sits in
// the hand (sword's handle is the opposite end from the pickaxe/staff → negative;
// 0 = held at the centre, e.g. the staff).
// `OFFSET` nudges the weapon along the hand bone (≈ +Y → toward the fingers) so
// it sits in the hand instead of anchored at the wrist (bone-local units: the
// bone's world scale is ~0.007, so ~10 ≈ 7 cm).
pub const HAND_OFFSET: Vec3 = Vec3::new(0.0, 10.0, 0.0);

pub const SWORD_PATH: &str = "models/adamar/weapons/adamar_sword.glb";
pub const SWORD_BONE: &str = "RightHand";
pub const SWORD_SCALE: f32 = 63.0;
pub const SWORD_OFFSET: Vec3 = HAND_OFFSET;
// x = 0 (not π): the sword grips the opposite mesh end (negative GRIP) which
// reverses its extension, so it points up/forward like the others without the flip.
pub const SWORD_ROTATION: Vec3 = Vec3::new(
    0.0,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::FRAC_PI_2,
);
pub const SWORD_SELF_FLIP: f32 = std::f32::consts::PI;
pub const SWORD_GRIP: f32 = -0.55;

pub const PICKAXE_PATH: &str = "models/adamar/weapons/adamar_pickaxe.glb";
pub const PICKAXE_BONE: &str = "RightHand";
pub const PICKAXE_SCALE: f32 = 35.3;
pub const PICKAXE_OFFSET: Vec3 = HAND_OFFSET;
pub const PICKAXE_ROTATION: Vec3 = Vec3::new(
    std::f32::consts::PI,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::FRAC_PI_2,
);
pub const PICKAXE_SELF_FLIP: f32 = std::f32::consts::PI;
pub const PICKAXE_GRIP: f32 = 0.65;

pub const STAFF_PATH: &str = "models/adamar/weapons/adamar_staff.glb";
pub const STAFF_BONE: &str = "RightHand";
pub const STAFF_SCALE: f32 = 63.0;
pub const STAFF_OFFSET: Vec3 = HAND_OFFSET;
pub const STAFF_ROTATION: Vec3 = Vec3::new(
    std::f32::consts::PI,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::FRAC_PI_2,
);
pub const STAFF_SELF_FLIP: f32 = std::f32::consts::PI + std::f32::consts::FRAC_PI_2;
pub const STAFF_GRIP: f32 = 0.0;
