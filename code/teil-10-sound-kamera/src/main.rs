use bevy::prelude::*;

const KACHEL_BREITE: f32 = 120.0;
const KACHEL_HOEHE: f32 = 60.0;
const SPIELER_TEMPO: f32 = 3.6;

/// 0 = Boden, 1 = Säule, 2 = Schlüssel, 3 = Ausgangstor (nach Osten),
/// 4 = Runen-Inschrift, 5 = Boden-Schalter, 6 = Heiltrank
const RAEUME: [(&str, [[u8; 8]; 8]); 3] = [
    (
        "R-01: Eingangshalle",
        [
            [1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 4, 0, 0, 0, 1],
            [1, 0, 1, 0, 0, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 3],
            [1, 0, 0, 0, 1, 0, 0, 1],
            [1, 0, 1, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1],
        ],
    ),
    (
        "R-02: Saeulen-Gewoelbe",
        [
            [1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 2, 0, 1],
            [1, 0, 1, 0, 1, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 3],
            [1, 0, 1, 0, 1, 0, 0, 1],
            [1, 0, 5, 4, 0, 0, 0, 1],
            [1, 6, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1],
        ],
    ),
    (
        "R-03: Schatzkammer",
        [
            [1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 2, 1],
            [1, 0, 1, 0, 0, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 3],
            [1, 0, 0, 0, 1, 0, 0, 1],
            [1, 0, 1, 0, 0, 5, 0, 1],
            [1, 0, 0, 0, 0, 0, 6, 1],
            [1, 1, 1, 1, 1, 1, 1, 1],
        ],
    ),
];

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum KerkerZustand {
    #[default]
    Laeuft,
    Gewonnen,
    GameOver,
}

#[derive(Component)]
struct RaumObjekt;

#[derive(Component)]
struct HauptKamera;

#[derive(Component)]
struct Held {
    gitter_pos: Vec2,
    unverwundbar: f32,
    schwert_cd: f32,
}

#[derive(Component)]
struct Waechter {
    gitter_pos: Vec2,
    wegpunkte: Vec<Vec2>,
    aktueller_index: usize,
    leben: u32,
    tempo: f32,
}

#[derive(Component)]
struct Schluessel {
    gitter_pos: Vec2,
}

#[derive(Component)]
struct RunenInschrift {
    gitter_pos: Vec2,
    text: String,
}

#[derive(Component)]
struct BodenSchalter {
    gitter_pos: Vec2,
    gedrueckt: bool,
}

#[derive(Component)]
struct Heiltrank {
    gitter_pos: Vec2,
}

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct DialogPanel;

#[derive(Component)]
struct DialogTextFeld;

#[derive(Component)]
struct StatusOverlay;

#[derive(Resource, Default)]
struct KameraShake {
    staerke: f32,
    restzeit: f32,
}

#[derive(Resource)]
struct KerkerStatus {
    raum_index: usize,
    leben: u32,
    schluessel: u32,
    ziel_schluessel: u32,
    schalter_aktiv: bool,
    schritt_timer: f32,
    dialog_text: Option<String>,
}

impl Default for KerkerStatus {
    fn default() -> Self {
        Self {
            raum_index: 0,
            leben: 3,
            schluessel: 0,
            ziel_schluessel: 2,
            schalter_aktiv: true,
            schritt_timer: 0.0,
            dialog_text: Some(
                "Werkstatt-Chronik: Finde die 2 goldenen Runenschluessel und entkomme durch Raum 3! ([E] lesen, [Space] Schwert)"
                    .into(),
            ),
        }
    }
}

fn grid_to_iso(gitter: Vec2) -> Vec2 {
    Vec2::new(
        (gitter.x - gitter.y) * (KACHEL_BREITE * 0.5),
        -(gitter.x + gitter.y) * (KACHEL_HOEHE * 0.5),
    )
}

