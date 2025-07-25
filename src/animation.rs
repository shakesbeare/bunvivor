use bevy::prelude::*;

const LEVEL_UP_INDICES: AnimationIndices = AnimationIndices::new(8, 15, AnimationMode::Once);
const PLAYER_RUN_INDICES: AnimationIndices = AnimationIndices::new(1, 3, AnimationMode::Bounce);
const BLUEBERRY_INDICES: AnimationIndices = AnimationIndices::new(12, 13, AnimationMode::Cycle);
const GRAPE_INDICES: AnimationIndices = AnimationIndices::new(68, 69, AnimationMode::Cycle);
const BANANA_INDICES: AnimationIndices = AnimationIndices::new(4, 5, AnimationMode::Cycle);
const MELON_INDICES: AnimationIndices = AnimationIndices::new(25, 26, AnimationMode::Cycle);
const WITCH_IDLE_INDICES: AnimationIndices = AnimationIndices::new(6, 7, AnimationMode::Cycle);
const WITCH_ATTACK_INDICES: AnimationIndices = AnimationIndices::new(10, 11, AnimationMode::Cycle);

pub enum SpriteScale {
    X32,
    X16,
    X8,
}
impl SpriteScale {
    const WITCH: Self = Self::X32;
    const BANANA: Self = Self::X16;
    const MELON: Self = Self::X16;
    const BLUEBERRY: Self = Self::X16;
    const GRAPE: Self = Self::X16;
    const PLAYER: Self = Self::X16;
}

pub fn get_texture_atlas_layout(scale: SpriteScale) -> TextureAtlasLayout {
    match scale {
        SpriteScale::X8 => {
            return TextureAtlasLayout::from_grid(UVec2::splat(8), 16, 16, None, None);
        }
        SpriteScale::X16 => {
            return TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 8, None, None);
        }
        SpriteScale::X32 => {
            return TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 4, None, None);
        }
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprites);
    }
}

#[derive(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
    mode: AnimationMode,
    dir: AnimationDir,
    cur: usize,
}

pub enum AnimationDir {
    Forward,
    Backward,
}

pub enum AnimationMode {
    Cycle,
    Bounce,
    Once,
}

impl AnimationIndices {
    pub const fn new(first: usize, last: usize, mode: AnimationMode) -> Self {
        Self {
            first,
            last,
            mode,
            dir: AnimationDir::Forward,
            cur: first,
        }
    }

    /// Advance the animator to the next frame
    /// Returns the index of the new frame
    pub fn next(&mut self) -> usize {
        match self.mode {
            AnimationMode::Once => {
                if self.cur <= self.last {
                    self.cur += 1;
                }
            }
            AnimationMode::Cycle => {
                self.cur += 1;
                if self.cur > self.last {
                    self.cur = self.first;
                }
            }
            AnimationMode::Bounce => {
                match self.dir {
                    AnimationDir::Forward => self.cur += 1,
                    AnimationDir::Backward => self.cur -= 1,
                }

                if self.cur == self.last {
                    self.dir = AnimationDir::Backward;
                } else if self.cur == self.first {
                    self.dir = AnimationDir::Forward;
                }
            }
        };

        return self.cur;
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (mut indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = indices.next();
            }
        }
    }
}
