use bevy::prelude::*;

const KACHEL_BREITE: f32 = 120.0;
const KACHEL_HOEHE: f32 = 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogEffekt {
    Keine,
    Heilung(u32),
    SchluesselHinweis,
}

#[derive(Clone, Debug)]
pub struct DialogOption {
    pub label: String,
    pub antwort: String,
    pub effekt: DialogEffekt,
}

#[derive(Event)]
pub struct DialogStartEvent {
    pub sprecher: String,
    pub zeilen: Vec<String>,
    pub optionen: Vec<DialogOption>,
}

#[derive(Component)]
struct Held {
    gitter_pos: Vec2,
    leben: u32,
}

#[derive(Component)]
struct RunenSaeule {
    gitter_pos: Vec2,
    titel: String,
    text: Vec<String>,
    optionen: Vec<DialogOption>,
}

#[derive(Component)]
struct DialogPanel;

#[derive(Component)]
struct DialogTextFeld;

#[derive(Resource, Default)]
pub struct DialogZustand {
    pub aktiv: bool,
    pub sprecher: String,
    pub warteschlange: Vec<String>,
    pub optionen: Vec<DialogOption>,
    pub aktueller_text: String,
    pub sichtbare_zeichen: usize,
    pub tipp_timer: f32,
}

pub fn tippe_weiter(zustand: &mut DialogZustand, dt: f32) {
    if !zustand.aktiv {
        return;
    }
    let gesamt = zustand.aktueller_text.chars().count();
    if zustand.sichtbare_zeichen >= gesamt {
        return;
    }
    zustand.tipp_timer += dt;
    while zustand.tipp_timer >= 0.028 && zustand.sichtbare_zeichen < gesamt {
        zustand.tipp_timer -= 0.028;
        zustand.sichtbare_zeichen += 1;
    }
}

/// Wählt eine Antwortoption (0 = Taste [1], 1 = Taste [2]) und liefert den ausgelösten Effekt.
pub fn waehle_dialog_option(zustand: &mut DialogZustand, index: usize) -> DialogEffekt {
    if let Some(opt) = zustand.optionen.get(index).cloned() {
        zustand.aktueller_text = opt.antwort;
        zustand.sichtbare_zeichen = 0;
        zustand.tipp_timer = 0.0;
        zustand.optionen.clear();
        zustand.warteschlange.clear();
        return opt.effekt;
    }
    DialogEffekt::Keine
}

