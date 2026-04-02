use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationController, AnimationTarget, SpritesheetAnimator, SpritesheetPlugin, StartOffset,
};
use support::{
    apply_example_defaults, main_library, make_demo_atlas, spawn_actor, spawn_demo_backdrop,
    spawn_demo_camera, spawn_overlay, write_overlay,
};

#[derive(Component)]
struct CrowdMember;

#[derive(Component)]
struct Overlay;

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet crowd variation");
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, update_overlay);
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

    for (index, x) in (-5..=5).enumerate() {
        let speed = 0.78 + index as f32 * 0.05;
        let tint = Color::srgb(
            0.82 + index as f32 * 0.01,
            0.9 - index as f32 * 0.02,
            1.0 - index as f32 * 0.03,
        );
        let entity = spawn_actor(
            &mut commands,
            &format!("Crowd Member {}", index + 1),
            &atlas,
            library.clone(),
            AnimationTarget::state("walk"),
            Vec3::new(x as f32 * 95.0, -110.0, 0.0),
            6.0,
            tint,
        );
        commands.entity(entity).insert((
            CrowdMember,
            AnimationController {
                default_target: Some(AnimationTarget::state("walk")),
                start_offset: StartOffset::EntitySeeded,
                ..default()
            },
            SpritesheetAnimator::default().with_speed(speed),
        ));
    }

    let overlay = spawn_overlay(&mut commands, "spritesheet crowd variation");
    commands.entity(overlay).insert(Overlay);
}

fn update_overlay(
    crowd: Query<&saddle_animation_spritesheet::SpritesheetAnimator, With<CrowdMember>>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    let mut min_progress: f32 = 1.0;
    let mut max_progress: f32 = 0.0;
    let mut avg_speed: f32 = 0.0;
    let mut count: f32 = 0.0;

    for animator in &crowd {
        min_progress = min_progress.min(animator.normalized_time);
        max_progress = max_progress.max(animator.normalized_time);
        avg_speed += animator.speed_multiplier;
        count += 1.0;
    }

    if count > 0.0 {
        avg_speed /= count;
    }

    write_overlay(
        &mut text,
        "spritesheet crowd variation",
        format!(
            "Every actor shares the same library and atlas, but start offsets and speed multipliers prevent lockstep motion.\nEntities: {}\nProgress span: {:.2} .. {:.2}\nAverage speed multiplier: {:.2}",
            count as u32, min_progress, max_progress, avg_speed,
        ),
    );
}
