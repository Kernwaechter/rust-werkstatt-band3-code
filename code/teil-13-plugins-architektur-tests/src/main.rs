use bevy::prelude::*;

#[derive(Component)]
pub struct Kugel {
    pub radius: f32,
}

#[derive(Component)]
pub struct Kristall {
    pub punkte: u32,
}

#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct Punktestand {
    pub punkte: u32,
    pub gesammelt: u32,
}

/// Kapselt die Sammel-Logik als eigenständiges Bevy-Plugin.
pub struct KristallPunktePlugin;

impl Plugin for KristallPunktePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Punktestand>()
            .add_systems(Update, kristalle_einsammeln);
    }
}

pub fn kristalle_einsammeln(
    mut commands: Commands,
    mut stand: ResMut<Punktestand>,
    kugel_q: Query<(&Kugel, &Transform)>,
    kristall_q: Query<(Entity, &Kristall, &Transform)>,
) {
    let Ok((kugel, kt)) = kugel_q.get_single() else {
        return;
    };
    for (entity, kristall, ct) in &kristall_q {
        let abstand = kt.translation.truncate().distance(ct.translation.truncate());
        if abstand <= kugel.radius + 0.7 {
            stand.punkte += kristall.punkte;
            stand.gesammelt += 1;
            commands.entity(entity).despawn();
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, KristallPunktePlugin))
        .add_systems(Startup, demo_szene)
        .run();
}

fn demo_szene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 9.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 12.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let holz_tex: Handle<Image> = asset_server.load("textures/arena/holz_arena.png");
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(14.0, 14.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(holz_tex),
            perceptual_roughness: 0.45,
            ..default()
        })),
    ));

    commands.spawn((
        Kugel { radius: 0.55 },
        Mesh3d(meshes.add(Sphere::new(0.55))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.82, 0.95),
            metallic: 0.9,
            perceptual_roughness: 0.12,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.55, 0.0),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_ecs_kristall_einsammeln() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, KristallPunktePlugin));

        // Kugel und Kristall direkt nebeneinander spawnen (ohne GPU/Fenster!)
        app.world_mut()
            .spawn((Kugel { radius: 0.55 }, Transform::from_xyz(0.0, 0.55, 0.0)));
        let kristall_id = app
            .world_mut()
            .spawn((
                Kristall { punkte: 250 },
                Transform::from_xyz(0.3, 0.55, 0.0),
            ))
            .id();

        app.update();

        let stand = app.world().resource::<Punktestand>();
        assert_eq!(stand.punkte, 250);
        assert_eq!(stand.gesammelt, 1);
        assert!(app.world().get_entity(kristall_id).is_err());
    }
}
