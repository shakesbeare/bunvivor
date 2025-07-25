use core::f32::consts::PI;

use bevy::{
    math::ops::{cos, sin},
    prelude::*,
};
use bevy_rapier3d::{
    pipeline::CollisionEvent,
    plugin::ReadRapierContext,
    prelude::{
        Collider, ExternalForce, GravityScale, QueryFilter, RigidBody, ShapeCastOptions, Velocity,
    },
};
use leafwing_input_manager::{Actionlike, prelude::ActionState};

use crate::{CameraDistance, CollidedGrounds, MoveVector, Player};
use crate::{Ground, MoveSpeed};
use crate::{IntendedRotation, VecTools};

const CAMERA_ANGLE: f32 = 30_f32.to_radians();

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MoveVector>();
        app.register_type::<CollidedGrounds>();
        app.add_systems(
            Update,
            (
                control_player,
                camera_lock.after(control_player),
                entities_try_to_move.after(control_player),
                gravity_control,
                check_collided_grounds,
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

pub fn check_collided_grounds(
    ground: Query<Entity, With<Ground>>,
    mut collidee: Query<(Entity, &mut CollidedGrounds), Without<Ground>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity, entity1, collision_event_flags) => {
                let mut ent = collidee.iter_mut().find(|(e, _)| e == entity);
                let other = ground.iter().find(|e| e == entity1);
                if let (Some((_, mut cg)), Some(other)) = (ent, other) {
                    cg.push(other);
                }
            }
            CollisionEvent::Stopped(entity, entity1, collision_event_flags) => {
                let mut ent = collidee.iter_mut().find(|(e, _)| e == entity);
                let mut other = ground.iter().find(|e| e == entity1);
                if let (Some((this, mut cg)), Some(other)) = (ent, other) {
                    let idx = cg.iter().position(|e| *e == other);
                    if let Some(idx) = idx {
                        cg.swap_remove(idx);
                    }
                }
            }
        }
    }
}

pub fn gravity_control(mut query: Query<(&mut GravityScale, &CollidedGrounds)>) {
    for (mut gs, cg) in query.iter_mut() {
        if cg.is_empty() {
            // object is not grounded, no collided grounds exist
            gs.0 = 30.0;
        } else {
            // object is grounded, collided grounds exist
            gs.0 = 0.0;
        }
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