fn ist_begehbar(raum: usize, gitter: Vec2) -> bool {
    let sx = gitter.x.round() as isize;
    let sy = gitter.y.round() as isize;
    if sx < 0 || sy < 0 || sx >= 8 || sy >= 8 {
        return false;
    }
    let feld = RAEUME[raum].1[sy as usize][sx as usize];
    feld != 1 && feld != 4
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.10)))
        .init_resource::<KerkerStatus>()
        .init_resource::<KameraShake>()
        .init_state::<KerkerZustand>()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Kerker-Flucht: Vollstaendiges Isometrie-Abenteuer".into(),
                    resolution: (1280.0_f32, 720.0_f32).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_systems(Startup, setup_basis)
        .add_systems(
            Update,
            (
                held_steuern,
                schwert_und_interaktion,
                waechter_patrouille,
                schalter_und_trank,
                schluessel_und_tor,
                kamera_folgen,
                hud_und_dialog_aktualisieren,
            )
                .run_if(in_state(KerkerZustand::Laeuft)),
        )
        .add_systems(OnEnter(KerkerZustand::Gewonnen), sieg_overlay)
        .add_systems(OnEnter(KerkerZustand::GameOver), gameover_overlay)
        .run();
}

fn setup_basis(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, HauptKamera, Transform::from_xyz(0.0, -210.0, 999.0)));

    let herz_tex: Handle<Image> = asset_server.load("sprites/kerker/herz.png");
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(20.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(herz_tex),
                Node {
                    width: Val::Px(28.0),
                    height: Val::Px(28.0),
                    ..default()
                },
            ));
            parent.spawn((
                HudText,
                Text::new(""),
                TextFont {
                    font_size: 23.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.92, 0.84)),
            ));
        });

    commands
        .spawn((
            DialogPanel,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(10.0),
                width: Val::Percent(80.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(2.5)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.90)),
            BorderColor(Color::srgb(0.85, 0.68, 0.30)),
        ))
        .with_children(|parent| {
            parent.spawn((
                DialogTextFeld,
                Text::new(""),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.93, 0.85)),
            ));
        });

    raum_aufbauen(&mut commands, &asset_server, 0);
}

