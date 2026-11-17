use bevy::prelude::*;
use rand::Rng;

const HALBE_BREITE: f32 = 640.0;
const HALBE_HOEHE: f32 = 360.0;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum SpielZustand {
    #[default]
    StartMenue,
    Laeuft,
    GameOver,
}

#[derive(Component)]
struct Spieler {
    unverwundbar: Timer,
}

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

#[derive(Component)]
struct SpielObjekt;

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct OverlayUi;

#[derive(Component, Default)]
struct Geschwindigkeit(Vec2);

#[derive(Resource)]
struct SpielDaten {
    punkte: u32,
    leben: u8,
    welle: u32,
}

impl Default for SpielDaten {
    fn default() -> Self {
        Self {
            punkte: 0,
            leben: 3,
            welle: 1,
        }
    }
}

#[derive(Resource)]
struct SchussCooldown(Timer);

#[derive(Resource, Default)]
struct ScreenShake(f32);

#[derive(Resource)]
struct SpielAssets {
    hintergrund: Handle<Image>,
    raumschiff: Handle<Image>,
    komet_gross: Handle<Image>,
    komet_klein: Handle<Image>,
    laser: Handle<Image>,
    funke: Handle<Image>,
    sound_laser: Handle<AudioSource>,
    sound_explosion: Handle<AudioSource>,
    sound_treffer: Handle<AudioSource>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kometen-Jäger — Das fertige Arcade-Spiel".into(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<SpielZustand>()
        .init_resource::<SpielDaten>()
        .init_resource::<ScreenShake>()
        .insert_resource(SchussCooldown(Timer::from_seconds(0.18, TimerMode::Once)))
        .add_systems(Startup, basis_einrichten)
        .add_systems(OnEnter(SpielZustand::StartMenue), startmenue_zeigen)
        .add_systems(OnExit(SpielZustand::StartMenue), overlay_entfernen)
        .add_systems(OnEnter(SpielZustand::Laeuft), spiel_starten)
        .add_systems(OnExit(SpielZustand::Laeuft), welt_aufraeumen)
        .add_systems(OnEnter(SpielZustand::GameOver), gameover_zeigen)
        .add_systems(OnExit(SpielZustand::GameOver), overlay_entfernen)
        .add_systems(
            Update,
            tastatur_menue.run_if(not(in_state(SpielZustand::Laeuft))),
        )
        .add_systems(
            Update,
            (
                spieler_steuern,
                laser_abfeuern,
                bewegung_und_wrap,
                kometen_drehen,
                kollision_laser_komet,
                kollision_spieler_komet,
                welle_pruefen,
                partikel_aktualisieren,
                kamera_wackeln,
                laser_aufraeumen,
                hud_aktualisieren,
            )
                .run_if(in_state(SpielZustand::Laeuft)),
        )
        .run();
}

fn basis_einrichten(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let assets = SpielAssets {
        hintergrund: asset_server.load("sprites/weltraum_hintergrund.png"),
        raumschiff: asset_server.load("sprites/raumschiff.png"),
        komet_gross: asset_server.load("sprites/komet_gross.png"),
        komet_klein: asset_server.load("sprites/komet_klein.png"),
        laser: asset_server.load("sprites/laser.png"),
        funke: asset_server.load("sprites/partikel_funke.png"),
        sound_laser: asset_server.load("audio/laser.wav"),
        sound_explosion: asset_server.load("audio/explosion.wav"),
        sound_treffer: asset_server.load("audio/treffer.wav"),
    };

    commands.spawn((
        Sprite::from_image(assets.hintergrund.clone()),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    commands.spawn((
        HudText,
        Text::new("Punkte: 0   |   Leben: 3   |   Welle: 1"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.96, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(22.0),
            ..default()
        },
    ));

    commands.insert_resource(assets);
}

fn startmenue_zeigen(mut commands: Commands) {
    overlay_spawnen(
        &mut commands,
        "KOMETEN-JÄGER",
        "W: Schub · A/D: Drehen · Leertaste: Laser\n\nDrücke [Enter] zum Starten",
    );
}

fn gameover_zeigen(mut commands: Commands, daten: Res<SpielDaten>) {
    let info = format!(
        "Erreichte Punkte: {} (Welle {})\n\nDrücke [Enter] für eine neue Runde",
        daten.punkte, daten.welle
    );
    overlay_spawnen(&mut commands, "GAME OVER", &info);
}

fn overlay_spawnen(commands: &mut Commands, titel: &str, untertitel: &str) {
    commands
        .spawn((
            OverlayUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.09, 0.72)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(titel),
                TextFont {
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgb(0.35, 0.85, 1.0)),
            ));
            parent.spawn((
                Text::new(untertitel),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.94, 1.0)),
            ));
        });
}

