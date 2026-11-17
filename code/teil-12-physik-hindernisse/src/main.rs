use bevy::prelude::*;

const ARENA_HALB: f32 = 7.2;
const KUGEL_RADIUS: f32 = 0.55;

#[derive(Component)]
struct Kugel {
    tempo: Vec3,
}

#[derive(Component)]
struct Hindernis {
    halb_groesse: Vec2,
}

#[derive(Component)]
struct Kristall;

#[derive(Resource, Default)]
struct Punkte(u32);

/// Begrenzt eine 3D-Position auf die Arena und spiegelt die Geschwindigkeit bei Bandenkontakt.
pub fn banden_abprall(pos: &mut Vec3, tempo: &mut Vec3, halb_grenze: f32) {
    if pos.x.abs() > halb_grenze {
        pos.x = pos.x.signum() * halb_grenze;
        tempo.x *= -0.75;
    }
    if pos.z.abs() > halb_grenze {
        pos.z = pos.z.signum() * halb_grenze;
        tempo.z *= -0.75;
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.03, 0.05, 0.08)))
        .init_resource::<Punkte>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (kugel_physik, hindernis_kollision, kristalle_sammeln))
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
    commands.spawn((
        DirectionalLight {
            illuminance: 4500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 14.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let holz_tex: Handle<Image> = asset_server.load("textures/arena/holz_arena.png");
    let stein_tex: Handle<Image> = asset_server.load("textures/arena/stein_block.png");

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(16.0, 16.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(holz_tex),
            perceptual_roughness: 0.42,
            ..default()
        })),
    ));

    let stein_mat = materials.add(StandardMaterial {
        base_color_texture: Some(stein_tex),
        perceptual_roughness: 0.65,
        ..default()
    });
    let block = meshes.add(Cuboid::new(1.8, 1.1, 1.8));
    for (bx, bz) in [(-3.0_f32, -2.0_f32), (3.0_f32, 2.0_f32)] {
        commands.spawn((
            Hindernis {
                halb_groesse: Vec2::splat(0.9),
            },
            Mesh3d(block.clone()),
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
        Transform::from_xyz(0.0, KUGEL_RADIUS, 4.0),
    ));

    commands.spawn((
        Kristall,
        Mesh3d(meshes.add(Cuboid::new(0.75, 0.75, 0.75))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.84, 0.22),
            emissive: LinearRgba::rgb(2.2, 1.5, 0.2),
            ..default()
        })),
        Transform::from_xyz(4.0, 1.0, -3.5)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.6, 0.4, 0.6)),
    ));
}

fn kugel_physik(
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
    for (mut k, mut t) in &mut abfrage {
        if schub != Vec3::ZERO {
            k.tempo += schub.normalize() * 18.0 * dt;
        }
        k.tempo *= (1.0 - 2.2 * dt).max(0.0);
        t.translation += k.tempo * dt;
        banden_abprall(&mut t.translation, &mut k.tempo, ARENA_HALB);
    }
}

fn hindernis_kollision(
    mut kugel_q: Query<(&mut Kugel, &mut Transform)>,
    hindernis_q: Query<(&Hindernis, &Transform), Without<Kugel>>,
) {
    let Ok((mut k, mut kt)) = kugel_q.get_single_mut() else {
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
                k.tempo.x *= -0.8;
            } else {
                kt.translation.z += dz.signum() * uz;
                k.tempo.z *= -0.8;
            }
        }
    }
}

fn kristalle_sammeln(
    mut commands: Commands,
    zeit: Res<Time>,
    mut punkte: ResMut<Punkte>,
    kugel_q: Query<&Transform, With<Kugel>>,
    mut kristall_q: Query<(Entity, &mut Transform), (With<Kristall>, Without<Kugel>)>,
) {
    let dt = zeit.delta_secs();
    let Ok(kt) = kugel_q.get_single() else {
        return;
    };
    for (e, mut ct) in &mut kristall_q {
        ct.rotate_y(2.0 * dt);
        if Vec2::new(kt.translation.x - ct.translation.x, kt.translation.z - ct.translation.z).length() < 1.1 {
            punkte.0 += 500;
            commands.entity(e).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banden_abprall_spiegelt_x_tempo() {
        let mut pos = Vec3::new(8.0, 0.55, 0.0);
        let mut tempo = Vec3::new(4.0, 0.0, 0.0);
        banden_abprall(&mut pos, &mut tempo, ARENA_HALB);
        assert_eq!(pos.x, ARENA_HALB);
        assert!(tempo.x < 0.0);
    }
}