fn raum_aufbauen(commands: &mut Commands, asset_server: &AssetServer, raum_index: usize) {
    let boden_tex: Handle<Image> = asset_server.load("sprites/kerker/boden_kachel.png");
    let saeule_tex: Handle<Image> = asset_server.load("sprites/kerker/saeule.png");
    let held_tex: Handle<Image> = asset_server.load("sprites/kerker/held.png");
    let skelett_tex: Handle<Image> = asset_server.load("sprites/kerker/skelett.png");
    let schluessel_tex: Handle<Image> = asset_server.load("sprites/kerker/schluessel.png");

    let karte = RAEUME[raum_index].1;
    for (zeile, reihe) in karte.iter().enumerate() {
        for (spalte, feld) in reihe.iter().enumerate() {
            let gitter = Vec2::new(spalte as f32, zeile as f32);
            let iso = grid_to_iso(gitter);

            let boden_farbe = if *feld == 3 {
                Color::srgb(1.2, 1.05, 0.6)
            } else {
                Color::WHITE
            };
            commands.spawn((
                RaumObjekt,
                Sprite {
                    image: boden_tex.clone(),
                    color: boden_farbe,
                    custom_size: Some(Vec2::new(128.0, 96.0)),
                    ..default()
                },
                Transform::from_xyz(iso.x, iso.y, 0.0),
            ));

            if *feld == 1 || *feld == 4 {
                commands.spawn((
                    RaumObjekt,
                    Sprite {
                        image: saeule_tex.clone(),
                        color: if *feld == 4 {
                            Color::srgb(0.85, 1.0, 1.15)
                        } else {
                            Color::WHITE
                        },
                        custom_size: Some(Vec2::new(112.0, 196.0)),
                        ..default()
                    },
                    Transform::from_xyz(iso.x, iso.y + 68.0, 10.0 - iso.y * 0.001),
                ));
                if *feld == 4 {
                    let lore = if raum_index == 0 {
                        "Inschrift: \"Die goldene Tuer im Osten fuehrt tiefer ins Gewölbe.\""
                    } else {
                        "Inschrift: \"Nur wer zwei Runenschluessel traegt, verlaesst die Schatzkammer.\""
                    };
                    commands.spawn((
                        RaumObjekt,
                        RunenInschrift {
                            gitter_pos: gitter,
                            text: lore.into(),
                        },
                    ));
                }
            } else if *feld == 2 {
                commands.spawn((
                    RaumObjekt,
                    Schluessel { gitter_pos: gitter },
                    Sprite {
                        image: schluessel_tex.clone(),
                        custom_size: Some(Vec2::new(48.0, 48.0)),
                        ..default()
                    },
                    Transform::from_xyz(iso.x, iso.y + 18.0, 10.0 - iso.y * 0.001),
                ));
            } else if *feld == 5 {
                commands.spawn((
                    RaumObjekt,
                    BodenSchalter {
                        gitter_pos: gitter,
                        gedrueckt: false,
                    },
                    Sprite {
                        color: Color::srgb(0.25, 0.75, 0.95),
                        custom_size: Some(Vec2::new(44.0, 22.0)),
                        ..default()
                    },
                    Transform::from_xyz(iso.x, iso.y + 4.0, 2.0 - iso.y * 0.001),
                ));
            } else if *feld == 6 {
                let herz_tex: Handle<Image> = asset_server.load("sprites/kerker/herz.png");
                commands.spawn((
                    RaumObjekt,
                    Heiltrank { gitter_pos: gitter },
                    Sprite {
                        image: herz_tex,
                        color: Color::srgb(0.4, 1.0, 0.65),
                        custom_size: Some(Vec2::new(34.0, 34.0)),
                        ..default()
                    },
                    Transform::from_xyz(iso.x, iso.y + 16.0, 10.0 - iso.y * 0.001),
                ));
            }
        }
    }

    let start = Vec2::new(1.2, 3.0);
    let start_iso = grid_to_iso(start);
    commands.spawn((
        RaumObjekt,
        Held {
            gitter_pos: start,
            unverwundbar: 0.0,
            schwert_cd: 0.0,
        },
        Sprite {
            image: held_tex,
            custom_size: Some(Vec2::new(72.0, 72.0)),
            ..default()
        },
        Transform::from_xyz(start_iso.x, start_iso.y + 24.0, 10.0 - start_iso.y * 0.001),
    ));

    if raum_index >= 1 {
        let wp = vec![Vec2::new(3.0, 3.0), Vec2::new(5.5, 3.0), Vec2::new(5.5, 5.0)];
        let w_iso = grid_to_iso(wp[0]);
        commands.spawn((
            RaumObjekt,
            Waechter {
                gitter_pos: wp[0],
                wegpunkte: wp,
                aktueller_index: 1,
                leben: 2,
                tempo: 1.5,
            },
            Sprite {
                image: skelett_tex,
                custom_size: Some(Vec2::new(72.0, 84.0)),
                ..default()
            },
            Transform::from_xyz(w_iso.x, w_iso.y + 28.0, 10.0 - w_iso.y * 0.001),
        ));
    }
}

