use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::SpritesheetPlugin;
use saddle_animation_spritesheet::{
    AnimationClip, AnimationLibrary, AnimationTarget, AnimationTarget as Target, ClipFrame,
    FrameTiming,
};
use support::{
    apply_example_defaults, make_demo_atlas, spawn_actor, spawn_demo_backdrop, spawn_demo_camera,
    spawn_overlay, write_overlay,
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
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut libraries: ResMut<Assets<saddle_animation_spritesheet::AnimationLibrary>>,
) {
    spawn_demo_camera(&mut commands);
    spawn_demo_backdrop(&mut commands);

    let atlas = make_demo_atlas(&mut images, &mut layouts);
    let library = libraries.add(directional_library());

    let actor = spawn_actor(
        &mut commands,
        "Directional Actor",
        &atlas,
        library,
        AnimationTarget::state("down"),
        Vec3::new(0.0, -40.0, 0.0),
        8.0,
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
            controller.set_target(match cycle.current {
                Facing::Down => Target::state("down"),
                Facing::Left => Target::state("left"),
                Facing::Up => Target::state("up"),
                Facing::Right => Target::state("right"),
            });
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
            "The same actor cycles through directional state clips every ~1.25s.\n\nFacing: {}\nCurrent clip: {}\nFrame: {}\nAtlas index: {}\n",
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

fn directional_library() -> AnimationLibrary {
    AnimationLibrary::new("demo_directional")
        .with_default_target(AnimationTarget::state("down"))
        .add_clip(
            AnimationClip::from_frames("down_clip", [ClipFrame::new(0), ClipFrame::new(1)])
                .with_timing(FrameTiming::SecondsPerFrame(0.2)),
        )
        .add_clip(
            AnimationClip::from_frames("up_clip", [ClipFrame::new(2), ClipFrame::new(3)])
                .with_timing(FrameTiming::SecondsPerFrame(0.2)),
        )
        .add_clip(
            AnimationClip::from_frames("left_clip", [ClipFrame::new(4), ClipFrame::new(5)])
                .with_timing(FrameTiming::SecondsPerFrame(0.2)),
        )
        .add_clip(
            AnimationClip::from_frames("right_clip", [ClipFrame::new(6), ClipFrame::new(7)])
                .with_timing(FrameTiming::SecondsPerFrame(0.2)),
        )
        .add_state(saddle_animation_spritesheet::AnimationState::new(
            "down",
            "down_clip",
        ))
        .add_state(saddle_animation_spritesheet::AnimationState::new(
            "up", "up_clip",
        ))
        .add_state(saddle_animation_spritesheet::AnimationState::new(
            "left",
            "left_clip",
        ))
        .add_state(saddle_animation_spritesheet::AnimationState::new(
            "right",
            "right_clip",
        ))
}