fn grid_to_iso(gitter: Vec2) -> Vec2 {
    Vec2::new(
        (gitter.x - gitter.y) * (KACHEL_BREITE * 0.5),
        -(gitter.x + gitter.y) * (KACHEL_HOEHE * 0.5),
    )
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.10)))
        .init_resource::<DialogZustand>()
        .add_event::<DialogStartEvent>()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Kerker-Flucht: Storytelling, Dialoge & Entscheidungen".into(),
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
            (
                held_und_interaktion,
                dialog_events_verarbeiten,
                dialog_anzeigen,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut dialog_events: EventWriter<DialogStartEvent>,
) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, -210.0, 999.0)));

    let boden_tex: Handle<Image> = asset_server.load("sprites/kerker/boden_kachel.png");
    let saeule_tex: Handle<Image> = asset_server.load("sprites/kerker/saeule.png");
    let held_tex: Handle<Image> = asset_server.load("sprites/kerker/held.png");

    for zeile in 0..7 {
        for spalte in 0..7 {
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

    let saeule_pos = Vec2::new(3.0, 2.0);
    let s_iso = grid_to_iso(saeule_pos);
    commands.spawn((
        RunenSaeule {
            gitter_pos: saeule_pos,
            titel: "Geist des Runenschmieds".into(),
            text: vec![
                "\"Lange schlief diese Werkstatt unter kaltem Stein, Wanderer.\"".into(),
                "\"Nimmst du das Licht der Laterne oder fragst du nach dem Geheimnis?\"".into(),
            ],
            optionen: vec![
                DialogOption {
                    label: "[1] Lichtsegen annehmen (+1 Leben)".into(),
                    antwort: "Waerme durchstroemt deine Laterne. Deine Lebenskraft steigt!"
                        .into(),
                    effekt: DialogEffekt::Heilung(1),
                },
                DialogOption {
                    label: "[2] Nach dem Geheimnis fragen".into(),
                    antwort: "Der Schmied fluestert: \"Beide Runenschluessel ruhen hinter den Skeletten im Osten.\""
                        .into(),
                    effekt: DialogEffekt::SchluesselHinweis,
                },
            ],
        },
        Sprite {
            image: saeule_tex,
            custom_size: Some(Vec2::new(112.0, 196.0)),
            ..default()
        },
        Transform::from_xyz(s_iso.x, s_iso.y + 68.0, 10.0 - s_iso.y * 0.001),
    ));

    let held_pos = Vec2::new(3.0, 4.0);
    let h_iso = grid_to_iso(held_pos);
    commands.spawn((
        Held {
            gitter_pos: held_pos,
            leben: 3,
        },
        Sprite {
            image: held_tex,
            custom_size: Some(Vec2::new(72.0, 72.0)),
            ..default()
        },
        Transform::from_xyz(h_iso.x, h_iso.y + 24.0, 10.0 - h_iso.y * 0.001),
    ));

    commands
        .spawn((
            DialogPanel,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(24.0),
                left: Val::Percent(10.0),
                width: Val::Percent(80.0),
                min_height: Val::Px(120.0),
                padding: UiRect::all(Val::Px(18.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.92)),
            BorderColor(Color::srgb(0.85, 0.68, 0.30)),
        ))
        .with_children(|parent| {
            parent.spawn((
                DialogTextFeld,
                Text::new(""),
                TextFont {
                    font_size: 21.0,
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.93, 0.85)),
            ));
        });

    dialog_events.send(DialogStartEvent {
        sprecher: "Erzaehler".into(),
        zeilen: vec![
            "Die Fackeln flackern im alten Zwergengewoelbe. Tritt naeher an die Runensaeule und druecke [E].".into(),
        ],
        optionen: vec![],
    });
}

fn held_und_interaktion(
    zeit: Res<Time>,
    tasten: Res<ButtonInput<KeyCode>>,
    mut dialog: ResMut<DialogZustand>,
    mut dialog_events: EventWriter<DialogStartEvent>,
    mut held_q: Query<(&mut Held, &mut Transform)>,
    saeulen_q: Query<&RunenSaeule>,
) {
    let Ok((mut held, mut ht)) = held_q.get_single_mut() else {
        return;
    };

    if dialog.aktiv {
        // Verzweigte Wahlmöglichkeiten auf der letzten Zeile mit [1] oder [2]
        if dialog.warteschlange.is_empty() && !dialog.optionen.is_empty() {
            let wahl = if tasten.just_pressed(KeyCode::Digit1) {
                Some(0)
            } else if tasten.just_pressed(KeyCode::Digit2) {
                Some(1)
            } else {
                None
            };
            if let Some(idx) = wahl {
                if let DialogEffekt::Heilung(punkte) = waehle_dialog_option(&mut dialog, idx) {
                    held.leben += punkte;
                }
                return;
            }
        }

        if tasten.just_pressed(KeyCode::Enter) || tasten.just_pressed(KeyCode::KeyE) {
            let gesamt = dialog.aktueller_text.chars().count();
            if dialog.sichtbare_zeichen < gesamt {
                dialog.sichtbare_zeichen = gesamt;
            } else if !dialog.warteschlange.is_empty() {
                dialog.aktueller_text = dialog.warteschlange.remove(0);
                dialog.sichtbare_zeichen = 0;
            } else if dialog.optionen.is_empty() {
                dialog.aktiv = false;
            }
        }
        return;
    }

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
    if richtung != Vec2::ZERO {
        held.gitter_pos = (held.gitter_pos + richtung.normalize() * 3.2 * zeit.delta_secs())
            .clamp(Vec2::splat(0.4), Vec2::splat(5.6));
        let iso = grid_to_iso(held.gitter_pos);
        ht.translation.x = iso.x;
        ht.translation.y = iso.y + 24.0;
        ht.translation.z = 10.0 - iso.y * 0.001;
    }

    if tasten.just_pressed(KeyCode::KeyE) {
        for saeule in &saeulen_q {
            if held.gitter_pos.distance(saeule.gitter_pos) < 1.6 {
                dialog_events.send(DialogStartEvent {
                    sprecher: saeule.titel.clone(),
                    zeilen: saeule.text.clone(),
                    optionen: saeule.optionen.clone(),
                });
            }
        }
    }
}

