use bevy::prelude::*;

const KACHEL_BREITE: f32 = 120.0;
const KACHEL_HOEHE: f32 = 60.0;
const SPIELER_TEMPO: f32 = 3.5; // Gitter-Einheiten pro Sekunde

#[derive(Component)]
struct Held {
    gitter_pos: Vec2,
}

#[derive(Component)]
struct IsoTiefe {
    basis_z: f32,
}

/// Wandelt 2D-Gitterkoordinaten (spalte, zeile) in 2:1-isometrische Bildschirmkoordinaten um.
pub fn grid_to_iso(gitter: Vec2) -> Vec2 {
    let x = (gitter.x - gitter.y) * (KACHEL_BREITE * 0.5);
    let y = -(gitter.x + gitter.y) * (KACHEL_HOEHE * 0.5);
    Vec2::new(x, y)
}

/// Wandelt isometrische Bildschirmkoordinaten zurück in Gitterkoordinaten.
pub fn iso_to_grid(iso: Vec2) -> Vec2 {
    let halb_b = KACHEL_BREITE * 0.5;
    let halb_h = KACHEL_HOEHE * 0.5;
    let gx = (iso.x / halb_b - iso.y / halb_h) * 0.5;
    let gy = (-iso.y / halb_h - iso.x / halb_b) * 0.5;
    Vec2::new(gx, gy)
}

/// Berechnet den Z-Wert für Y-Sorting: Weiter unten im Bild = größeres Z (steht weiter vorne).
pub fn y_sort_z(welt_y: f32, basis_z: f32) -> f32 {
    basis_z - welt_y * 0.001
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.10)))
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Kerker-Flucht: Isometrische Welt & Y-Sorting".into(),
                    resolution: (1280.0_f32, 720.0_f32).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (held_steuern, y_sorting_aktualisieren))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, -200.0, 999.0)));

    let boden_tex: Handle<Image> = asset_server.load("sprites/kerker/boden_kachel.png");
    let saeule_tex: Handle<Image> = asset_server.load("sprites/kerker/saeule.png");
    let held_tex: Handle<Image> = asset_server.load("sprites/kerker/held.png");

    // 8x8 isometrischer Kerkerboden
    for zeile in 0..8 {
        for spalte in 0..8 {
            let iso = grid_to_iso(Vec2::new(spalte as f32, zeile as f32));
            commands.spawn((
                Sprite {
                    image: boden_tex.clone(),
                    custom_size: Some(Vec2::new(128.0, 96.0)),
                    ..default()
                },
                Transform::from_xyz(iso.x, iso.y, 0.0),
            ));
        }
    }

    // Zwei Steinsäulen mit Y-Sorting
    for (spalte, zeile) in [(2.0_f32, 3.0_f32), (5.0_f32, 4.0_f32)] {
        let iso = grid_to_iso(Vec2::new(spalte, zeile));
        commands.spawn((
            IsoTiefe { basis_z: 10.0 },
            Sprite {
                image: saeule_tex.clone(),
                custom_size: Some(Vec2::new(112.0, 196.0)),
                ..default()
            },
            Transform::from_xyz(iso.x, iso.y + 68.0, y_sort_z(iso.y, 10.0)),
        ));
    }

    // Held auf Startfeld (3, 4)
    let start = Vec2::new(3.0, 4.0);
    let start_iso = grid_to_iso(start);
    commands.spawn((
        Held { gitter_pos: start },
        IsoTiefe { basis_z: 10.0 },
        Sprite {
            image: held_tex,
            custom_size: Some(Vec2::new(72.0, 72.0)),
            ..default()
        },
        Transform::from_xyz(start_iso.x, start_iso.y + 24.0, y_sort_z(start_iso.y, 10.0)),
    ));
}

fn held_steuern(
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
        held.gitter_pos = (held.gitter_pos + schritt).clamp(Vec2::splat(0.3), Vec2::splat(6.7));
        let iso = grid_to_iso(held.gitter_pos);
        transform.translation.x = iso.x;
        transform.translation.y = iso.y + 24.0;
    }
}

fn y_sorting_aktualisieren(mut abfrage: Query<(&IsoTiefe, &mut Transform)>) {
    for (tiefe, mut transform) in &mut abfrage {
        transform.translation.z = y_sort_z(transform.translation.y - 24.0, tiefe.basis_z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_iso_hin_und_zurueck() {
        let ursprung = Vec2::new(3.5, 2.0);
        let iso = grid_to_iso(ursprung);
        let zurueck = iso_to_grid(iso);
        assert!((zurueck - ursprung).length() < 0.001);
    }

    #[test]
    fn vorderes_objekt_hat_groesseres_z() {
        let z_vorne = y_sort_z(-200.0, 10.0);
        let z_hinten = y_sort_z(-50.0, 10.0);
        assert!(z_vorne > z_hinten);
    }
}
