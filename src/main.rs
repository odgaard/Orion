use crate::components::ScoreText;
use bevy::utils::HashSet;
use bevy::math::Vec3Swizzles;
use bevy::sprite::collide_aabb::collide;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::components::{FpsText, FromEnemy, Explosion, ExplosionTimer, ExplosionToSpawn, SpriteSize, Laser, FromPlayer, Enemy, Movable, Velocity, Player};
use player::PlayerPlugin;
use enemy::EnemyPlugin;

mod components;
mod enemy;
mod player;


const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144., 75.);
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMEY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMEY_LASER_SIZE: (f32, f32) = (17., 55.);

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize = 16;

const ENEMY_MAX: u32 = 4;
const FORMATION_MEMBERS_MAX: u32 = 2;

const PLAYER_RESPAWN_DELAY: f64 = 2.0;
const SPRITE_SCALE: f32 = 0.5;
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;

pub struct WinSize {
    pub w: f32,
    pub h: f32
}

struct GameTextures {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion: Handle<TextureAtlas>
}

struct EnemyCount(u32);

struct PlayerState {
    on: bool,
    health: i64,
    last_shot: f64,
    score: i64,
}
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: -1.,
            health: 3,
            score: 0,
        }
    }
}

impl PlayerState {
    pub fn shot(&mut self, time: f64) {
        self.health -= 1;
        self.last_shot = time;
        if self.health <= 0 {
            self.on = false;        
            self.score = 0;
        }
    }

    pub fn spawned(&mut self) {
        self.on = true;
        self.last_shot = -1.;
        self.health = 3;
        self.score = 0;
    }
}


fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Rust Invaders!".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        })
    .add_plugins(DefaultPlugins)
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(PlayerPlugin)
    .add_plugin(EnemyPlugin)
    .add_startup_system(setup_system)
    .add_system(movable_system)
    .add_system(player_laser_hit_enemy_system)
    .add_system(enemy_laser_hit_player_system)
    .add_system(explosion_to_spawn_system)
    .add_system(explosion_animation_system)
    .add_system(fps_update_system)
    .add_system(score_update_system)
    .run();
}

fn setup_system(mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut windows: ResMut<Windows>) {
    commands.spawn_bundle(Camera2dBundle::default());


    let window = windows.get_primary_mut().unwrap();
    let (_win_w, win_h) = (window.width(), window.height());

    window.set_position(IVec2::new(900, 1400));

    let win_size = WinSize { w: win_h, h: win_h };
    commands.insert_resource(win_size);

    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4);
    let explosion = texture_atlases.add(texture_atlas);

    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMEY_LASER_SPRITE),
        explosion,
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0));
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
    .with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexEnd,
                ..default()
                },
                color: Color::NONE.into(),
                ..default()
            })

        .with_children(|parent| {
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(250.0), Val::Auto),
                ..default()
                },
                image: asset_server.load("logo.png").into(),
                ..default()
            });
        });
    parent.spawn_bundle(
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 60.0,
                color: Color::GOLD,
            }),
        ])
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            ..default()
        }),
    )
    .insert(FpsText);
    parent.spawn_bundle(
        TextBundle::from_sections([
            TextSection::new(
                "Score : ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 60.0,
                color: Color::GREEN,
            }),
        ])
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            ..default()
        }),
    )
    .insert(ScoreText);
    });
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        translation.y += velocity.y * TIME_STEP * BASE_SPEED;

        if movable.auto_despawn {
            const MARGIN: f32 = 200.;
            if translation.y > win_size.h / 2. + MARGIN
             || translation.y < -win_size.h / 2. - MARGIN
             || translation.x > win_size.w / 2. + MARGIN
             || translation.x < -win_size.w / 2. - MARGIN 
            {
                commands.entity(entity).despawn();
            }
        }
    }
}


fn player_laser_hit_enemy_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    mut enemy_count: ResMut<EnemyCount>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>
) {

    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    for (laser_entity, laser_tf, laser_size) in laser_query.iter() {

        if despawned_entities.contains(&laser_entity) {
            continue;
        }

        let laser_scale = Vec2::from(laser_tf.scale.xy());

        for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
            if despawned_entities.contains(&laser_entity) 
                || despawned_entities.contains(&enemy_entity){
                continue;
            }


            let enemy_scale = Vec2::from(enemy_tf.scale.xy());
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                enemy_tf.translation,
                enemy_size.0 * enemy_scale,
            );

            if let Some(_) = collision {
                player_state.score += 1;
                commands.entity(enemy_entity).despawn();
                despawned_entities.insert(enemy_entity);
                enemy_count.0 -= 1;
                commands.entity(laser_entity).despawn();
                despawned_entities.insert(laser_entity);
                commands.spawn().insert(ExplosionToSpawn(enemy_tf.translation.clone()));
            }
        }
    }
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        commands.spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.explosion.clone(),
            transform: Transform {
                translation: explosion_to_spawn.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Explosion)
        .insert(ExplosionTimer::default());
        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) 
{
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1;
            if sprite.index >= EXPLOSION_LEN {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn enemy_laser_hit_player_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {
    if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
        let player_scale = Vec2::from(player_tf.scale.xy());
        for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
            let laser_scale = Vec2::from(laser_tf.scale.xy());

            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                player_tf.translation,
                player_size.0 * player_scale,
            );

            if let Some(_) = collision {
                player_state.shot(time.seconds_since_startup());
                if player_state.on == false {
                    commands.entity(player_entity).despawn();
                }
                commands.entity(laser_entity).despawn();
                break;
            }
        }
    }
}


fn fps_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.sections[1].value = format!("{average:.2}");
            }
        }
    }
}

fn score_update_system(
    player_state: ResMut<PlayerState>,
    mut query: Query<&mut Text, With<ScoreText>>
) {
    let score = player_state.score;
    for mut text in &mut query {
        text.sections[1].value = format!("{score}");
    }
}

