use bevy::prelude::*;

const ARENA_HALB: f32 = 7.2;
const KUGEL_RADIUS: f32 = 0.55;

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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.03, 0.05, 0.08)))
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Werkstatt-Arena 3D: PBR-Materialien & Licht".into(),
                    resolution: (1280.0_f32, 720.0_f32).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (kugel_rollen, hindernis_abprall, kristall_drehen))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.5, 13.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Warmes Werkstatt-Deckenlicht mit Echtzeitschatten
    commands.spawn((
        DirectionalLight {
            illuminance: 4500.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.94, 0.84),
            ..default()
        },
        Transform::from_xyz(6.0, 14.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Warme Punktlichter über der Arena
    for x in [-4.5_f32, 4.5_f32] {
        commands.spawn((
            PointLight {
                intensity: 900_000.0,
                range: 24.0,
                color: Color::srgb(1.0, 0.80, 0.52),
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(x, 6.5, 0.0),
        ));
    }

    // Texturierte Eichenholz-Plattform
    let holz_tex: Handle<Image> = asset_server.load("textures/arena/holz_arena.png");
    let stahl_tex: Handle<Image> = asset_server.load("textures/arena/stahl_rahmen.png");
    let stein_tex: Handle<Image> = asset_server.load("textures/arena/stein_block.png");

    let boden_mat = materials.add(StandardMaterial {
        base_color_texture: Some(holz_tex),
        perceptual_roughness: 0.42,
        reflectance: 0.5,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(16.0, 16.0))),
        MeshMaterial3d(boden_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Gebürstete Stahl-Banden rund um die Plattform
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

    // Dunkle Stein-/Gusseisen-Hindernisblöcke
    let stein_mat = materials.add(StandardMaterial {
        base_color_texture: Some(stein_tex),
        perceptual_roughness: 0.65,
        metallic: 0.2,
        ..default()
    });
    let block_mesh = meshes.add(Cuboid::new(1.8, 1.1, 1.8));
    for (bx, bz) in [(-3.0_f32, -2.5_f32), (3.0_f32, 2.0_f32), (0.0_f32, -4.5_f32)] {
        commands.spawn((
            Hindernis {
                halb_groesse: Vec2::splat(0.9),
            },
            Mesh3d(block_mesh.clone()),
            MeshMaterial3d(stein_mat.clone()),
            Transform::from_xyz(bx, 0.55, bz),
        ));
    }

    // Glänzende cyan-metallische Spielerkugel
    let kugel_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.82, 0.95),
        metallic: 0.92,
        perceptual_roughness: 0.12,
        emissive: LinearRgba::rgb(0.02, 0.22, 0.30),
        ..default()
    });
    commands.spawn((
        Kugel { tempo: Vec3::ZERO },
        Mesh3d(meshes.add(Sphere::new(KUGEL_RADIUS))),
        MeshMaterial3d(kugel_mat),
        Transform::from_xyz(0.0, KUGEL_RADIUS, 4.0),
    ));

    // Leuchtender goldener Kristall
    let kristall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.84, 0.22),
        emissive: LinearRgba::rgb(2.2, 1.5, 0.2),
        metallic: 0.4,
        perceptual_roughness: 0.15,
        ..default()
    });
    commands.spawn((
        Kristall,
        Mesh3d(meshes.add(Cuboid::new(0.75, 0.75, 0.75))),
        MeshMaterial3d(kristall_mat),
        Transform::from_xyz(4.0, 1.1, -3.5)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.6, 0.4, 0.6)),
    ));
}

fn kugel_rollen(
    zeit: Res<Time>,
    tasten: Res<ButtonInput<KeyCode>>,
    mut abfrage: Query<(&mut Kugel, &mut Transform)>,
) {
    let dt = zeit.delta_secs();
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

    for (mut kugel, mut transform) in &mut abfrage {
        if schub != Vec3::ZERO {
            kugel.tempo += schub.normalize() * 18.0 * dt;
        }
        kugel.tempo *= (1.0 - 2.2 * dt).max(0.0);
        transform.translation += kugel.tempo * dt;

        // Banden-Abprall
        if transform.translation.x.abs() > ARENA_HALB {
            transform.translation.x = transform.translation.x.signum() * ARENA_HALB;
            kugel.tempo.x *= -0.75;
        }
        if transform.translation.z.abs() > ARENA_HALB {
            transform.translation.z = transform.translation.z.signum() * ARENA_HALB;
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
    for (hindernis, ht) in &hindernis_q {
        let diff_x = kt.translation.x - ht.translation.x;
        let diff_z = kt.translation.z - ht.translation.z;
        let min_x = hindernis.halb_groesse.x + KUGEL_RADIUS;
        let min_z = hindernis.halb_groesse.y + KUGEL_RADIUS;
        if diff_x.abs() < min_x && diff_z.abs() < min_z {
            let ueberlapp_x = min_x - diff_x.abs();
            let ueberlapp_z = min_z - diff_z.abs();
            if ueberlapp_x < ueberlapp_z {
                kt.translation.x += diff_x.signum() * ueberlapp_x;
                kugel.tempo.x *= -0.8;
            } else {
                kt.translation.z += diff_z.signum() * ueberlapp_z;
                kugel.tempo.z *= -0.8;
            }
        }
    }
}

fn kristall_drehen(zeit: Res<Time>, mut abfrage: Query<&mut Transform, With<Kristall>>) {
    let dt = zeit.delta_secs();
    for mut transform in &mut abfrage {
        transform.rotate_y(2.0 * dt);
    }
}
