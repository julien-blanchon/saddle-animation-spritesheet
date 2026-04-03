use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationController, AnimationLibrary, AnimationTarget, SpritesheetAnimator, SpritesheetPlugin,
};
use support::{
    apply_example_defaults, make_demo_atlas, spawn_actor, spawn_demo_backdrop, spawn_demo_camera,
    spawn_overlay, write_overlay,
};

const ASEPRITE_JSON: &str = r#"
{
  "frames": {
    "frame_0": { "frame": { "x": 0, "y": 0, "w": 24, "h": 24 }, "duration": 220 },
    "frame_1": { "frame": { "x": 24, "y": 0, "w": 24, "h": 24 }, "duration": 220 },
    "frame_2": { "frame": { "x": 48, "y": 0, "w": 24, "h": 24 }, "duration": 100 },
    "frame_3": { "frame": { "x": 72, "y": 0, "w": 24, "h": 24 }, "duration": 100 },
    "frame_4": { "frame": { "x": 96, "y": 0, "w": 24, "h": 24 }, "duration": 100 },
    "frame_5": { "frame": { "x": 120, "y": 0, "w": 24, "h": 24 }, "duration": 160 },
    "frame_6": { "frame": { "x": 144, "y": 0, "w": 24, "h": 24 }, "duration": 140 },
    "frame_7": { "frame": { "x": 168, "y": 0, "w": 24, "h": 24 }, "duration": 140 }
  },
  "meta": {
    "size": { "w": 192, "h": 24 },
    "frameTags": [
      { "name": "idle", "from": 0, "to": 1, "direction": "pingpong" },
      { "name": "walk", "from": 2, "to": 4, "direction": "forward" },
      { "name": "flare", "from": 5, "to": 7, "direction": "forward" }
    ]
  }
}
"#;

#[derive(Component)]
struct LanternKeeper;

#[derive(Component)]
struct Overlay;

#[derive(Resource, Default)]
struct CycleState {
    segment: i32,
}

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet aseprite import");
    support::install_pane(&mut app);
    app.init_resource::<CycleState>();
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, (drive_scene_cycle, update_overlay));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut libraries: ResMut<Assets<AnimationLibrary>>,
) {
    spawn_demo_camera(&mut commands);
    spawn_demo_backdrop(&mut commands);

    commands.spawn((
        Name::new("Moon Arch"),
        Sprite {
            color: Color::srgba(0.24, 0.31, 0.46, 0.34),
            custom_size: Some(Vec2::new(260.0, 220.0)),
            ..default()
        },
        Transform::from_xyz(220.0, 20.0, -22.0),
    ));
    commands.spawn((
        Name::new("Lantern Pool"),
        Sprite {
            color: Color::srgba(1.0, 0.74, 0.26, 0.18),
            custom_size: Some(Vec2::new(300.0, 140.0)),
            ..default()
        },
        Transform::from_xyz(-20.0, -120.0, -19.0),
    ));
    commands.spawn((
        Name::new("Bridge Rail"),
        Sprite::from_color(Color::srgb(0.16, 0.12, 0.10), Vec2::new(460.0, 26.0)),
        Transform::from_xyz(0.0, -164.0, -10.0),
    ));

    let atlas = make_demo_atlas(&mut images, &mut layouts);
    let library = libraries.add(
        AnimationLibrary::from_aseprite_json("lantern_keeper", ASEPRITE_JSON)
            .expect("embedded Aseprite JSON should parse"),
    );
    let actor = spawn_actor(
        &mut commands,
        "Lantern Keeper",
        &atlas,
        library,
        AnimationTarget::state("idle"),
        Vec3::new(-140.0, -132.0, 0.0),
        8.5,
        Color::WHITE,
    );
    commands.entity(actor).insert(LanternKeeper);

    let overlay = spawn_overlay(&mut commands, "spritesheet aseprite import");
    commands.entity(overlay).insert(Overlay);
}

fn drive_scene_cycle(
    time: Res<Time>,
    mut cycle: ResMut<CycleState>,
    mut query: Query<(&mut Transform, &mut AnimationController), With<LanternKeeper>>,
) {
    let next_segment = (time.elapsed_secs() / 2.0).floor() as i32;
    if cycle.segment == next_segment {
        for (mut transform, _) in &mut query {
            let sway = (time.elapsed_secs() * 1.4).sin() * 6.0;
            transform.translation.x = -140.0 + sway;
        }
        return;
    }

    cycle.segment = next_segment;
    let phase = next_segment.rem_euclid(3);

    for (mut transform, mut controller) in &mut query {
        match phase {
            0 => {
                controller.set_target(AnimationTarget::state("idle"));
                transform.scale.x = 8.5;
            }
            1 => {
                controller.set_target(AnimationTarget::state("walk"));
                transform.scale.x = -8.5;
            }
            _ => {
                controller.play_state_once("flare");
                transform.scale.x = 8.5;
            }
        }
    }
}

fn update_overlay(
    actor: Single<&SpritesheetAnimator, With<LanternKeeper>>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    let animator = *actor;
    write_overlay(
        &mut text,
        "spritesheet aseprite import",
        format!(
            "This scene imports clip/state tags directly from embedded Aseprite JSON.\n\nCurrent clip: {}\nCurrent state: {}\nFrame: {}\nAtlas index: {}\nLoops: {}\nPlayback: {:?}",
            animator
                .current_clip
                .as_ref()
                .map(|clip| clip.as_str())
                .unwrap_or("none"),
            animator
                .current_state
                .as_ref()
                .map(|state| state.as_str())
                .unwrap_or("none"),
            animator.current_frame,
            animator.atlas_index,
            animator.completed_loops,
            animator.playback_state,
        ),
    );
}