fn held_steuern(
    mut commands: Commands,
    zeit: Res<Time>,
    tasten: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut status: ResMut<KerkerStatus>,
    mut abfrage: Query<(&mut Held, &mut Transform)>,
) {
    let dt = zeit.delta_secs();
    status.schritt_timer = (status.schritt_timer - dt).max(0.0);

    let mut richtung = Vec2::ZERO;
    if tasten.pressed(KeyCode::KeyW) || tasten.pressed(KeyCode::ArrowUp) {
        richtung.y -= 1.0;
    }
    if tasten.pressed(KeyCode::KeyS) || tasten.pressed(KeyCode::ArrowDown) {
        richtung.y += 1.0;
    }
    if tasten.pressed(KeyCode::KeyA) || tasten.pressed(KeyCode::ArrowLeft) {
        richtung.x -= 1.0;
    }
    if tasten.pressed(KeyCode::KeyD) || tasten.pressed(KeyCode::ArrowRight) {
        richtung.x += 1.0;
    }

    for (mut held, mut transform) in &mut abfrage {
        held.unverwundbar = (held.unverwundbar - dt).max(0.0);
        held.schwert_cd = (held.schwert_cd - dt).max(0.0);
        if richtung == Vec2::ZERO {
            continue;
        }
        let schritt = richtung.normalize() * SPIELER_TEMPO * dt;
        let kx = Vec2::new(held.gitter_pos.x + schritt.x, held.gitter_pos.y);
        if ist_begehbar(status.raum_index, kx) {
            held.gitter_pos.x = kx.x;
        }
        let ky = Vec2::new(held.gitter_pos.x, held.gitter_pos.y + schritt.y);
        if ist_begehbar(status.raum_index, ky) {
            held.gitter_pos.y = ky.y;
        }
        let iso = grid_to_iso(held.gitter_pos);
        transform.translation.x = iso.x;
        transform.translation.y = iso.y + 24.0;
        transform.translation.z = 10.0 - iso.y * 0.001;

        if status.schritt_timer <= 0.0 {
            status.schritt_timer = 0.28;
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/schritt.wav")),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

fn schwert_und_interaktion(
    mut commands: Commands,
    tasten: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut status: ResMut<KerkerStatus>,
    mut held_q: Query<&mut Held>,
    mut waechter_q: Query<(Entity, &mut Waechter, &mut Transform)>,
    inschrift_q: Query<&RunenInschrift>,
) {
    let Ok(mut held) = held_q.get_single_mut() else {
        return;
    };

    if tasten.just_pressed(KeyCode::KeyE) {
        if status.dialog_text.is_some() {
            status.dialog_text = None;
        } else {
            for inschrift in &inschrift_q {
                if held.gitter_pos.distance(inschrift.gitter_pos) < 1.6 {
                    status.dialog_text = Some(inschrift.text.clone());
                }
            }
        }
    }

    if tasten.just_pressed(KeyCode::Space) && held.schwert_cd <= 0.0 {
        held.schwert_cd = 0.4;
        for (entity, mut w, mut wt) in &mut waechter_q {
            if held.gitter_pos.distance(w.gitter_pos) < 1.35 {
                w.leben = w.leben.saturating_sub(1);
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/treffer_kerker.wav")),
                    PlaybackSettings::DESPAWN,
                ));
                let diff = (w.gitter_pos - held.gitter_pos).normalize_or_zero() * 0.7;
                w.gitter_pos = (w.gitter_pos + diff).clamp(Vec2::splat(1.0), Vec2::splat(6.0));
                let iso = grid_to_iso(w.gitter_pos);
                wt.translation.x = iso.x;
                wt.translation.y = iso.y + 28.0;
                if w.leben == 0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn waechter_patrouille(
    mut commands: Commands,
    zeit: Res<Time>,
    asset_server: Res<AssetServer>,
    mut status: ResMut<KerkerStatus>,
    mut shake: ResMut<KameraShake>,
    mut naechster: ResMut<NextState<KerkerZustand>>,
    mut held_q: Query<&mut Held>,
    mut waechter_q: Query<(&mut Waechter, &mut Transform)>,
) {
    let dt = zeit.delta_secs();
    let Ok(mut held) = held_q.get_single_mut() else {
        return;
    };

    for (mut w, mut wt) in &mut waechter_q {
        let abstand = w.gitter_pos.distance(held.gitter_pos);
        let ziel = if abstand < 3.0 {
            held.gitter_pos
        } else {
            let wp = w.wegpunkte[w.aktueller_index];
            if w.gitter_pos.distance(wp) < 0.2 {
                w.aktueller_index = (w.aktueller_index + 1) % w.wegpunkte.len();
            }
            w.wegpunkte[w.aktueller_index]
        };

        let diff = ziel - w.gitter_pos;
        if diff.length() > 0.05 {
            let schritt = diff.normalize() * w.tempo * dt;
            w.gitter_pos += schritt;
        }
        let iso = grid_to_iso(w.gitter_pos);
        wt.translation.x = iso.x;
        wt.translation.y = iso.y + 28.0;
        wt.translation.z = 10.0 - iso.y * 0.001;

        if abstand < 0.6 && held.unverwundbar <= 0.0 {
            held.unverwundbar = 1.2;
            status.leben = status.leben.saturating_sub(1);
            shake.staerke = 10.0;
            shake.restzeit = 0.22;
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/treffer_kerker.wav")),
                PlaybackSettings::DESPAWN,
            ));
            if status.leben == 0 {
                naechster.set(KerkerZustand::GameOver);
            }
        }
    }
}

fn schluessel_und_tor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut status: ResMut<KerkerStatus>,
    mut naechster: ResMut<NextState<KerkerZustand>>,
    held_q: Query<&Held>,
    schluessel_q: Query<(Entity, &Schluessel)>,
    raum_objekte: Query<Entity, With<RaumObjekt>>,
) {
    let Ok(held) = held_q.get_single() else {
        return;
    };
    for (entity, schluessel) in &schluessel_q {
        if held.gitter_pos.distance(schluessel.gitter_pos) < 0.65 {
            status.schluessel += 1;
            commands.entity(entity).despawn();
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/schluessel.wav")),
                PlaybackSettings::DESPAWN,
            ));
            status.dialog_text = Some(format!(
                "Goldener Runenschluessel gefunden! ({}/{})",
                status.schluessel, status.ziel_schluessel
            ));
        }
    }

    // Tor bei (7, 3) betreten
    if held.gitter_pos.distance(Vec2::new(7.0, 3.0)) < 0.65 {
        if !status.schalter_aktiv {
            status.dialog_text = Some(
                "Das Tor ist durch einen Runen-Mechanismus blockiert! Tritt auf den blauen Boden-Schalter."
                    .into(),
            );
            return;
        }
        if status.raum_index + 1 < RAEUME.len() {
            status.raum_index += 1;
            let hat_schalter = RAEUME[status.raum_index]
                .1
                .iter()
                .any(|reihe| reihe.contains(&5));
            status.schalter_aktiv = !hat_schalter;
            for e in &raum_objekte {
                commands.entity(e).despawn();
            }
            raum_aufbauen(&mut commands, &asset_server, status.raum_index);
        } else if status.schluessel >= status.ziel_schluessel {
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/sieg.wav")),
                PlaybackSettings::DESPAWN,
            ));
            naechster.set(KerkerZustand::Gewonnen);
        } else {
            status.dialog_text = Some("Das Ausgangstor ist verschlossen! Dir fehlen noch Runenschluessel.".into());
        }
    }
}

