use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationController, AnimationTarget, SpritesheetAnimator, SpritesheetPlugin, StartOffset,
};
use support::{
    apply_example_defaults, gabe_library, load_gabe_atlas, load_mani_atlas, mani_library,
    spawn_actor, spawn_demo_backdrop, spawn_demo_camera, spawn_overlay, write_overlay,
};

#[derive(Component)]
struct CrowdMember;

#[derive(Component)]
struct Overlay;

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet crowd variation");
    support::install_pane(&mut app);
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, update_overlay);
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

    for (index, x) in (-5..=5).enumerate() {
        let speed = 0.78 + index as f32 * 0.05;
        let use_mani = index % 2 == 1;
        let atlas = if use_mani { &mani_atlas } else { &gabe_atlas };
        let library = if use_mani {
            mani_lib.clone()
        } else {
            gabe_lib.clone()
        };

        let entity = spawn_actor(
            &mut commands,
            &format!("Crowd Member {}", index + 1),
            atlas,
            library,
            AnimationTarget::state("run"),
            Vec3::new(x as f32 * 95.0, -110.0, 0.0),
            6.0,
            Color::WHITE,
        );
        commands.entity(entity).insert((
            CrowdMember,
            AnimationController {
                default_target: Some(AnimationTarget::state("run")),
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
            "Gabe and Mani alternate across the crowd. Entity-seeded offsets and speed multipliers prevent lockstep.\nEntities: {}\nProgress span: {:.2} .. {:.2}\nAverage speed multiplier: {:.2}",
            count as u32, min_progress, max_progress, avg_speed,
        ),
    );
}
