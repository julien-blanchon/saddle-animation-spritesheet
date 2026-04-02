use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationChanged, AnimationController, AnimationEventFired, AnimationFinished, AnimationTarget,
    SpritesheetPlugin,
};
use support::{
    apply_example_defaults, main_library, make_demo_atlas, spawn_actor, spawn_demo_backdrop,
    spawn_demo_camera, spawn_overlay, write_overlay,
};

#[derive(Component)]
struct Actor;

#[derive(Component)]
struct Overlay;

#[derive(Resource)]
struct ActionCycle {
    timer: Timer,
    clip_changes: u32,
    impacts: u32,
    finishes: u32,
    last_clip: String,
}

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet state machine");
    app.insert_resource(ActionCycle {
        timer: Timer::from_seconds(1.8, TimerMode::Repeating),
        clip_changes: 0,
        impacts: 0,
        finishes: 0,
        last_clip: "idle_clip".into(),
    });
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, (trigger_use_tool, record_messages, update_overlay));
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
    let library = libraries.add(main_library());
    let actor = spawn_actor(
        &mut commands,
        "Action Actor",
        &atlas,
        library,
        AnimationTarget::state("idle"),
        Vec3::new(0.0, -120.0, 0.0),
        8.0,
        Color::WHITE,
    );
    commands.entity(actor).insert(Actor);

    let overlay = spawn_overlay(&mut commands, "spritesheet state machine");
    commands.entity(overlay).insert(Overlay);
}

fn trigger_use_tool(
    time: Res<Time>,
    mut cycle: ResMut<ActionCycle>,
    mut query: Query<&mut AnimationController, With<Actor>>,
) {
    if !cycle.timer.tick(time.delta()).just_finished() {
        return;
    }

    for mut controller in &mut query {
        controller.play_state_once("use_tool");
        controller.requested_target = Some(AnimationTarget::state("idle"));
    }
}

fn record_messages(
    mut cycle: ResMut<ActionCycle>,
    mut changed: MessageReader<AnimationChanged>,
    mut events: MessageReader<AnimationEventFired>,
    mut finished: MessageReader<AnimationFinished>,
) {
    for message in changed.read() {
        cycle.clip_changes += 1;
        cycle.last_clip = message.clip.as_str().to_string();
    }

    cycle.impacts += events
        .read()
        .filter(|message| message.marker.name == "impact")
        .count() as u32;
    cycle.finishes += finished.read().count() as u32;
}

fn update_overlay(
    cycle: Res<ActionCycle>,
    actor: Single<&saddle_animation_spritesheet::SpritesheetAnimator, With<Actor>>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    let animator = *actor;
    write_overlay(
        &mut text,
        "spritesheet state machine",
        format!(
            "A repeating trigger plays a locked one-shot, emits a frame event, then falls back to idle.\nCurrent clip: {}\nPlayback: {:?}\nClip changes: {}\nImpact markers: {}\nFinished messages: {}",
            animator
                .current_clip
                .as_ref()
                .map(|clip| clip.as_str())
                .unwrap_or("none"),
            animator.playback_state,
            cycle.clip_changes,
            cycle.impacts,
            cycle.finishes,
        ),
    );
}