fn pruefe_schalter(held_pos: Vec2, schalter_pos: Vec2, gedrueckt: &mut bool) -> bool {
    if !*gedrueckt && held_pos.distance(schalter_pos) < 0.6 {
        *gedrueckt = true;
        return true;
    }
    false
}

fn trank_einsammeln(leben: &mut u32, max_leben: u32) -> bool {
    if *leben < max_leben {
        *leben += 1;
        return true;
    }
    false
}

fn schalter_und_trank(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut status: ResMut<KerkerStatus>,
    held_q: Query<&Held>,
    mut schalter_q: Query<(&mut BodenSchalter, &mut Sprite)>,
    trank_q: Query<(Entity, &Heiltrank)>,
) {
    let Ok(held) = held_q.get_single() else {
        return;
    };

    for (mut schalter, mut sprite) in &mut schalter_q {
        if pruefe_schalter(held.gitter_pos, schalter.gitter_pos, &mut schalter.gedrueckt) {
            status.schalter_aktiv = true;
            sprite.color = Color::srgb(0.25, 0.95, 0.45);
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/schluessel.wav")),
                PlaybackSettings::DESPAWN,
            ));
            status.dialog_text = Some(
                "Klick! Der Boden-Schalter rastet ein — das Ausgangstor im Osten ist entriegelt!"
                    .into(),
            );
        }
    }

    for (entity, trank) in &trank_q {
        if held.gitter_pos.distance(trank.gitter_pos) < 0.6 {
            if trank_einsammeln(&mut status.leben, 5) {
                commands.entity(entity).despawn();
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/schluessel.wav")),
                    PlaybackSettings::DESPAWN,
                ));
                status.dialog_text = Some(format!(
                    "Heiltrank getrunken! Leben wiederhergestellt ({}/5).",
                    status.leben
                ));
            }
        }
    }
}

fn berechne_shake_versatz(shake: &mut KameraShake, dt: f32, gesamtzeit: f32) -> Vec2 {
    if shake.restzeit <= 0.0 {
        shake.staerke = 0.0;
        return Vec2::ZERO;
    }
    shake.restzeit = (shake.restzeit - dt).max(0.0);
    let aktuell = shake.staerke * (shake.restzeit / 0.22).min(1.0);
    Vec2::new(
        (gesamtzeit * 55.0).sin() * aktuell,
        (gesamtzeit * 73.0).cos() * aktuell * 0.7,
    )
}

