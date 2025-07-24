use core::f32::consts::PI;

use bevy::{
    math::ops::{cos, sin},
    prelude::*,
};
use bevy_rapier3d::{
    plugin::ReadRapierContext,
    prelude::{ExternalForce, QueryFilter, RigidBody, Velocity},
};
use leafwing_input_manager::{Actionlike, prelude::ActionState};

use crate::{IntendedRotation, VecTools};
use crate::{CameraDistance, MoveVector, Player};
use crate::{Ground, MoveSpeed};

const CAMERA_ANGLE: f32 = 30_f32.to_radians();

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MoveVector>();
        app.add_systems(
            Update,
            (
                control_player,
                camera_lock.after(control_player),
                entities_try_to_move.after(control_player),
                stay_grounded,
                fix_rotation,
            ),
        );
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub(crate) enum Action {
    Left,
    Right,
    Up,
    Down,
}

pub fn control_player(
    mut query: Query<(&mut MoveVector, &MoveSpeed, &ActionState<Action>), With<Player>>,
    cam: Query<&Transform, With<Camera3d>>,
    player: Query<&Transform, With<Player>>,
) {
    let (mut move_vec, move_speed, action_state) = query.single_mut().unwrap();
    **move_vec = Vec3::ZERO;
    let cam = cam.single().unwrap();
    let player = player.single().unwrap();

    let forward = Vec3::new(
        player.translation.x - cam.translation.x,
        0.0,
        player.translation.z - cam.translation.z,
    )
    .normalize();
    let right = forward.cross(Vec3::Y);

    // handle pressing buttons
    if action_state.pressed(&Action::Left) {
        **move_vec -= right;
    }

    if action_state.pressed(&Action::Right) {
        **move_vec += right;
    }

    if action_state.pressed(&Action::Down) {
        **move_vec -= forward;
    }

    if action_state.pressed(&Action::Up) {
        **move_vec += forward;
    }

    // handle releasing the buttons
    // can't just set move_vec to 0 at start of function call
    // because we have to "keep track" of how much velocity
    // moving is adding to the overall velocity so we have
    // snappy movement AND physics based movement
    if action_state.released(&Action::Left) {
        **move_vec += right;
    }

    if action_state.released(&Action::Right) {
        **move_vec -= right;
    }

    if action_state.released(&Action::Down) {
        **move_vec += forward;
    }

    if action_state.released(&Action::Up) {
        **move_vec -= forward;
    }

    **move_vec = move_vec.normalize_or(Vec3::ZERO) * **move_speed;
    // dbg!(move_vec);
}

pub fn stay_grounded(
    mut query: Query<(Entity, &mut Transform), (With<RigidBody>, Without<Ground>)>,
    grounds: Query<(Entity, &mut Transform), With<Ground>>,
    rapier_context: ReadRapierContext,
) {
    for (this, mut t) in query.iter_mut() {
        let ray_pos = t.translation - Vec3::new(0.0, 2.0, 0.0);
        let ray_dir = -Vec3::Y;
        let solid = true;
        let filter = QueryFilter {
            exclude_collider: Some(this),
            ..default()
        };

        rapier_context.single().unwrap().intersections_with_ray(
            ray_pos,
            ray_dir,
            bevy_rapier3d::math::Real::MAX,
            solid,
            filter,
            |entity, intersection| {
                for (ge, gt) in grounds.iter() {
                    if ge == entity {
                        t.translation.y = intersection.point.y + 2.0;
                        return true;
                    }
                }
                return false;
            }
        );
    }
}

pub fn fix_rotation(mut query: Query<(&mut Transform, &IntendedRotation)>) {
    for (mut t, r) in query.iter_mut() {
        t.rotation = **r;
    }
}

pub fn entities_try_to_move(mut query: Query<(&mut ExternalForce, &Velocity, &MoveVector)>) {
    for (mut force, vel, move_vec) in query.iter_mut() {
        // velocity.linvel.max_mag(move_vec);
        let new_force = calc_force_diff(1.0, vel.linvel.xz(), move_vec.xz());
        force.force = Vec3::new(new_force.x, force.force.y, new_force.y);
    }
}

/// clamped_input is a 0.0-1.0 value representing the user's
/// desired percentage of top speed to hold
///
/// `current_velocity` is the current horizontal velocity
fn calc_force_diff(clamped_input: f32, current_velocity: Vec2, target_velocity: Vec2) -> Vec2 {
    let target_speed = target_velocity * clamped_input;
    let diff_to_make_up = target_speed - current_velocity;
    diff_to_make_up * 300.0
}

pub fn camera_lock(
    mut cam: Query<(&mut Transform, &CameraDistance), (With<Camera3d>, Without<Player>)>,
    player: Query<&Transform, With<Player>>,
) {
    let ((mut cam, dist), player) = (cam.single_mut().unwrap(), player.single().unwrap());

    let x = **dist * sin(CAMERA_ANGLE);
    let y = **dist * cos(CAMERA_ANGLE);

    cam.translation = player.translation + Vec3::new(x, y, x);
    *cam = cam.looking_at(player.translation, Vec3::Y);
}
