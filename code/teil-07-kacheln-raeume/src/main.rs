use bevy::prelude::*;

const KACHEL_BREITE: f32 = 120.0;
const KACHEL_HOEHE: f32 = 60.0;
const SPIELER_TEMPO: f32 = 3.4;

/// 0 = Boden, 1 = Säule (blockiert), 2 = Goldener Schlüssel
const KARTE: [[u8; 8]; 8] = [
    [1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 2, 1],
    [1, 0, 1, 0, 0, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 1, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 1],
];

#[derive(Component)]
struct Held {
    gitter_pos: Vec2,
}

#[derive(Component)]
struct Waechter {
    wegpunkte: Vec<Vec2>,
    aktueller_index: usize,
    tempo: f32,
}

#[derive(Component)]
struct Schluessel {
    gitter_pos: Vec2,
}

#[derive(Resource, Default)]
struct Inventar {
    schluessel: u32,
}

pub fn grid_to_iso(gitter: Vec2) -> Vec2 {
    Vec2::new(
        (gitter.x - gitter.y) * (KACHEL_BREITE * 0.5),
        -(gitter.x + gitter.y) * (KACHEL_HOEHE * 0.5),
    )
}

pub fn ist_begehbar(karte: &[[u8; 8]; 8], gitter: Vec2) -> bool {
    let sx = gitter.x.round() as isize;
    let sy = gitter.y.round() as isize;
    if sx < 0 || sy < 0 || sx >= 8 || sy >= 8 {
        return false;
    }
    karte[sy as usize][sx as usize] != 1
}

pub fn naechster_wegpunkt(aktuell: usize, anzahl: usize) -> usize {
    if anzahl == 0 {
        0
    } else {
        (aktuell + 1) % anzahl
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.10)))
        .init_resource::<Inventar>()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Kerker-Flucht: Kacheln, Kollision & Skelett-KI".into(),
                    resolution: (1280.0_f32, 720.0_f32).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (held_mit_kollision, waechter_patrouille, schluessel_einsammeln),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, -210.0, 999.0)));

    let boden_tex: Handle<Image> = asset_server.load("sprites/kerker/boden_kachel.png");
    let saeule_tex: Handle<Image> = asset_server.load("sprites/kerker/saeule.png");
    let held_tex: Handle<Image> = asset_server.load("sprites/kerker/held.png");
    let skelett_tex: Handle<Image> = asset_server.load("sprites/kerker/skelett.png");
    let schluessel_tex: Handle<Image> = asset_server.load("sprites/kerker/schluessel.png");

    for (zeile, reihe) in KARTE.iter().enumerate() {
        for (spalte, feld) in reihe.iter().enumerate() {
            let gitter = Vec2::new(spalte as f32, zeile as f32);
            let iso = grid_to_iso(gitter);

            commands.spawn((
                Sprite {
                    image: boden_tex.clone(),
                    custom_size: Some(Vec2::new(128.0, 96.0)),
                    ..default()
                },
                Transform::from_xyz(iso.x, iso.y, 0.0),
            ));

            if *feld == 1 {
                commands.spawn((
                    Sprite {
                        image: saeule_tex.clone(),
                        custom_size: Some(Vec2::new(112.0, 196.0)),
                        ..default()
                    },
                    Transform::from_xyz(iso.x, iso.y + 68.0, 10.0 - iso.y * 0.001),
                ));
            } else if *feld == 2 {
                commands.spawn((
                    Schluessel { gitter_pos: gitter },
                    Sprite {
                        image: schluessel_tex.clone(),
                        custom_size: Some(Vec2::new(48.0, 48.0)),
                        ..default()
                    },
                    Transform::from_xyz(iso.x, iso.y + 18.0, 10.0 - iso.y * 0.001),
                ));
            }
        }
    }

    let start = Vec2::new(1.0, 6.0);
    let start_iso = grid_to_iso(start);
    commands.spawn((
        Held { gitter_pos: start },
        Sprite {
            image: held_tex,
            custom_size: Some(Vec2::new(72.0, 72.0)),
            ..default()
        },
        Transform::from_xyz(start_iso.x, start_iso.y + 24.0, 10.0 - start_iso.y * 0.001),
    ));

    let waechter_start = Vec2::new(3.0, 3.0);
    let waechter_iso = grid_to_iso(waechter_start);
    commands.spawn((
        Waechter {
            wegpunkte: vec![
                Vec2::new(3.0, 3.0),
                Vec2::new(6.0, 3.0),
                Vec2::new(6.0, 5.0),
                Vec2::new(3.0, 5.0),
            ],
            aktueller_index: 1,
            tempo: 1.8,
        },
        Sprite {
            image: skelett_tex,
            custom_size: Some(Vec2::new(72.0, 84.0)),
            ..default()
        },
        Transform::from_xyz(
            waechter_iso.x,
            waechter_iso.y + 28.0,
            10.0 - waechter_iso.y * 0.001,
        ),
    ));
}

