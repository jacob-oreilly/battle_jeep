use bevy::{
    math::bounding::{Aabb2d, Bounded2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier2d::na::ComplexField;
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
    bomb_spawn_timer: Timer,
    number_of_bombs: i32,
}

#[derive(Component)]
struct Bomb {
    falling_speed: f32,
}

#[derive(Resource)]
struct PlaneSpawnTimer {
    timer: Timer,
}

impl Default for PlaneSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<PlaneSpawnTimer>()
        .add_event::<CollisionEvent>()
        .add_systems(Startup, (setup_camera, spawn_player))
        .add_systems(
            Update,
            (
                move_player,
                fire_rocket,
                spawn_planes,
                spawn_bombs,
                plane_spawn_timer_update,
                bomb_spawn_timer_update,
                plane_update.run_if(run_if_planes),
                bomb_spawn_timer_update.run_if(run_if_planes),
                rocket_update.run_if(run_if_rockets),
                update_bombs.run_if(run_if_bombs),
            ),
        )
        .add_systems(FixedUpdate, rocket_collision.run_if(run_if_rockets_and_planes))
        .run();
}

fn setup_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..Default::default()
    });
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/jeep.png"),
            transform: Transform::from_xyz(window.width() / 2.0, 32.0, 0.0)
                .with_scale(Vec3::new(2.0, 2.0, 0.0)),
            ..default()
        },
        Player {
            movement_speed: 500.0,
        },
        Collider,
    ));
}

fn move_player(
    mut player_query: Query<(&mut Transform, &Player), With<Player>>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
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
            Rocket {
                movement_speed: 600.0,
            },
        ));
    }
}

fn rocket_update(
    mut commands: Commands,
    time: Res<Time>,
    mut rocket_query: Query<(&mut Transform, Entity, &Rocket), With<Rocket>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    for (mut rocket_transform, rocket_entity, rocket) in &mut rocket_query {
        if rocket_transform.translation.y < window.height() {
            rocket_transform.translation.y += rocket.movement_speed * time.delta_seconds();
        } else {
            commands.entity(rocket_entity).despawn();
        }
    }
}

fn spawn_planes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    plane_spawn_timer: Res<PlaneSpawnTimer>,
) {
    let window = window_query.get_single().unwrap();
    if plane_spawn_timer.timer.finished() {
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("../assets/plane.png"),
                transform: Transform::from_xyz(window.width(), window.height() - 100.0, 0.0)
                    .with_scale(Vec3::new(2.0, 2.0, 0.0)),
                ..default()
            },
            Plane {
                movement_speed: 100.0,
                bomb_spawn_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                number_of_bombs: 1,
            },
            Collider,
        ));
    }
}

fn plane_update(
    mut commands: Commands,
    time: Res<Time>,
    mut plane_query: Query<(&mut Transform, Entity, &Plane), With<Plane>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    for (mut plane_transform, plane_entity, plane) in &mut plane_query {
        if plane_transform.translation.y < window.height() {
            plane_transform.translation.x -= plane.movement_speed * time.delta_seconds();
        } else {
            commands.entity(plane_entity).despawn();
        }
    }
}

fn spawn_bombs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    plane_query: Query<(&Transform, &Plane), With<Plane>>,
) {
    for (plane_transform, plane) in plane_query.iter() {
        if plane.bomb_spawn_timer.finished() {
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("../assets/bomb.png"),
                    transform: Transform::from_translation(plane_transform.translation)
                        .with_scale(Vec3::new(2.0, 2.0, 0.0)),
                    ..default()
                },
                Bomb {
                    falling_speed: 100.0,
                },
            ));
        }
    }
}

fn update_bombs(
    mut commands: Commands,
    time: Res<Time>,
    mut bomb_query: Query<(&mut Transform, Entity, &Bomb), With<Bomb>>,
) {
    for (mut bomb_transform, bomb_entity, bomb) in &mut bomb_query {
        if bomb_transform.translation.y > -16.0 {
            bomb_transform.translation.y -= bomb.falling_speed * time.delta_seconds();
        } else {
            commands.entity(bomb_entity).despawn();
        }
    }
}

fn rocket_collision(
    mut commands: Commands,
    rocket_query: Query<(Entity, &Transform), With<Rocket>>,
    collider_query: Query<(Entity, &Transform, Option<&Plane>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    for (rocket_entity, rocket_transform) in rocket_query.iter() {
        for (collider_entity, collider_transform, plane) in &collider_query {
            let collision = is_collision(
                Aabb2d::new(
                    rocket_transform.translation.truncate(),
                    rocket_transform.scale.truncate() / 2.0,
                ),
                Aabb2d::new(
                    collider_transform.translation.truncate(),
                    collider_transform.scale.truncate() / 2.,
                ),
            );

            if let Some(collision) = collision {
                collision_events.send_default();
                if plane.is_some() {
                    commands.entity(collider_entity).despawn();
                    commands.entity(rocket_entity).despawn();
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

fn is_collision(colliding: Aabb2d, collider: Aabb2d) -> Option<Collision> {
    if !&colliding.intersects(&collider) {
        println!("rocket: {:?}, plane: {:?}", colliding, collider);
        return None;
    }

    let closest = collider.closest_point(colliding.center());
    let offset = colliding.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn plane_spawn_timer_update(mut plane_spawn_timer: ResMut<PlaneSpawnTimer>, time: Res<Time>) {
    plane_spawn_timer.timer.tick(time.delta());
}

fn bomb_spawn_timer_update(
    mut bomb_spawn_timer_query: Query<&mut Plane, With<Plane>>,
    time: Res<Time>,
) {
    for mut plane in bomb_spawn_timer_query.iter_mut() {
        plane.bomb_spawn_timer.tick(time.delta());
    }
}
fn run_if_rockets(rocket_query: Query<(), With<Rocket>>) -> bool {
    !rocket_query.is_empty()
}

fn run_if_planes(plane_query: Query<(), With<Plane>>) -> bool {
    !plane_query.is_empty()
}

fn run_if_bombs(bomb_query: Query<(), With<Bomb>>) -> bool {
    !bomb_query.is_empty()
}
fn run_if_rockets_and_planes(
    plane_query: Query<(), With<Plane>>,
    rocket_query: Query<(), With<Rocket>>,
) -> bool {
    if !rocket_query.is_empty() && !plane_query.is_empty() {
        true
    } else {
        false
    }
}
