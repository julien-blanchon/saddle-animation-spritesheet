use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationController, AnimationTarget, SpritesheetAnimator, SpritesheetPlugin,
};
use support::{
    apply_example_defaults, main_library, make_demo_atlas, spawn_actor, spawn_demo_backdrop,
    spawn_demo_camera, spawn_overlay, write_overlay,
};

#[derive(Component)]
struct Walker;

#[derive(Component)]
struct Overlay;

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet basic");
    support::install_pane(&mut app);
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, (drive_walker, update_overlay));
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
        "Walker",
        &atlas,
        library,
        AnimationTarget::state("idle"),
        Vec3::new(-220.0, -130.0, 0.0),
        7.0,
        Color::WHITE,
    );
    commands.entity(actor).insert(Walker);

    let overlay = spawn_overlay(&mut commands, "spritesheet basic");
    commands.entity(overlay).insert(Overlay);
}

fn drive_walker(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &mut AnimationController,
            &SpritesheetAnimator,
        ),
        With<Walker>,
    >,
) {
    let elapsed = time.elapsed_secs();
    let walk_weight = (elapsed * 0.9).sin();

    for (mut transform, mut controller, animator) in &mut query {
        if walk_weight.abs() > 0.25 {
            controller.set_target(AnimationTarget::state("walk"));
            transform.translation.x = walk_weight * 240.0;
            transform.scale.x = if walk_weight >= 0.0 { 7.0 } else { -7.0 };
        } else {
            controller.set_target(AnimationTarget::state("idle"));
            transform.translation.x = walk_weight * 80.0;
            transform.scale.x =
                if animator.current_clip.as_ref().map(|clip| clip.as_str()) == Some("walk_clip") {
                    transform.scale.x.signum() * 7.0
                } else {
                    7.0
                };
        }
    }
}

fn update_overlay(
    walker: Single<&saddle_animation_spritesheet::SpritesheetAnimator, With<Walker>>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    let animator = *walker;
    write_overlay(
        &mut text,
        "spritesheet basic",
        format!(
            "A single actor swaps between idle and walk based on a simple motion signal.\nCurrent clip: {}\nFrame: {}  Atlas index: {}\nPlayback: {:?}",
            animator
                .current_clip
                .as_ref()
                .map(|clip| clip.as_str())
                .unwrap_or("none"),
            animator.current_frame,
            animator.atlas_index,
            animator.playback_state,
        ),
    );
}
