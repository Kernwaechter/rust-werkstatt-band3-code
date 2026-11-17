use bevy::prelude::*;

const KACHEL_BREITE: f32 = 120.0;
const KACHEL_HOEHE: f32 = 60.0;
const SPIELER_TEMPO: f32 = 3.5;

#[derive(Component)]
struct Held {
    gitter_pos: Vec2,
    leben: u32,
    schwert_cooldown: f32,
    unverwundbar: f32,
}

#[derive(Component)]
struct Skelett {
    gitter_pos: Vec2,
    wegpunkte: Vec<Vec2>,
    aktueller_index: usize,
    leben: u32,
    tempo: f32,
}

pub fn grid_to_iso(gitter: Vec2) -> Vec2 {
    Vec2::new(
        (gitter.x - gitter.y) * (KACHEL_BREITE * 0.5),
        -(gitter.x + gitter.y) * (KACHEL_HOEHE * 0.5),
    )
}

/// Berechnet den Rückstoßvektor vom Angreifer weg im Gitterraum.
pub fn berechne_rueckstoss(angreifer: Vec2, ziel: Vec2, staerke: f32) -> Vec2 {
    let diff = ziel - angreifer;
    if diff.length_squared() < 0.0001 {
        Vec2::X * staerke
    } else {
        diff.normalize() * staerke
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.10)))
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Kerker-Flucht: Gegner-KI & Schwertkampf".into(),
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
            (held_steuern, schwert_angriff, skelett_ki_und_treffer),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, -210.0, 999.0)));

    let boden_tex: Handle<Image> = asset_server.load("sprites/kerker/boden_kachel.png");
    let held_tex: Handle<Image> = asset_server.load("sprites/kerker/held.png");
    let skelett_tex: Handle<Image> = asset_server.load("sprites/kerker/skelett.png");

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

    let start = Vec2::new(2.0, 5.0);
    let start_iso = grid_to_iso(start);
    commands.spawn((
        Held {
            gitter_pos: start,
            leben: 3,
            schwert_cooldown: 0.0,
            unverwundbar: 0.0,
        },
        Sprite {
            image: held_tex,
            custom_size: Some(Vec2::new(72.0, 72.0)),
            ..default()
        },
        Transform::from_xyz(start_iso.x, start_iso.y + 24.0, 10.0 - start_iso.y * 0.001),
    ));

    let s_pos = Vec2::new(4.0, 3.0);
    let s_iso = grid_to_iso(s_pos);
    commands.spawn((
        Skelett {
            gitter_pos: s_pos,
            wegpunkte: vec![Vec2::new(2.0, 3.0), Vec2::new(5.5, 3.0)],
            aktueller_index: 1,
            leben: 2,
            tempo: 1.5,
        },
        Sprite {
            image: skelett_tex,
            custom_size: Some(Vec2::new(72.0, 84.0)),
            ..default()
        },
        Transform::from_xyz(s_iso.x, s_iso.y + 28.0, 10.0 - s_iso.y * 0.001),
    ));
}

fn held_steuern(
    zeit: Res<Time>,
    tasten: Res<ButtonInput<KeyCode>>,
    mut abfrage: Query<(&mut Held, &mut Transform)>,
) {
    let dt = zeit.delta_secs();
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

    for (mut held, mut t) in &mut abfrage {
        held.schwert_cooldown = (held.schwert_cooldown - dt).max(0.0);
        held.unverwundbar = (held.unverwundbar - dt).max(0.0);
        if richtung != Vec2::ZERO {
            let schritt = richtung.normalize() * SPIELER_TEMPO * dt;
            held.gitter_pos = (held.gitter_pos + schritt).clamp(Vec2::splat(0.4), Vec2::splat(6.6));
            let iso = grid_to_iso(held.gitter_pos);
            t.translation.x = iso.x;
            t.translation.y = iso.y + 24.0;
            t.translation.z = 10.0 - iso.y * 0.001;
        }
    }
}

fn schwert_angriff(
    mut commands: Commands,
    tasten: Res<ButtonInput<KeyCode>>,
    mut held_q: Query<&mut Held>,
    mut skelett_q: Query<(Entity, &mut Skelett, &mut Transform)>,
) {
    let Ok(mut held) = held_q.get_single_mut() else {
        return;
    };
    if !tasten.just_pressed(KeyCode::Space) || held.schwert_cooldown > 0.0 {
        return;
    }
    held.schwert_cooldown = 0.4;

    for (entity, mut sk, mut st) in &mut skelett_q {
        if held.gitter_pos.distance(sk.gitter_pos) < 1.35 {
            sk.leben = sk.leben.saturating_sub(1);
            let stoss = berechne_rueckstoss(held.gitter_pos, sk.gitter_pos, 0.8);
            sk.gitter_pos = (sk.gitter_pos + stoss).clamp(Vec2::splat(0.5), Vec2::splat(6.5));
            let iso = grid_to_iso(sk.gitter_pos);
            st.translation.x = iso.x;
            st.translation.y = iso.y + 28.0;
            if sk.leben == 0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn skelett_ki_und_treffer(
    zeit: Res<Time>,
    mut held_q: Query<&mut Held>,
    mut skelett_q: Query<(&mut Skelett, &mut Transform)>,
) {
    let dt = zeit.delta_secs();
    let Ok(mut held) = held_q.get_single_mut() else {
        return;
    };

    for (mut sk, mut st) in &mut skelett_q {
        // Verfolgt den Helden im Sichtradius von 3.2 Feldern, sonst Patrouille
        let abstand = sk.gitter_pos.distance(held.gitter_pos);
        let ziel = if abstand < 3.2 {
            held.gitter_pos
        } else {
            let wp = sk.wegpunkte[sk.aktueller_index];
            if sk.gitter_pos.distance(wp) < 0.2 {
                sk.aktueller_index = (sk.aktueller_index + 1) % sk.wegpunkte.len();
            }
            sk.wegpunkte[sk.aktueller_index]
        };

        let diff = ziel - sk.gitter_pos;
        if diff.length() > 0.05 {
            let schritt = diff.normalize() * sk.tempo * dt;
            sk.gitter_pos += schritt;
        }
        let iso = grid_to_iso(sk.gitter_pos);
        st.translation.x = iso.x;
        st.translation.y = iso.y + 28.0;
        st.translation.z = 10.0 - iso.y * 0.001;

        if abstand < 0.65 && held.unverwundbar <= 0.0 {
            held.leben = held.leben.saturating_sub(1);
            held.unverwundbar = 1.2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rueckstoss_zeigt_vom_angreifer_weg() {
        let stoss = berechne_rueckstoss(Vec2::new(2.0, 2.0), Vec2::new(3.0, 2.0), 0.8);
        assert!(stoss.x > 0.79 && stoss.y.abs() < 0.001);
    }
}
