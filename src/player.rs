use crate::components::ShipStats;
use bevy::prelude::*;
use bevy::core::FixedTimestep;
use crate::{PlayerState, GameTextures, WinSize, PLAYER_RESPAWN_DELAY, PLAYER_SIZE, SPRITE_SCALE, TIME_STEP, BASE_SPEED, PLAYER_LASER_SIZE};
use crate::components::{FromPlayer, Movable, Player, SpriteSize, Velocity, Laser};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerState::default())
            .add_system_set(
                SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.5))
                .with_system(player_spawn_system),
            )
            .add_system(player_keyboard_event_system )
            .add_system(player_fire_system);
    }
}

fn player_spawn_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>
) {
    let now = time.seconds_since_startup();
    let last_shot = player_state.last_shot;

    if !player_state.on && (last_shot == -1. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        let bottom = -win_size.h / 2.;

        commands.spawn_bundle(SpriteBundle {
            texture: game_textures.player.clone(),
            transform: Transform {
                translation: Vec3::new(0., bottom + PLAYER_SIZE.1 / 2. + 5., 10.),
                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
            .insert(Player)
            .insert(SpriteSize::from(PLAYER_SIZE))
            .insert(Movable{auto_despawn: false})
            .insert(Velocity {x: 0., y: 0.})
            .insert(ShipStats { accel_speed: 0.3, decel_speed: 10.0, max_speed: 0.7, laser_speed: 1.0 });

        player_state.spawned();
    }

}

fn player_fire_system(mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    game_textures: Res<GameTextures>,
    query: Query<&Transform, With<Player>>) 
{
    if let Ok(player_tf) = query.get_single() {
        if kb.just_pressed(KeyCode::Space) {
            let (x, y) = (player_tf.translation.x, player_tf.translation.y);
            let x_offset = PLAYER_SIZE.0 / 2. * SPRITE_SCALE - 5.;

            let mut spawn_laser = |x_offset: f32| {
                commands.spawn_bundle(SpriteBundle {
                    texture: game_textures.player_laser.clone(),
                    transform: Transform {
                        translation: Vec3::new(x + x_offset, y + 15., 0.),
                        scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Laser)
                .insert(FromPlayer)
                .insert(SpriteSize::from(PLAYER_LASER_SIZE))
                .insert(Movable{auto_despawn: true})
                .insert(Velocity {x: 0., y: 1.});
            };
            spawn_laser(x_offset);
            spawn_laser(-x_offset);

        }
            
    }
}

fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &ShipStats), With<Player>>
) {
    if let Ok((mut velocity, ship_stats)) = query.get_single_mut() {
        velocity.x += if kb.pressed(KeyCode::Left) {
            -ship_stats.accel_speed
        } else if kb.pressed(KeyCode::Right) {
            ship_stats.accel_speed
        } else {
            -velocity.x / ship_stats.decel_speed
        };
        // velocity.x = clamp(velocity.x, -1., 1.);
        if velocity.x > ship_stats.max_speed { velocity.x = ship_stats.max_speed }
        if velocity.x < -ship_stats.max_speed { velocity.x = -ship_stats.max_speed }

        velocity.y += if kb.pressed(KeyCode::Up) {
            ship_stats.accel_speed
        } else if kb.pressed(KeyCode::Down) {
            -ship_stats.accel_speed
        } else {
            -velocity.y / ship_stats.decel_speed
        };
        // velocity.y = clamp(velocity.x, -1., 1.);
        if velocity.y > ship_stats.max_speed { velocity.y = ship_stats.max_speed }
        if velocity.y < -ship_stats.max_speed { velocity.y = -ship_stats.max_speed }
    }
}