fn dialog_events_verarbeiten(
    mut events: EventReader<DialogStartEvent>,
    mut dialog: ResMut<DialogZustand>,
) {
    for ev in events.read() {
        if let Some((erste, rest)) = ev.zeilen.split_first() {
            dialog.aktiv = true;
            dialog.sprecher = ev.sprecher.clone();
            dialog.aktueller_text = erste.clone();
            dialog.warteschlange = rest.to_vec();
            dialog.optionen = ev.optionen.clone();
            dialog.sichtbare_zeichen = 0;
            dialog.tipp_timer = 0.0;
        }
    }
}

fn dialog_anzeigen(
    zeit: Res<Time>,
    mut dialog: ResMut<DialogZustand>,
    mut panel_q: Query<&mut Visibility, With<DialogPanel>>,
    mut text_q: Query<&mut Text, With<DialogTextFeld>>,
) {
    tippe_weiter(&mut dialog, zeit.delta_secs());

    if let Ok(mut vis) = panel_q.get_single_mut() {
        *vis = if dialog.aktiv {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut text) = text_q.get_single_mut() {
        if dialog.aktiv {
            let teil: String = dialog
                .aktueller_text
                .chars()
                .take(dialog.sichtbare_zeichen)
                .collect();
            if dialog.warteschlange.is_empty() && !dialog.optionen.is_empty() {
                let auswahl: Vec<String> = dialog.optionen.iter().map(|o| o.label.clone()).collect();
                **text = format!("[{}]\n{}\n{}", dialog.sprecher, teil, auswahl.join("   |   "));
            } else {
                **text = format!("[{}]\n{}  (Enter/E)", dialog.sprecher, teil);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schreibmaschinen_effekt_laeuft_vorwaerts() {
        let mut d = DialogZustand {
            aktiv: true,
            sprecher: "Runen".into(),
            warteschlange: vec![],
            optionen: vec![],
            aktueller_text: "Hallo Welt".into(),
            sichtbare_zeichen: 0,
            tipp_timer: 0.0,
        };
        tippe_weiter(&mut d, 0.15);
        assert_eq!(d.sichtbare_zeichen, 5);
    }

    #[test]
    fn dialog_option_loest_heilung_aus() {
        let mut d = DialogZustand {
            aktiv: true,
            sprecher: "Schmied".into(),
            warteschlange: vec![],
            optionen: vec![DialogOption {
                label: "[1] Heilung".into(),
                antwort: "Geheilt!".into(),
                effekt: DialogEffekt::Heilung(1),
            }],
            aktueller_text: "Waehle".into(),
            sichtbare_zeichen: 6,
            tipp_timer: 0.0,
        };
        let effekt = waehle_dialog_option(&mut d, 0);
        assert_eq!(effekt, DialogEffekt::Heilung(1));
        assert_eq!(d.aktueller_text, "Geheilt!");
        assert!(d.optionen.is_empty());
    }
}
