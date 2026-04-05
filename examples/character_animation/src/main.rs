use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationController, AnimationTarget, SpritesheetAnimator, SpritesheetPlugin,
};
use support::{
    apply_example_defaults, gabe_library, load_gabe_atlas, load_mani_atlas, mani_library,
    spawn_actor, spawn_demo_backdrop, spawn_demo_camera, spawn_overlay, write_overlay,
};

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Companion;

#[derive(Component)]
struct Overlay;

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet character animation");
    support::install_pane(&mut app);
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, (drive_player, drive_companion, update_overlay));
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

    let gabe_atlas = load_gabe_atlas(&asset_server, &mut layouts);
    let mani_atlas = load_mani_atlas(&asset_server, &mut layouts);
    let gabe_lib = libraries.add(gabe_library());
    let mani_lib = libraries.add(mani_library());

    let player = spawn_actor(
        &mut commands,
        "Player (Gabe)",
        &gabe_atlas,
        gabe_lib,
        AnimationTarget::state("idle"),
        Vec3::new(-160.0, -130.0, 0.0),
        7.0,
        Color::WHITE,
    );
    commands.entity(player).insert(Player);

    let companion = spawn_actor(
        &mut commands,
        "Companion (Mani)",
        &mani_atlas,
        mani_lib,
        AnimationTarget::state("idle"),
        Vec3::new(120.0, -130.0, 0.0),
        7.0,
        Color::WHITE,
    );
    commands.entity(companion).insert(Companion);

    let overlay = spawn_overlay(&mut commands, "spritesheet character animation");
    commands.entity(overlay).insert(Overlay);
}

fn drive_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut AnimationController), With<Player>>,
) {
    let speed = 3.0;
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

    for (mut transform, mut controller) in &mut query {
        if direction.length_squared() > 0.01 {
            let movement = direction.normalize() * speed;
            transform.translation.x += movement.x;
            transform.translation.y += movement.y;
            controller.set_target(AnimationTarget::state("run"));
            if direction.x != 0.0 {
                transform.scale.x = if direction.x > 0.0 { 7.0 } else { -7.0 };
            }
        } else {
            controller.set_target(AnimationTarget::state("idle"));
        }

        if keyboard.just_pressed(KeyCode::Space) {
            controller.play_state_once("action");
            controller.requested_target = Some(AnimationTarget::state("idle"));
        }
    }
}

fn drive_companion(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut AnimationController), With<Companion>>,
) {
    let elapsed = time.elapsed_secs();
    let walk_weight = (elapsed * 0.7).sin();

    for (mut transform, mut controller) in &mut query {
        if walk_weight.abs() > 0.3 {
            controller.set_target(AnimationTarget::state("run"));
            transform.translation.x = 120.0 + walk_weight * 180.0;
            transform.scale.x = if walk_weight >= 0.0 { 7.0 } else { -7.0 };
        } else {
            controller.set_target(AnimationTarget::state("idle"));
        }
    }
}

fn update_overlay(
    player: Single<&SpritesheetAnimator, With<Player>>,
    companion: Single<&SpritesheetAnimator, (With<Companion>, Without<Player>)>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    let p = *player;
    let c = *companion;
    write_overlay(
        &mut text,
        "spritesheet character animation",
        format!(
            "Arrow keys / WASD to move Gabe. Space to trigger action.\nMani patrols automatically.\n\nGabe: {}  frame {}  {:?}\nMani: {}  frame {}  {:?}",
            p.current_clip
                .as_ref()
                .map(|c| c.as_str())
                .unwrap_or("none"),
            p.current_frame,
            p.playback_state,
            c.current_clip
                .as_ref()
                .map(|c| c.as_str())
                .unwrap_or("none"),
            c.current_frame,
            c.playback_state,
        ),
    );
}
