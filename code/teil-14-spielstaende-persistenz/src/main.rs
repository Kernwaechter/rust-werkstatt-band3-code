use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Resource, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Spielstand {
    pub highscore: u32,
    pub bestzeit_sekunden: f32,
}

impl Default for Spielstand {
    fn default() -> Self {
        Self {
            highscore: 0,
            bestzeit_sekunden: 999.0,
        }
    }
}

pub fn spielstand_pfad() -> PathBuf {
    if let Some(proj) = ProjectDirs::from("de", "rustwerkstatt", "werkstatt-arena") {
        proj.data_local_dir().join("spielstand.ron")
    } else {
        PathBuf::from("spielstand.ron")
    }
}

pub fn spielstand_speichern(pfad: &std::path::Path, stand: &Spielstand) -> Result<(), String> {
    if let Some(eltern) = pfad.parent() {
        fs::create_dir_all(eltern).map_err(|e| e.to_string())?;
    }
    let text = ron::ser::to_string_pretty(stand, ron::ser::PrettyConfig::default())
        .map_err(|e| e.to_string())?;
    fs::write(pfad, text).map_err(|e| e.to_string())
}

pub fn spielstand_laden(pfad: &std::path::Path) -> Spielstand {
    fs::read_to_string(pfad)
        .ok()
        .and_then(|inhalt| ron::from_str::<Spielstand>(&inhalt).ok())
        .unwrap_or_default()
}

fn main() {
    let pfad = spielstand_pfad();
    let geladen = spielstand_laden(&pfad);

    App::new()
        .insert_resource(geladen)
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    stand: Res<Spielstand>,
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
        Text::new(format!(
            "Highscore: {}  |  Bestzeit: {:.1}s",
            stand.highscore, stand.bestzeit_sekunden
        )),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ron_speichern_und_laden() {
        let temp_pfad = std::env::temp_dir().join("rust_werkstatt_test_stand.ron");
        let original = Spielstand {
            highscore: 1800,
            bestzeit_sekunden: 72.4,
        };
        spielstand_speichern(&temp_pfad, &original).expect("Speichern fehlgeschlagen");
        let geladen = spielstand_laden(&temp_pfad);
        let _ = fs::remove_file(&temp_pfad);
        assert_eq!(original, geladen);
    }
}