fn held_mit_kollision(
    zeit: Res<Time>,
    tasten: Res<ButtonInput<KeyCode>>,
    mut abfrage: Query<(&mut Held, &mut Transform)>,
) {
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
    if richtung == Vec2::ZERO {
        return;
    }

    let dt = zeit.delta_secs();
    let schritt = richtung.normalize() * SPIELER_TEMPO * dt;
    for (mut held, mut transform) in &mut abfrage {
        let kandidat_x = Vec2::new(held.gitter_pos.x + schritt.x, held.gitter_pos.y);
        if ist_begehbar(&KARTE, kandidat_x) {
            held.gitter_pos.x = kandidat_x.x;
        }
        let kandidat_y = Vec2::new(held.gitter_pos.x, held.gitter_pos.y + schritt.y);
        if ist_begehbar(&KARTE, kandidat_y) {
            held.gitter_pos.y = kandidat_y.y;
        }

        let iso = grid_to_iso(held.gitter_pos);
        transform.translation.x = iso.x;
        transform.translation.y = iso.y + 24.0;
        transform.translation.z = 10.0 - iso.y * 0.001;
    }
}

fn waechter_patrouille(
    zeit: Res<Time>,
    mut abfrage: Query<(&mut Waechter, &mut Transform)>,
) {
    let dt = zeit.delta_secs();
    for (mut waechter, mut transform) in &mut abfrage {
        let ziel = waechter.wegpunkte[waechter.aktueller_index];
        let pos_iso = Vec2::new(transform.translation.x, transform.translation.y - 28.0);
        let ziel_iso = grid_to_iso(ziel);
        let diff = ziel_iso - pos_iso;

        if diff.length() < 6.0 {
            let n = waechter.wegpunkte.len();
            waechter.aktueller_index = naechster_wegpunkt(waechter.aktueller_index, n);
        } else {
            let bewegung = diff.normalize() * (waechter.tempo * 70.0) * dt;
            transform.translation.x += bewegung.x;
            transform.translation.y += bewegung.y;
            let fuss_y = transform.translation.y - 28.0;
            transform.translation.z = 10.0 - fuss_y * 0.001;
        }
    }
}

fn schluessel_einsammeln(
    mut commands: Commands,
    mut inventar: ResMut<Inventar>,
    held_q: Query<&Held>,
    schluessel_q: Query<(Entity, &Schluessel)>,
) {
    let Ok(held) = held_q.get_single() else {
        return;
    };
    for (entity, schluessel) in &schluessel_q {
        if held.gitter_pos.distance(schluessel.gitter_pos) < 0.65 {
            inventar.schluessel += 1;
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wand_blockiert_bewegung() {
        assert!(!ist_begehbar(&KARTE, Vec2::new(0.0, 0.0)));
        assert!(!ist_begehbar(&KARTE, Vec2::new(2.0, 2.0)));
        assert!(ist_begehbar(&KARTE, Vec2::new(1.0, 1.0)));
    }

    #[test]
    fn wegpunkt_rotiert_zyklisch() {
        assert_eq!(naechster_wegpunkt(0, 4), 1);
        assert_eq!(naechster_wegpunkt(3, 4), 0);
    }
}
