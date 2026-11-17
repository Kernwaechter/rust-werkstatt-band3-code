use bevy::prelude::*;

const DREH_GESCHWINDIGKEIT: f32 = 4.2;
const SCHUB_KRAFT: f32 = 520.0;
const REIBUNG: f32 = 0.985;
const HALBE_BREITE: f32 = 640.0;
const HALBE_HOEHE: f32 = 360.0;

#[derive(Component)]
struct Spieler;

#[derive(Component, Default)]
struct Geschwindigkeit(Vec2);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kometen-Jäger — Teil 2 (Steuerung)".into(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, szene_aufbauen)
        .add_systems(Update, (spieler_steuern, bewegung_und_wrap).chain())
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
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
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
        transform.rotate_z(DREH_GESCHWINDIGKEIT * dt);
    }
    if tastatur.pressed(KeyCode::KeyD) || tastatur.pressed(KeyCode::ArrowRight) {
        transform.rotate_z(-DREH_GESCHWINDIGKEIT * dt);
    }
    if tastatur.pressed(KeyCode::KeyW) || tastatur.pressed(KeyCode::ArrowUp) {
        let blick = (transform.rotation * Vec3::Y).truncate();
        tempo.0 += blick * SCHUB_KRAFT * dt;
    }
    tempo.0 *= REIBUNG;
}

fn bewegung_und_wrap(
    zeit: Res<Time>,
    mut abfrage: Query<(&mut Transform, &Geschwindigkeit)>,
) {
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
