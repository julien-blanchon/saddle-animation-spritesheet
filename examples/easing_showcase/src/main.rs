use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationClip, AnimationLibrary, AnimationTarget, Easing, EasingVariety, FrameTiming,
    SpritesheetAnimationBundle, SpritesheetAnimator, SpritesheetPlugin,
};
use support::{
    DemoAtlas, apply_example_defaults, load_gabe_atlas, spawn_demo_backdrop, spawn_demo_camera,
    spawn_overlay, write_overlay,
};

#[derive(Component)]
struct EasingActor {
    label: &'static str,
}

#[derive(Component)]
struct Overlay;

struct EasingEntry {
    label: &'static str,
    easing: Easing,
}

fn easing_entries() -> Vec<EasingEntry> {
    vec![
        EasingEntry {
            label: "Linear",
            easing: Easing::Linear,
        },
        EasingEntry {
            label: "In Quadratic",
            easing: Easing::In(EasingVariety::Quadratic),
        },
        EasingEntry {
            label: "Out Quadratic",
            easing: Easing::Out(EasingVariety::Quadratic),
        },
        EasingEntry {
            label: "InOut Cubic",
            easing: Easing::InOut(EasingVariety::Cubic),
        },
        EasingEntry {
            label: "In Exponential",
            easing: Easing::In(EasingVariety::Exponential),
        },
        EasingEntry {
            label: "Out Circular",
            easing: Easing::Out(EasingVariety::Circular),
        },
        EasingEntry {
            label: "InOut Sin",
            easing: Easing::InOut(EasingVariety::Sin),
        },
        EasingEntry {
            label: "In Quintic",
            easing: Easing::In(EasingVariety::Quintic),
        },
    ]
}

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet easing showcase");
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
    mut libraries: ResMut<Assets<AnimationLibrary>>,
) {
    spawn_demo_camera(&mut commands);
    spawn_demo_backdrop(&mut commands);

    let atlas = load_gabe_atlas(&asset_server, &mut layouts);
    let entries = easing_entries();
    let row_count = entries.len();
    let start_y = 280.0;
    let row_spacing = 72.0;

    for (index, entry) in entries.into_iter().enumerate() {
        let y = start_y - index as f32 * row_spacing;

        let library = libraries.add(
            AnimationLibrary::new(format!("easing_{}", entry.label))
                .with_default_target(AnimationTarget::clip("run"))
                .add_clip(
                    AnimationClip::from_indices("run", [2, 3, 4, 5, 6])
                        .with_timing(FrameTiming::FramesPerSecond(6.0))
                        .with_easing(entry.easing),
                ),
        );

        commands.spawn((
            Name::new(format!("Easing Label: {}", entry.label)),
            Node {
                position_type: PositionType::Absolute,
                left: px(490.0),
                top: px(start_y - y + 158.0),
                ..default()
            },
            Text::new(entry.label.to_string()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.9, 0.7, 0.9)),
        ));

        spawn_easing_actor(
            &mut commands,
            &atlas,
            library,
            entry.label,
            Vec3::new(-480.0, y, 0.0),
        );
    }

    commands.spawn((
        Name::new("Row Count Label"),
        Node {
            position_type: PositionType::Absolute,
            left: px(490.0),
            top: px(start_y - (row_count as f32 * row_spacing) + 158.0 + row_spacing),
            ..default()
        },
        Text::new(format!("{row_count} easing curves compared")),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(0.7, 0.7, 0.7, 0.7)),
    ));

    let overlay = spawn_overlay(&mut commands, "spritesheet easing showcase");
    commands.entity(overlay).insert(Overlay);
}

fn spawn_easing_actor(
    commands: &mut Commands,
    atlas: &DemoAtlas,
    library: Handle<AnimationLibrary>,
    label: &'static str,
    translation: Vec3,
) {
    commands.spawn((
        Name::new(format!("Easing Actor: {label}")),
        EasingActor { label },
        Sprite::from_atlas_image(
            atlas.image.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: 0,
            },
        ),
        Transform::from_translation(translation).with_scale(Vec3::splat(5.5)),
        SpritesheetAnimationBundle::new(library, AnimationTarget::clip("run")),
    ));
}

fn update_overlay(
    actors: Query<(&SpritesheetAnimator, &EasingActor)>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    let mut lines = String::from(
        "Each row runs the same 5-frame run clip with a different easing curve.\n\
         Easing redistributes time across frames without interpolating between them.\n",
    );

    for (animator, actor) in &actors {
        lines.push_str(&format!(
            "\n{:<18} frame {} t={:.2}",
            actor.label, animator.current_frame, animator.normalized_time,
        ));
    }

    write_overlay(&mut text, "spritesheet easing showcase", &lines);
}
