use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{AnimationTarget, SpritesheetPlugin};
use support::{
    apply_example_defaults, kenney_directional_library, load_kenney_directional_atlas, spawn_actor,
    spawn_demo_backdrop, spawn_demo_camera, spawn_overlay, write_overlay,
};

#[derive(Component)]
struct DirectionalActor;

#[derive(Component)]
struct Overlay;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Facing {
    Up,
    Down,
    Left,
    Right,
}

impl Facing {
    fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Resource)]
struct DirectionCycle {
    current: Facing,
}

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet directional");
    support::install_pane(&mut app);
    app.insert_resource(DirectionCycle {
        current: Facing::Down,
    });
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, (cycle_facing, update_overlay));
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut libraries: ResMut<Assets<saddle_animation_spritesheet::AnimationLibrary>>,
) {
    spawn_demo_camera(&mut commands);
    spawn_demo_backdrop(&mut commands);

    let atlas = load_kenney_directional_atlas(&asset_server, &mut layouts);
    let library = libraries.add(kenney_directional_library());

    let actor = spawn_actor(
        &mut commands,
        "Directional Actor",
        &atlas,
        library,
        AnimationTarget::state("down"),
        Vec3::new(0.0, -40.0, 0.0),
        5.0,
        Color::WHITE,
    );
    commands.entity(actor).insert(DirectionalActor);

    let overlay = spawn_overlay(&mut commands, "spritesheet directional");
    commands.entity(overlay).insert(Overlay);
}

fn cycle_facing(
    time: Res<Time>,
    mut cycle: ResMut<DirectionCycle>,
    mut query: Query<
        &mut saddle_animation_spritesheet::AnimationController,
        With<DirectionalActor>,
    >,
) {
    let segment = (time.elapsed_secs() * 0.8).floor() as i32;
    let next = match segment.rem_euclid(4) {
        0 => Facing::Down,
        1 => Facing::Left,
        2 => Facing::Up,
        _ => Facing::Right,
    };

    if cycle.current != next {
        cycle.current = next;
        for mut controller in &mut query {
            controller.set_target(AnimationTarget::state(cycle.current.as_str()));
        }
    }
}

fn update_overlay(
    facing: Res<DirectionCycle>,
    actor: Single<&saddle_animation_spritesheet::SpritesheetAnimator, With<DirectionalActor>>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    write_overlay(
        &mut text,
        "spritesheet directional",
        format!(
            "The Kenney character cycles through directional state clips every ~1.25s.\n\nFacing: {}\nCurrent clip: {}\nFrame: {}\nAtlas index: {}\n",
            facing.current.as_str(),
            actor
                .current_clip
                .as_ref()
                .map(|clip| clip.as_str())
                .unwrap_or("none"),
            actor.current_frame,
            actor.atlas_index,
        ),
    );
}
