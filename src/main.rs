use bevy::{prelude::*, window::PrimaryWindow};
use rand::prelude::*;

#[derive(Component)]
struct Player {
    movement_speed: f32,
}

#[derive(Component)]
struct Rocket {
    movement_speed: f32,
}

#[derive(Component)]
struct Plane {
    movement_speed: f32,
}

#[derive(Resource)]
struct PlaneSpawnTimer {
    timer: Timer,
}

impl Default for PlaneSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Repeating)
        }
    }
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<PlaneSpawnTimer>()
        .add_systems(Startup, (setup_camera, spawn_player))
        .add_systems(Update, (move_player, fire_rocket, spawn_planes, plane_spawn_timer_update, plane_update.run_if(run_if_planes), rocket_update.run_if(run_if_rockets)))
        .run();
}

fn setup_camera(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..Default::default()
    });
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_query.get_single().unwrap();
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/jeep.png"),
            transform: Transform::from_xyz(window.width() / 2.0, 32.0, 0.0).with_scale(Vec3::new(2.0, 2.0, 0.0)),
            ..default()
        },
        Player {
            movement_speed: 500.0,
        }
    ));
}

fn move_player(
    mut player_query: Query<(&mut Transform, &Player), With<Player>>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>
) {
    let (mut player_transform, player) = player_query.get_single_mut().unwrap();

    let mut direction = 0.0;
    if key_input.pressed(KeyCode::ArrowLeft) {
        direction += -1.0;
    }
    if key_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }
    player_transform.translation.x += player.movement_speed * direction * time.delta_seconds();
}

fn fire_rocket(
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
    key_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    let player_transform = player_query.get_single().unwrap();
    let player_loc: Vec3 = player_transform.translation;
    if key_input.just_pressed(KeyCode::Space) {
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("../assets/rocket.png"),
                transform: Transform::from_translation(player_loc),
                ..default()
            },
            Rocket{
                movement_speed: 600.0
            }
        ));
    }
}

fn rocket_update(
    mut commands: Commands,
    time: Res<Time>,
    mut rocket_query: Query<(&mut Transform, Entity, &Rocket), With<Rocket>>,
    window_query: Query<&Window, With<PrimaryWindow>>
) { 
    let window = window_query.get_single().unwrap();
    for (mut rocket_transform, rocket_entity, rocket) in &mut rocket_query {
        if rocket_transform.translation.y < window.height() {
            rocket_transform.translation.y += rocket.movement_speed * time.delta_seconds();
        }
        else {
            commands.entity(rocket_entity).despawn();
        }
    }
}

fn spawn_planes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    plane_spawn_timer: Res<PlaneSpawnTimer>
) {
    let window = window_query.get_single().unwrap();
    if plane_spawn_timer.timer.finished() {
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("../assets/plane.png"),
                transform: Transform::from_xyz(window.width(), window.height() - 100.0, 0.0).with_scale(Vec3::new(2.0, 2.0, 0.0)),
                ..default()
            },
            Plane {
                movement_speed: 300.0
            }
        ));
    }

}

fn plane_update(
    mut commands: Commands,
    time: Res<Time>,
    mut plane_query: Query<(&mut Transform, Entity, &Plane), With<Plane>>,
    window_query: Query<&Window, With<PrimaryWindow>>
) { 
    let window = window_query.get_single().unwrap();
    for (mut plane_transform, plane_entity, plane) in &mut plane_query {
        if plane_transform.translation.y < window.height() {
            plane_transform.translation.x -= plane.movement_speed * time.delta_seconds();
        }
        else {
            commands.entity(plane_entity).despawn();
        }
    }
}

fn plane_spawn_timer_update(mut plane_spawn_timer: ResMut<PlaneSpawnTimer>, time: Res<Time>) {
    plane_spawn_timer.timer.tick(time.delta());
}

fn run_if_rockets(
    rocket_query: Query<(), With<Rocket>>,
) -> bool {
    !rocket_query.is_empty()
}

fn run_if_planes(
    plane_query: Query<(), With<Plane>>
) -> bool{
    !plane_query.is_empty()
}