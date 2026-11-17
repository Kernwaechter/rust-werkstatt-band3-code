use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const ARENA_HALB: f32 = 7.2;
const KUGEL_RADIUS: f32 = 0.55;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum ArenaZustand {
    #[default]
    StartMenue,
    Laeuft,
    Gewonnen,
}

#[derive(Component)]
struct Kugel {
    tempo: Vec3,
}

#[derive(Component)]
struct Kristall;

#[derive(Component)]
struct Hindernis {
    halb_groesse: Vec2,
}

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct OverlayText;

#[derive(Resource, Serialize, Deserialize, Clone)]
struct Spielstand {
    highscore: u32,
    bestzeit: f32,
}

impl Default for Spielstand {
    fn default() -> Self {
        Self {
            highscore: 0,
            bestzeit: 999.0,
        }
    }
}

#[derive(Resource, Default)]
struct LaufDaten {
    punkte: u32,
    verstrichen: f32,
    kristalle_uebrig: usize,
}

fn spielstand_pfad() -> PathBuf {
    directories::ProjectDirs::from("de", "rustwerkstatt", "werkstatt-arena")
        .map(|p| p.data_local_dir().join("spielstand.ron"))
        .unwrap_or_else(|| PathBuf::from("spielstand.ron"))
}

fn spielstand_laden() -> Spielstand {
    fs::read_to_string(spielstand_pfad())
        .ok()
        .and_then(|s| ron::from_str(&s).ok())
        .unwrap_or_default()
}

fn spielstand_speichern(stand: &Spielstand) {
    let pfad = spielstand_pfad();
    if let Some(eltern) = pfad.parent() {
        let _ = fs::create_dir_all(eltern);
    }
    if let Ok(text) = ron::ser::to_string_pretty(stand, ron::ser::PrettyConfig::default()) {
        let _ = fs::write(pfad, text);
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.03, 0.05, 0.08)))
        .insert_resource(spielstand_laden())
        .init_resource::<LaufDaten>()
        .init_state::<ArenaZustand>()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Werkstatt-Arena 3D — Finale Edition".into(),
                    resolution: (1280.0_f32, 720.0_f32).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_systems(Startup, setup_szene)
        .add_systems(OnEnter(ArenaZustand::StartMenue), startmenue_zeigen)
        .add_systems(OnExit(ArenaZustand::StartMenue), overlay_loeschen)
        .add_systems(OnEnter(ArenaZustand::Laeuft), runde_vorbereiten)
        .add_systems(OnEnter(ArenaZustand::Gewonnen), sieg_zeigen)
        .add_systems(OnExit(ArenaZustand::Gewonnen), overlay_loeschen)
        .add_systems(
            Update,
            menue_eingabe.run_if(not(in_state(ArenaZustand::Laeuft))),
        )
        .add_systems(
            Update,
            (
                kugel_rollen,
                hindernis_abprall,
                kristalle_sammeln,
                hud_aktualisieren,
            )
                .run_if(in_state(ArenaZustand::Laeuft)),
        )
        .run();
}