fn kamera_folgen(
    zeit: Res<Time>,
    mut shake: ResMut<KameraShake>,
    held_q: Query<&Transform, (With<Held>, Without<HauptKamera>)>,
    mut kam_q: Query<&mut Transform, With<HauptKamera>>,
) {
    let (Ok(ht), Ok(mut kt)) = (held_q.get_single(), kam_q.get_single_mut()) else {
        return;
    };
    let dt = zeit.delta_secs();
    let faktor = (4.5 * dt).min(1.0);
    let versatz = berechne_shake_versatz(&mut shake, dt, zeit.elapsed_secs());
    kt.translation.x += (ht.translation.x + versatz.x - kt.translation.x) * faktor;
    kt.translation.y += (ht.translation.y + versatz.y - kt.translation.y) * faktor;
}

fn hud_und_dialog_aktualisieren(
    status: Res<KerkerStatus>,
    mut hud_q: Query<&mut Text, (With<HudText>, Without<DialogTextFeld>)>,
    mut panel_q: Query<&mut Visibility, With<DialogPanel>>,
    mut dtext_q: Query<&mut Text, (With<DialogTextFeld>, Without<HudText>)>,
) {
    if let Ok(mut text) = hud_q.get_single_mut() {
        **text = format!(
            "{}  |  Schluessel: {}/{}  |  {}",
            status.leben,
            status.schluessel,
            status.ziel_schluessel,
            RAEUME[status.raum_index].0
        );
    }
    if let (Ok(mut vis), Ok(mut dtext)) = (panel_q.get_single_mut(), dtext_q.get_single_mut()) {
        if let Some(ref nachricht) = status.dialog_text {
            *vis = Visibility::Visible;
            **dtext = format!("{}  ([E] schliessen)", nachricht);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn sieg_overlay(mut commands: Commands) {
    commands.spawn((
        StatusOverlay,
        Text::new("KERKER-FLUCHT GEMEISTERT!\nDu hast das Zwergengewoelbe mit allen Runenschluesseln verlassen."),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.85, 0.25)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(14.0),
            ..default()
        },
    ));
}

fn gameover_overlay(mut commands: Commands) {
    commands.spawn((
        StatusOverlay,
        Text::new("GAME OVER\nDer Skelett-Waechter war schneller."),
        TextFont {
            font_size: 38.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.30, 0.30)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(22.0),
            ..default()
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schalter_aktivieren() {
        let mut gedrueckt = false;
        assert!(!pruefe_schalter(Vec2::new(1.0, 1.0), Vec2::new(3.0, 5.0), &mut gedrueckt));
        assert!(!gedrueckt);

        assert!(pruefe_schalter(Vec2::new(3.1, 5.0), Vec2::new(3.0, 5.0), &mut gedrueckt));
        assert!(gedrueckt);

        // Ein zweites Mal darf der Schalter nicht erneut auslösen
        assert!(!pruefe_schalter(Vec2::new(3.0, 5.0), Vec2::new(3.0, 5.0), &mut gedrueckt));
    }

    #[test]
    fn trank_heilen() {
        let mut leben = 3;
        assert!(trank_einsammeln(&mut leben, 5));
        assert_eq!(leben, 4);
        assert!(trank_einsammeln(&mut leben, 5));
        assert_eq!(leben, 5);
        // Bei vollem Leben wird kein weiterer Trank verbraucht
        assert!(!trank_einsammeln(&mut leben, 5));
        assert_eq!(leben, 5);
    }

    #[test]
    fn kamera_shake_klingt_ab() {
        let mut shake = KameraShake {
            staerke: 10.0,
            restzeit: 0.22,
        };
        let v1 = berechne_shake_versatz(&mut shake, 0.05, 1.0);
        assert!(v1.length() > 0.0);

        let _ = berechne_shake_versatz(&mut shake, 0.25, 1.25);
        let v_ende = berechne_shake_versatz(&mut shake, 0.016, 1.3);
        assert_eq!(v_ende, Vec2::ZERO);
        assert_eq!(shake.staerke, 0.0);
    }
}