fn overlay_entfernen(mut commands: Commands, abfrage: Query<Entity, With<OverlayUi>>) {
    for ent in &abfrage {
        commands.entity(ent).despawn_recursive();
    }
}

fn tastatur_menue(
    tastatur: Res<ButtonInput<KeyCode>>,
    mut zustand: ResMut<NextState<SpielZustand>>,
) {
    if tastatur.just_pressed(KeyCode::Enter) {
        zustand.set(SpielZustand::Laeuft);
    }
}

fn spiel_starten(
    mut commands: Commands,
    assets: Res<SpielAssets>,
    mut daten: ResMut<SpielDaten>,
) {
    *daten = SpielDaten::default();
    commands.spawn((
        Spieler {
            unverwundbar: Timer::from_seconds(1.5, TimerMode::Once),
        },
        SpielObjekt,
        Geschwindigkeit::default(),
        Sprite {
            image: assets.raumschiff.clone(),
            custom_size: Some(Vec2::splat(72.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 2.0),
    ));
    kometen_welle_spawnen(&mut commands, &assets, 5);
}

fn kometen_welle_spawnen(commands: &mut Commands, assets: &SpielAssets, anzahl: usize) {
    let mut rng = rand::thread_rng();
    for _ in 0..anzahl {
        let rand_seite = rng.gen_range(0..4);
        let pos = match rand_seite {
            0 => Vec2::new(-HALBE_BREITE + 20.0, rng.gen_range(-HALBE_HOEHE..HALBE_HOEHE)),
            1 => Vec2::new(HALBE_BREITE - 20.0, rng.gen_range(-HALBE_HOEHE..HALBE_HOEHE)),
            2 => Vec2::new(rng.gen_range(-HALBE_BREITE..HALBE_BREITE), HALBE_HOEHE - 20.0),
            _ => Vec2::new(rng.gen_range(-HALBE_BREITE..HALBE_BREITE), -HALBE_HOEHE + 20.0),
        };
        let winkel = rng.gen_range(0.0..std::f32::consts::TAU);
        commands.spawn((
            Komet {
                groesse: 88.0,
                klein: false,
                drehung: rng.gen_range(-1.8..1.8),
            },
            SpielObjekt,
            Geschwindigkeit(Vec2::from_angle(winkel) * rng.gen_range(75.0..145.0)),
            Sprite {
                image: assets.komet_gross.clone(),
                custom_size: Some(Vec2::splat(88.0)),
                ..default()
            },
            Transform::from_translation(pos.extend(1.0)),
        ));
    }
}

fn welt_aufraeumen(mut commands: Commands, abfrage: Query<Entity, With<SpielObjekt>>) {
    for ent in &abfrage {
        commands.entity(ent).despawn_recursive();
    }
}

fn spieler_steuern(
    mut commands: Commands,
    tastatur: Res<ButtonInput<KeyCode>>,
    zeit: Res<Time>,
    assets: Res<SpielAssets>,
    mut abfrage: Query<(&mut Transform, &mut Geschwindigkeit, &mut Spieler)>,
) {
    let Ok((mut transform, mut tempo, mut spieler)) = abfrage.get_single_mut() else {
        return;
    };
    spieler.unverwundbar.tick(zeit.delta());
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
                lebensdauer: Timer::from_seconds(0.2, TimerMode::Once),
                max_zeit: 0.2,
            },
            SpielObjekt,
            Geschwindigkeit(-blick * 110.0),
            Sprite {
                image: assets.funke.clone(),
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
    assets: Res<SpielAssets>,
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
        SpielObjekt,
        Geschwindigkeit(blick * 840.0),
        Sprite {
            image: assets.laser.clone(),
            custom_size: Some(Vec2::new(18.0, 48.0)),
            ..default()
        },
        Transform {
            translation: transform.translation + (blick * 36.0).extend(0.0),
            rotation: transform.rotation,
            ..default()
        },
    ));
    commands.spawn((AudioPlayer::new(assets.sound_laser.clone()), PlaybackSettings::DESPAWN));
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
    assets: Res<SpielAssets>,
    mut daten: ResMut<SpielDaten>,
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
                commands.spawn((
                    AudioPlayer::new(assets.sound_explosion.clone()),
                    PlaybackSettings::DESPAWN,
                ));
                daten.punkte += if komet.klein { 150 } else { 100 };
                shake.0 = if komet.klein { 5.0 } else { 9.0 };

                for _ in 0..14 {
                    let winkel = rng.gen_range(0.0..std::f32::consts::TAU);
                    commands.spawn((
                        Partikel {
                            lebensdauer: Timer::from_seconds(0.42, TimerMode::Once),
                            max_zeit: 0.42,
                        },
                        SpielObjekt,
                        Geschwindigkeit(Vec2::from_angle(winkel) * rng.gen_range(80.0..260.0)),
                        Sprite {
                            image: assets.funke.clone(),
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
                            SpielObjekt,
                            Geschwindigkeit(Vec2::from_angle(winkel) * rng.gen_range(120.0..210.0)),
                            Sprite {
                                image: assets.komet_klein.clone(),
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

fn kollision_spieler_komet(
    mut commands: Commands,
    assets: Res<SpielAssets>,
    mut daten: ResMut<SpielDaten>,
    mut shake: ResMut<ScreenShake>,
    mut zustand: ResMut<NextState<SpielZustand>>,
    mut spieler_q: Query<(&mut Transform, &mut Geschwindigkeit, &mut Spieler)>,
    komet_q: Query<(&Transform, &Komet), Without<Spieler>>,
) {
    let Ok((mut s_tf, mut s_tempo, mut spieler)) = spieler_q.get_single_mut() else {
        return;
    };
    if !spieler.unverwundbar.finished() {
        return;
    }
    for (k_tf, komet) in &komet_q {
        let abstand = s_tf.translation.truncate().distance(k_tf.translation.truncate());
        if abstand < (komet.groesse * 0.4 + 22.0) {
            commands.spawn((
                AudioPlayer::new(assets.sound_treffer.clone()),
                PlaybackSettings::DESPAWN,
            ));
            shake.0 = 12.0;
            if daten.leben > 1 {
                daten.leben -= 1;
                s_tf.translation = Vec3::new(0.0, 0.0, 2.0);
                s_tempo.0 = Vec2::ZERO;
                spieler.unverwundbar.reset();
            } else {
                daten.leben = 0;
                zustand.set(SpielZustand::GameOver);
            }
            break;
        }
    }
}

fn welle_pruefen(
    mut commands: Commands,
    assets: Res<SpielAssets>,
    mut daten: ResMut<SpielDaten>,
    komet_q: Query<(), With<Komet>>,
) {
    if komet_q.is_empty() {
        daten.welle += 1;
        kometen_welle_spawnen(&mut commands, &assets, 4 + daten.welle as usize);
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

fn hud_aktualisieren(daten: Res<SpielDaten>, mut abfrage: Query<&mut Text, With<HudText>>) {
    if !daten.is_changed() {
        return;
    }
    if let Ok(mut text) = abfrage.get_single_mut() {
        text.0 = format!(
            "Punkte: {}   |   Leben: {}   |   Welle: {}",
            daten.punkte, daten.leben, daten.welle
        );
    }
}