fn setup_szene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.5, 13.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 4500.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.94, 0.84),
            ..default()
        },
        Transform::from_xyz(6.0, 14.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let holz_tex: Handle<Image> = asset_server.load("textures/arena/holz_arena.png");
    let stahl_tex: Handle<Image> = asset_server.load("textures/arena/stahl_rahmen.png");
    let stein_tex: Handle<Image> = asset_server.load("textures/arena/stein_block.png");

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(16.0, 16.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(holz_tex),
            perceptual_roughness: 0.42,
            ..default()
        })),
    ));

    let stahl_mat = materials.add(StandardMaterial {
        base_color_texture: Some(stahl_tex),
        metallic: 0.85,
        perceptual_roughness: 0.28,
        ..default()
    });
    let bande_lang = meshes.add(Cuboid::new(16.4, 0.5, 0.4));
    let bande_quer = meshes.add(Cuboid::new(0.4, 0.5, 16.4));
    for z in [-8.0_f32, 8.0_f32] {
        commands.spawn((
            Mesh3d(bande_lang.clone()),
            MeshMaterial3d(stahl_mat.clone()),
            Transform::from_xyz(0.0, 0.25, z),
        ));
    }
    for x in [-8.0_f32, 8.0_f32] {
        commands.spawn((
            Mesh3d(bande_quer.clone()),
            MeshMaterial3d(stahl_mat.clone()),
            Transform::from_xyz(x, 0.25, 0.0),
        ));
    }

    let stein_mat = materials.add(StandardMaterial {
        base_color_texture: Some(stein_tex),
        perceptual_roughness: 0.65,
        ..default()
    });
    let block_mesh = meshes.add(Cuboid::new(1.8, 1.1, 1.8));
    for (bx, bz) in [(-3.0_f32, -2.5_f32), (3.0_f32, 2.0_f32)] {
        commands.spawn((
            Hindernis {
                halb_groesse: Vec2::splat(0.9),
            },
            Mesh3d(block_mesh.clone()),
            MeshMaterial3d(stein_mat.clone()),
            Transform::from_xyz(bx, 0.55, bz),
        ));
    }

    commands.spawn((
        Kugel { tempo: Vec3::ZERO },
        Mesh3d(meshes.add(Sphere::new(KUGEL_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.82, 0.95),
            metallic: 0.92,
            perceptual_roughness: 0.12,
            ..default()
        })),
        Transform::from_xyz(0.0, KUGEL_RADIUS, 4.5),
    ));

    commands.spawn((
        HudText,
        Text::new("Punkte: 0  |  Zeit: 0.0s"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.95, 0.98)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));
}

fn runde_vorbereiten(
    mut commands: Commands,
    mut lauf: ResMut<LaufDaten>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut kugel_q: Query<(&mut Kugel, &mut Transform)>,
    kristall_q: Query<Entity, With<Kristall>>,
) {
    for e in &kristall_q {
        commands.entity(e).despawn();
    }
    if let Ok((mut k, mut t)) = kugel_q.get_single_mut() {
        k.tempo = Vec3::ZERO;
        t.translation = Vec3::new(0.0, KUGEL_RADIUS, 4.5);
    }

    let punkte = [(-4.8_f32, -4.5_f32), (4.8_f32, -4.5_f32), (0.0_f32, -1.0_f32)];
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.84, 0.22),
        emissive: LinearRgba::rgb(2.2, 1.5, 0.2),
        ..default()
    });
    let mesh = meshes.add(Cuboid::new(0.7, 0.7, 0.7));
    for (kx, kz) in punkte {
        commands.spawn((
            Kristall,
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(kx, 1.0, kz)
                .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.6, 0.4, 0.6)),
        ));
    }
    lauf.punkte = 0;
    lauf.verstrichen = 0.0;
    lauf.kristalle_uebrig = punkte.len();
}

fn kugel_rollen(
    zeit: Res<Time>,
    tasten: Res<ButtonInput<KeyCode>>,
    mut lauf: ResMut<LaufDaten>,
    mut abfrage: Query<(&mut Kugel, &mut Transform)>,
) {
    let dt = zeit.delta_secs();
    lauf.verstrichen += dt;
    let mut schub = Vec3::ZERO;
    if tasten.pressed(KeyCode::KeyW) || tasten.pressed(KeyCode::ArrowUp) {
        schub.z -= 1.0;
    }
    if tasten.pressed(KeyCode::KeyS) || tasten.pressed(KeyCode::ArrowDown) {
        schub.z += 1.0;
    }
    if tasten.pressed(KeyCode::KeyA) || tasten.pressed(KeyCode::ArrowLeft) {
        schub.x -= 1.0;
    }
    if tasten.pressed(KeyCode::KeyD) || tasten.pressed(KeyCode::ArrowRight) {
        schub.x += 1.0;
    }

    for (mut kugel, mut t) in &mut abfrage {
        if schub != Vec3::ZERO {
            kugel.tempo += schub.normalize() * 18.0 * dt;
        }
        kugel.tempo *= (1.0 - 2.2 * dt).max(0.0);
        t.translation += kugel.tempo * dt;

        if t.translation.x.abs() > ARENA_HALB {
            t.translation.x = t.translation.x.signum() * ARENA_HALB;
            kugel.tempo.x *= -0.75;
        }
        if t.translation.z.abs() > ARENA_HALB {
            t.translation.z = t.translation.z.signum() * ARENA_HALB;
            kugel.tempo.z *= -0.75;
        }
    }
}

