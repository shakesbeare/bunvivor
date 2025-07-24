#![allow(clippy::type_complexity)]
#![allow(unused)]

use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css::{SILVER, WHITE};
use bevy::prelude::App as BevyApp;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
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

        app.add_systems(Update, (move_entities, accel_entities));

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

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Player;

/// Describes the move speed of the player in terms of background tiles per second
#[derive(Component, Deref, DerefMut)]
pub struct MoveSpeed(pub f32);

/// Describes the direction an entity is trying to move
#[derive(Component, Deref, DerefMut)]
pub struct MoveVector(pub Vec3);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct Velocity(Vec3);

impl Default for Velocity {
    fn default() -> Self {
        Velocity(Vec3::ZERO)
    }
}

impl Velocity {
    pub fn normalize(&mut self) {
        self.0 = self.0.normalize_or(Vec3::ZERO);
    }

    pub fn scale(&mut self, scalar: f32) {
        self.0 *= scalar;
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Acceleration(Vec3);

#[derive(Component, Deref, DerefMut)]
pub struct AimAtPoint(Vec3);

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
            radius: 2.0,
            half_length: 2.0,
        })),
        MeshMaterial3d(materials.add(Color::from(WHITE))),
        InputMap::new([
            (Action::Left, KeyCode::ArrowLeft),
            (Action::Right, KeyCode::ArrowRight),
            (Action::Up, KeyCode::ArrowUp),
            (Action::Down, KeyCode::ArrowDown),
        ]),
        MoveSpeed(23.6),
        MoveVector(Vec3::ZERO),
        Velocity(Vec3::ZERO),
        Acceleration(Vec3::ZERO),
        Player,
        Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
        Name::new("Player")
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
        Name::new("Sun")
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(112.0, 112.0).subdivisions(10))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Name::new("Debug Floor")
    ));

    // spawn camera
    commands.spawn((
        Camera3d { ..default() },
        // Projection::Orthographic(OrthographicProjection {
        //     far: 5000.0,
        //     ..OrthographicProjection::default_3d()
        // }),
        Projection::Perspective(PerspectiveProjection {
            fov: 35_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        CameraDistance(120.),
        MainCamera,
        Name::new("MainCamera")
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

fn move_entities(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta().as_secs_f32();
    for (mut t, v) in query.iter_mut() {
        t.translation.x += v.x * dt;
        t.translation.y += v.y * dt;
        t.translation.z += v.z * dt;
    }
}

fn accel_entities(time: Res<Time>, mut query: Query<(&mut Velocity, &Acceleration)>) {
    let dt = time.delta().as_secs_f32();
    for (mut v, a) in query.iter_mut() {
        v.x += a.x * dt;
        v.y += a.y * dt;
        v.z += a.z * dt;
    }
}
