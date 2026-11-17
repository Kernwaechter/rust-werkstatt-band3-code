use bevy::prelude::*;

#[derive(Component)]
struct Spieler;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kometen-Jäger — Teil 1".into(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, szene_aufbauen)
        .run();
}

fn szene_aufbauen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_image(asset_server.load("sprites/weltraum_hintergrund.png")),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    commands.spawn((
        Spieler,
        Sprite {
            image: asset_server.load("sprites/raumschiff.png"),
            custom_size: Some(Vec2::splat(72.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