fn hindernis_abprall(
    mut kugel_q: Query<(&mut Kugel, &mut Transform)>,
    hindernis_q: Query<(&Hindernis, &Transform), Without<Kugel>>,
) {
    let Ok((mut kugel, mut kt)) = kugel_q.get_single_mut() else {
        return;
    };
    for (h, ht) in &hindernis_q {
        let dx = kt.translation.x - ht.translation.x;
        let dz = kt.translation.z - ht.translation.z;
        let mx = h.halb_groesse.x + KUGEL_RADIUS;
        let mz = h.halb_groesse.y + KUGEL_RADIUS;
        if dx.abs() < mx && dz.abs() < mz {
            let ux = mx - dx.abs();
            let uz = mz - dz.abs();
            if ux < uz {
                kt.translation.x += dx.signum() * ux;
                kugel.tempo.x *= -0.8;
            } else {
                kt.translation.z += dz.signum() * uz;
                kugel.tempo.z *= -0.8;
            }
        }
    }
}

fn kristalle_sammeln(
    mut commands: Commands,
    zeit: Res<Time>,
    asset_server: Res<AssetServer>,
    mut lauf: ResMut<LaufDaten>,
    mut stand: ResMut<Spielstand>,
    mut naechster: ResMut<NextState<ArenaZustand>>,
    kugel_q: Query<&Transform, With<Kugel>>,
    mut kristall_q: Query<(Entity, &mut Transform), (With<Kristall>, Without<Kugel>)>,
) {
    let dt = zeit.delta_secs();
    let Ok(kt) = kugel_q.get_single() else {
        return;
    };
    for (e, mut ct) in &mut kristall_q {
        ct.rotate_y(2.2 * dt);
        let dist = Vec2::new(kt.translation.x - ct.translation.x, kt.translation.z - ct.translation.z).length();
        if dist < 1.1 {
            commands.entity(e).despawn();
            lauf.punkte += 600;
            lauf.kristalle_uebrig = lauf.kristalle_uebrig.saturating_sub(1);
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/schluessel.wav")),
                PlaybackSettings::DESPAWN,
            ));
            if lauf.kristalle_uebrig == 0 {
                if lauf.punkte > stand.highscore {
                    stand.highscore = lauf.punkte;
                }
                if lauf.verstrichen < stand.bestzeit {
                    stand.bestzeit = lauf.verstrichen;
                }
                spielstand_speichern(&stand);
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/sieg.wav")),
                    PlaybackSettings::DESPAWN,
                ));
                naechster.set(ArenaZustand::Gewonnen);
            }
        }
    }
}

fn hud_aktualisieren(
    lauf: Res<LaufDaten>,
    stand: Res<Spielstand>,
    mut text_q: Query<&mut Text, With<HudText>>,
) {
    if let Ok(mut text) = text_q.get_single_mut() {
        let best = if stand.bestzeit < 900.0 {
            format!("{:.1}s", stand.bestzeit)
        } else {
            "--".to_string()
        };
        **text = format!(
            "Punkte: {}  |  Zeit: {:.1}s  |  Bestzeit: {}",
            lauf.punkte, lauf.verstrichen, best
        );
    }
}

fn startmenue_zeigen(mut commands: Commands) {
    commands.spawn((
        OverlayText,
        Text::new("WERKSTATT-ARENA 3D\nDruecke [LEERTASTE] zum Starten"),
        TextFont {
            font_size: 38.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.90, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(24.0),
            ..default()
        },
    ));
}

fn sieg_zeigen(mut commands: Commands, lauf: Res<LaufDaten>) {
    commands.spawn((
        OverlayText,
        Text::new(format!(
            "ARENA GEMEISTERT! {} Punkte in {:.1}s\nDruecke [LEERTASTE] fuer neue Runde",
            lauf.punkte, lauf.verstrichen
        )),
        TextFont {
            font_size: 34.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.85, 0.25)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(18.0),
            ..default()
        },
    ));
}

fn overlay_loeschen(mut commands: Commands, q: Query<Entity, With<OverlayText>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn menue_eingabe(
    tasten: Res<ButtonInput<KeyCode>>,
    mut naechster: ResMut<NextState<ArenaZustand>>,
) {
    if tasten.just_pressed(KeyCode::Space) {
        naechster.set(ArenaZustand::Laeuft);
    }
}
