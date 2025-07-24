use core::f32::consts::PI;

use bevy::{
    math::ops::{cos, sin},
    prelude::*,
};
use leafwing_input_manager::{Actionlike, prelude::ActionState};

use crate::{move_entities, CameraDistance, MoveVector, Player, Velocity};

const CAMERA_ANGLE: f32 = 30_f32.to_radians();

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                control_player,
                camera_lock.after(control_player).after(move_entities),
                entities_try_to_move.after(control_player),
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
    mut query: Query<(&mut MoveVector, &ActionState<Action>), With<Player>>,
    cam: Query<&Transform, With<Camera3d>>,
    player: Query<&Transform, With<Player>>,
) {
    let (mut move_vec, action_state) = query.single_mut().unwrap();
    **move_vec = Vec3::new(0., 0., 0.);
    let cam = cam.single().unwrap();
    let player = player.single().unwrap();

    let forward = Vec3::new(
        player.translation.x - cam.translation.x,
        0.0,
        player.translation.z - cam.translation.z,
    ).normalize();
    let right = forward.cross(Vec3::Y);

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

    move_vec.normalize();
}

pub fn entities_try_to_move(mut query: Query<(&mut Velocity, &crate::MoveSpeed, &MoveVector)>) {
    for (mut velocity, move_speed, move_vec) in query.iter_mut() {
        **velocity = **move_vec * move_speed.0;
    }
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
