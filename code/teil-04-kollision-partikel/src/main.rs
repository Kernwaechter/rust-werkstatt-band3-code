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
struct Komet {
    groesse: f32,
    klein: bool,
    drehung: f32,
}

#[derive(Component)]
struct Partikel {
    lebensdauer: Timer,
    max_zeit: f32,
}

#[derive(Component, Default)]
struct Geschwindigkeit(Vec2);

#[derive(Resource)]
struct SchussCooldown(Timer);

#[derive(Resource, Default)]
struct ScreenShake(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kometen-Jäger — Teil 4 (Kollision & Partikel)".into(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(SchussCooldown(Timer::from_seconds(0.18, TimerMode::Once)))
        .init_resource::<ScreenShake>()
        .add_systems(Startup, szene_aufbauen)
        .add_systems(
            Update,
            (
                spieler_steuern,
                laser_abfeuern,
                bewegung_und_wrap,
                kometen_drehen,
                kollision_laser_komet,
                partikel_aktualisieren,
                kamera_wackeln,
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
        Transform::from_xyz(0.0, 0.0, 2.0),
    ));

    let mut rng = rand::thread_rng();
    for _ in 0..6 {
        let pos = Vec2::new(rng.gen_range(-520.0..520.0), rng.gen_range(-280.0..280.0));
        let winkel = rng.gen_range(0.0..std::f32::consts::TAU);
        commands.spawn((
            Komet {
                groesse: 88.0,
                klein: false,
                drehung: rng.gen_range(-1.8..1.8),
            },
            Geschwindigkeit(Vec2::from_angle(winkel) * rng.gen_range(70.0..130.0)),
            Sprite {
                image: asset_server.load("sprites/komet_gross.png"),
                custom_size: Some(Vec2::splat(88.0)),
                ..default()
            },
            Transform::from_translation(pos.extend(1.0)),
        ));
    }
}

fn spieler_steuern(
    mut commands: Commands,
    tastatur: Res<ButtonInput<KeyCode>>,
    zeit: Res<Time>,
    asset_server: Res<AssetServer>,
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
        let heck = transform.translation - (blick * 28.0).extend(0.0);
        commands.spawn((
            Partikel {
                lebensdauer: Timer::from_seconds(0.22, TimerMode::Once),
                max_zeit: 0.22,
            },
            Geschwindigkeit(-blick * 120.0),
            Sprite {
                image: asset_server.load("sprites/partikel_funke.png"),
                color: Color::srgba(1.0, 0.55, 0.15, 0.85),
                custom_size: Some(Vec2::splat(20.0)),
                ..default()
            },
            Transform::from_translation(heck),
        ));
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

fn kometen_drehen(zeit: Res<Time>, mut abfrage: Query<(&mut Transform, &Komet)>) {
    let dt = zeit.delta_secs();
    for (mut transform, komet) in &mut abfrage {
        transform.rotate_z(komet.drehung * dt);
    }
}

fn kollision_laser_komet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut shake: ResMut<ScreenShake>,
    laser_q: Query<(Entity, &Transform), With<Laser>>,
    komet_q: Query<(Entity, &Transform, &Komet)>,
) {
    let mut rng = rand::thread_rng();
    for (l_ent, l_tf) in &laser_q {
        for (k_ent, k_tf, komet) in &komet_q {
            let abstand = l_tf.translation.truncate().distance(k_tf.translation.truncate());
            if abstand < komet.groesse * 0.45 {
                commands.entity(l_ent).despawn();
                commands.entity(k_ent).despawn();
                shake.0 = 8.0;

                for _ in 0..14 {
                    let winkel = rng.gen_range(0.0..std::f32::consts::TAU);
                    let geschw = Vec2::from_angle(winkel) * rng.gen_range(80.0..260.0);
                    commands.spawn((
                        Partikel {
                            lebensdauer: Timer::from_seconds(0.4, TimerMode::Once),
                            max_zeit: 0.4,
                        },
                        Geschwindigkeit(geschw),
                        Sprite {
                            image: asset_server.load("sprites/partikel_funke.png"),
                            color: Color::srgba(0.4, 0.85, 1.0, 0.95),
                            custom_size: Some(Vec2::splat(26.0)),
                            ..default()
                        },
                        Transform::from_translation(k_tf.translation),
                    ));
                }

                if !komet.klein {
                    for _ in 0..2 {
                        let winkel = rng.gen_range(0.0..std::f32::consts::TAU);
                        commands.spawn((
                            Komet {
                                groesse: 48.0,
                                klein: true,
                                drehung: rng.gen_range(-3.0..3.0),
                            },
                            Geschwindigkeit(Vec2::from_angle(winkel) * rng.gen_range(120.0..200.0)),
                            Sprite {
                                image: asset_server.load("sprites/komet_klein.png"),
                                custom_size: Some(Vec2::splat(48.0)),
                                ..default()
                            },
                            Transform::from_translation(k_tf.translation),
                        ));
                    }
                }
                break;
            }
        }
    }
}

fn partikel_aktualisieren(
    mut commands: Commands,
    zeit: Res<Time>,
    mut abfrage: Query<(Entity, &mut Partikel, &mut Sprite)>,
) {
    for (ent, mut partikel, mut sprite) in &mut abfrage {
        partikel.lebensdauer.tick(zeit.delta());
        let rest = 1.0 - (partikel.lebensdauer.elapsed_secs() / partikel.max_zeit);
        sprite.color.set_alpha(rest.clamp(0.0, 1.0));
        if partikel.lebensdauer.finished() {
            commands.entity(ent).despawn();
        }
    }
}

fn kamera_wackeln(
    zeit: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut kamera: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut tf) = kamera.get_single_mut() else {
        return;
    };
    if shake.0 > 0.2 {
        let mut rng = rand::thread_rng();
        tf.translation.x = rng.gen_range(-shake.0..shake.0);
        tf.translation.y = rng.gen_range(-shake.0..shake.0);
        shake.0 *= (1.0 - 10.0 * zeit.delta_secs()).max(0.0);
    } else {
        shake.0 = 0.0;
        tf.translation.x = 0.0;
        tf.translation.y = 0.0;
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
