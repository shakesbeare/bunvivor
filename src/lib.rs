#![allow(clippy::type_complexity)]
#![allow(unused)]

use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css::{SILVER, WHITE};
use bevy::prelude::App as BevyApp;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::inspector_egui_impls::InspectorPrimitive;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_math::ops::{cos, sin};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use rand::prelude::*;

use crate::animation::SpriteScale;
use crate::controls::Action;

mod animation;
mod controls;
#[cfg(debug_assertions)]
mod inspector;

pub struct App {
    _app: BevyApp,
}

impl App {
    pub fn new() -> Self {
        let mut app = BevyApp::new();

        app.add_plugins(SetupPlugin);
        app.add_plugins(InputManagerPlugin::<crate::controls::Action>::default());
        #[cfg(debug_assertions)]
        app.add_plugins(crate::inspector::Inspector);
        app.add_plugins(crate::controls::ControlsPlugin);
        app.add_plugins(crate::animation::AnimationPlugin);
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        app.add_plugins(RapierDebugRenderPlugin::default());

        return Self { _app: app };
    }

    pub fn run(&mut self) {
        self._app.run();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub trait VecTools {
    type Other;
    fn max_mag(&mut self, other: &Self::Other);
}

impl VecTools for Vec3 {
    type Other = Vec3;

    fn max_mag(&mut self, other: &Self::Other) {
        if self.x.abs() < other.x.abs() {
            self.x = other.x;
        }

        if self.y.abs() < other.y.abs() {
            self.y = other.y;
        }

        if self.z.abs() < other.z.abs() {
            self.z = other.z;
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Ground;

/// Describes the move speed of the player in terms of background tiles per second
#[derive(Component, Deref, DerefMut)]
pub struct MoveSpeed(pub f32);

#[derive(Component, Deref, DerefMut, Reflect, Debug, PartialEq, Default)]
pub struct IntendedRotation(pub Quat);

/// Describes the direction an entity is trying to move
#[derive(Debug, Component, Deref, DerefMut, Reflect)]
pub struct MoveVector {
    pub vec: Vec3,
}

impl Default for MoveVector {
    fn default() -> Self {
        Self { vec: Vec3::ZERO }
    }
}

impl AsRef<Vec3> for MoveVector {
    fn as_ref(&self) -> &Vec3 {
        return &self.vec;
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct CameraDistance(pub f32);

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut BevyApp) {
        app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // bun
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d {
            radius: 1.0,
            half_length: 1.0,
        })),
        MeshMaterial3d(materials.add(Color::from(WHITE))),
        InputMap::new([
            (Action::Left, KeyCode::ArrowLeft),
            (Action::Right, KeyCode::ArrowRight),
            (Action::Up, KeyCode::ArrowUp),
            (Action::Down, KeyCode::ArrowDown),
        ]),
        MoveSpeed(23.6),
        MoveVector::default(),
        Player,
        Transform::from_translation(Vec3::new(0.0, 2.1, 0.0)),
        Name::new("Player"),
        bevy_rapier3d::dynamics::Damping {
            linear_damping: 0.0,
            angular_damping: 6.5
        },
        RigidBody::Dynamic,
        Velocity::default(),
        ExternalForce::default(),
        GravityScale(0.0),
        Collider::capsule(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 0.0), 1.0),
        IntendedRotation::default(),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 80.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(16.0, 16.0, 16.0),
        Name::new("Sun"),
    ));

    // base floor
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(112.0, 112.0)
                    .subdivisions(10),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            ..default()
        })),
        Collider::cuboid(66.0, 0.1, 66.0),
        Friction {
            coefficient: 0.0,
            ..default()
        },
        RigidBody::Fixed,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Name::new("Debug Floor"),
        Ground,
    ));

    // ramp
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(112.0, 112.0)
                    .subdivisions(10),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            ..default()
        })),
        Collider::cuboid(66.0, 0.1, 66.0),
        Friction {
            coefficient: 0.0,
            ..default()
        },
        RigidBody::Fixed,
        Transform::from_translation(Vec3::new(-120.0, 66.0 * sin(30_f32.to_radians()), 0.0))
            .with_rotation(Quat::from_rotation_z(-30_f32.to_radians())),
        Name::new("Debug Floor"),
        Ground,
    ));

    // second floor
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(112.0, 112.0)
                    .subdivisions(10),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            ..default()
        })),
        Collider::cuboid(66.0, 0.1, 66.0),
        Friction {
            coefficient: 0.0,
            ..default()
        },
        RigidBody::Fixed,
        Transform::from_translation(Vec3::new(-112.0 + -112.0 * cos(30_f32.to_radians()), 112.0 * sin(30_f32.to_radians()), 0.0)),
        Name::new("Debug Floor"),
        Ground,
    ));

    // spawn camera
    commands.spawn((
        Camera3d { ..default() },
        Projection::Perspective(PerspectiveProjection {
            fov: 35_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        CameraDistance(120.),
        MainCamera,
        Name::new("MainCamera"),
    ));
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
