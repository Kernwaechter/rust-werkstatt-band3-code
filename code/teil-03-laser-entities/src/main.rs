use bevy::prelude::*;
use rand::Rng;

const HALBE_BREITE: f32 = 640.0;
const HALBE_HOEHE: f32 = 360.0;

#[derive(Component)]
struct Spieler;

#[derive(Component)]
struct Laser {
    lebensdauer: Timer,
}

#[derive(Component)]
struct Komet;

#[derive(Component, Default)]
struct Geschwindigkeit(Vec2);

#[derive(Resource)]
struct SchussCooldown(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kometen-Jäger — Teil 3 (Laser & Entities)".into(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(SchussCooldown(Timer::from_seconds(0.18, TimerMode::Once)))
        .add_systems(Startup, szene_aufbauen)
        .add_systems(
            Update,
            (
                spieler_steuern,
                laser_abfeuern,
                bewegung_und_wrap,
                laser_aufraeumen,
            ),
        )
        .run();
}

fn szene_aufbauen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_image(asset_server.load("sprites/weltraum_hintergrund.png")),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    commands.spawn((
        Spieler,
        Geschwindigkeit::default(),
        Sprite {
            image: asset_server.load("sprites/raumschiff.png"),
            custom_size: Some(Vec2::splat(72.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    let mut rng = rand::thread_rng();
    for _ in 0..6 {
        let winkel = rng.gen_range(0.0..std::f32::consts::TAU);
        let pos = Vec2::new(
            rng.gen_range(-520.0..520.0),
            rng.gen_range(-280.0..280.0),
        );
        let tempo = Vec2::from_angle(winkel) * rng.gen_range(60.0..130.0);
        commands.spawn((
            Komet,
            Geschwindigkeit(tempo),
            Sprite {
                image: asset_server.load("sprites/komet_gross.png"),
                custom_size: Some(Vec2::splat(88.0)),
                ..default()
            },
            Transform::from_translation(pos.extend(0.0)),
        ));
    }
}

fn spieler_steuern(
    tastatur: Res<ButtonInput<KeyCode>>,
    zeit: Res<Time>,
    mut abfrage: Query<(&mut Transform, &mut Geschwindigkeit), With<Spieler>>,
) {
    let Ok((mut transform, mut tempo)) = abfrage.get_single_mut() else {
        return;
    };
    let dt = zeit.delta_secs();
    if tastatur.pressed(KeyCode::KeyA) || tastatur.pressed(KeyCode::ArrowLeft) {
        transform.rotate_z(4.2 * dt);
    }
    if tastatur.pressed(KeyCode::KeyD) || tastatur.pressed(KeyCode::ArrowRight) {
        transform.rotate_z(-4.2 * dt);
    }
    if tastatur.pressed(KeyCode::KeyW) || tastatur.pressed(KeyCode::ArrowUp) {
        let blick = (transform.rotation * Vec3::Y).truncate();
        tempo.0 += blick * 520.0 * dt;
    }
    tempo.0 *= 0.985;
}

fn laser_abfeuern(
    mut commands: Commands,
    tastatur: Res<ButtonInput<KeyCode>>,
    zeit: Res<Time>,
    mut cooldown: ResMut<SchussCooldown>,
    asset_server: Res<AssetServer>,
    abfrage: Query<&Transform, With<Spieler>>,
) {
    cooldown.0.tick(zeit.delta());
    if !tastatur.pressed(KeyCode::Space) || !cooldown.0.finished() {
        return;
    }
    let Ok(transform) = abfrage.get_single() else {
        return;
    };
    cooldown.0.reset();
    let blick = (transform.rotation * Vec3::Y).truncate();
    commands.spawn((
        Laser {
            lebensdauer: Timer::from_seconds(1.2, TimerMode::Once),
        },
        Geschwindigkeit(blick * 820.0),
        Sprite {
            image: asset_server.load("sprites/laser.png"),
            custom_size: Some(Vec2::new(18.0, 48.0)),
            ..default()
        },
        Transform {
            translation: transform.translation + (blick * 36.0).extend(0.0),
            rotation: transform.rotation,
            ..default()
        },
    ));
}

fn bewegung_und_wrap(zeit: Res<Time>, mut abfrage: Query<(&mut Transform, &Geschwindigkeit)>) {
    let dt = zeit.delta_secs();
    for (mut transform, tempo) in &mut abfrage {
        transform.translation += tempo.0.extend(0.0) * dt;
        if transform.translation.x > HALBE_BREITE {
            transform.translation.x = -HALBE_BREITE;
        } else if transform.translation.x < -HALBE_BREITE {
            transform.translation.x = HALBE_BREITE;
        }
        if transform.translation.y > HALBE_HOEHE {
            transform.translation.y = -HALBE_HOEHE;
        } else if transform.translation.y < -HALBE_HOEHE {
            transform.translation.y = HALBE_HOEHE;
        }
    }
}

fn laser_aufraeumen(
    mut commands: Commands,
    zeit: Res<Time>,
    mut abfrage: Query<(Entity, &mut Laser)>,
) {
    for (entity, mut laser) in &mut abfrage {
        laser.lebensdauer.tick(zeit.delta());
        if laser.lebensdauer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
