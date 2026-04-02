use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{AnimationController, AnimationEventFired, AnimationTarget, SpritesheetPlugin};
use support::{
    apply_example_defaults, main_library, make_demo_atlas, spawn_actor, spawn_demo_backdrop,
    spawn_demo_camera, spawn_overlay, write_overlay,
};

#[derive(Component)]
struct Actor;

#[derive(Component)]
struct Beacon;

#[derive(Component)]
struct Overlay;

#[derive(Resource)]
struct EventPreview {
    timer: Timer,
    flashes: u32,
    glow: f32,
}

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet frame events");
    app.insert_resource(EventPreview {
        timer: Timer::from_seconds(1.6, TimerMode::Repeating),
        flashes: 0,
        glow: 0.0,
    });
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            trigger_action,
            react_to_events,
            update_beacon,
            update_overlay,
        ),
    );
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
        "Frame Event Actor",
        &atlas,
        library,
        AnimationTarget::state("idle"),
        Vec3::new(-120.0, -120.0, 0.0),
        8.0,
        Color::WHITE,
    );
    commands.entity(actor).insert(Actor);

    commands.spawn((
        Name::new("Impact Beacon"),
        Beacon,
        Sprite::from_color(Color::srgb(0.2, 0.22, 0.26), Vec2::new(140.0, 140.0)),
        Transform::from_xyz(250.0, -100.0, 0.0),
    ));

    let overlay = spawn_overlay(&mut commands, "spritesheet frame events");
    commands.entity(overlay).insert(Overlay);
}

fn trigger_action(
    time: Res<Time>,
    mut preview: ResMut<EventPreview>,
    mut query: Query<&mut AnimationController, With<Actor>>,
) {
    if !preview.timer.tick(time.delta()).just_finished() {
        return;
    }

    for mut controller in &mut query {
        controller.play_state_once("use_tool");
        controller.requested_target = Some(AnimationTarget::state("idle"));
    }
}

fn react_to_events(
    mut preview: ResMut<EventPreview>,
    mut events: MessageReader<AnimationEventFired>,
) {
    for message in events.read() {
        if message.marker.name == "impact" {
            preview.flashes += 1;
            preview.glow = 1.0;
        }
    }
}

fn update_beacon(
    time: Res<Time>,
    mut preview: ResMut<EventPreview>,
    mut beacon: Single<&mut Sprite, With<Beacon>>,
) {
    preview.glow = (preview.glow - time.delta_secs() * 1.8).max(0.0);
    beacon.color = Color::srgb(0.2 + preview.glow * 0.75, 0.22 + preview.glow * 0.35, 0.26);
}

fn update_overlay(
    preview: Res<EventPreview>,
    actor: Single<&saddle_animation_spritesheet::SpritesheetAnimator, With<Actor>>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    let animator = *actor;
    write_overlay(
        &mut text,
        "spritesheet frame events",
        format!(
            "The beacon flashes only when the one-shot crosses the 'impact' frame marker.\nCurrent clip: {}\nCurrent frame: {}\nImpact flashes: {}\nBeacon glow: {:.2}",
            animator
                .current_clip
                .as_ref()
                .map(|clip| clip.as_str())
                .unwrap_or("none"),
            animator.current_frame,
            preview.flashes,
            preview.glow,
        ),
    );
}
