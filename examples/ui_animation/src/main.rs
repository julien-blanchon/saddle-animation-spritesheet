use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationClip, AnimationLibrary, AnimationState, AnimationTarget, FrameTiming,
    PlaybackDirection, SpritesheetAnimationBundle, SpritesheetAnimator, SpritesheetPlugin,
};
use support::{apply_example_defaults, load_kenney_dungeon_atlas, spawn_overlay, write_overlay};

#[derive(Component)]
struct AnimatedIcon {
    label: &'static str,
}

#[derive(Component)]
struct Overlay;

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet UI animation");
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
    commands.spawn((Name::new("UI Camera"), Camera2d));

    let atlas = load_kenney_dungeon_atlas(&asset_server, &mut layouts);

    // Row of animated UI icons in the center of the screen.
    // Kenney tiny dungeon tiles: 12 columns x 11 rows, 16x16 each.
    // We pick a few distinct tile ranges to animate as UI elements.

    let icon_configs: Vec<(&str, Vec<usize>, f32, PlaybackDirection)> = vec![
        ("Torches", vec![60, 61, 62], 4.0, PlaybackDirection::Forward),
        (
            "Potions",
            vec![48, 49, 50, 51],
            3.0,
            PlaybackDirection::PingPong,
        ),
        ("Skulls", vec![96, 97, 98], 2.5, PlaybackDirection::Forward),
        (
            "Chests",
            vec![84, 85, 86, 87],
            2.0,
            PlaybackDirection::PingPong,
        ),
        (
            "Characters",
            vec![108, 109, 110, 111],
            5.0,
            PlaybackDirection::Forward,
        ),
    ];

    // Container for the animated icons
    let container = commands
        .spawn((
            Name::new("Icon Container"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(32.0),
                ..default()
            },
        ))
        .id();

    // Title
    let title = commands
        .spawn((
            Name::new("Title"),
            Text::new("Animated UI Elements (ImageNode)"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.9, 0.7, 0.95)),
        ))
        .id();
    commands.entity(container).add_child(title);

    // Row container for the icons
    let row = commands
        .spawn((
            Name::new("Icon Row"),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(48.0),
                ..default()
            },
        ))
        .id();
    commands.entity(container).add_child(row);

    for (label, frames, fps, direction) in icon_configs {
        let library = libraries.add(
            AnimationLibrary::new(format!("ui_{label}"))
                .with_default_target(AnimationTarget::state("anim"))
                .add_clip(
                    AnimationClip::from_indices("clip", frames)
                        .with_timing(FrameTiming::FramesPerSecond(fps))
                        .with_direction(direction),
                )
                .add_state(AnimationState::new("anim", "clip")),
        );

        let icon_col = commands
            .spawn((
                Name::new(format!("Icon Column: {label}")),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(8.0),
                    ..default()
                },
            ))
            .id();

        let icon = commands
            .spawn((
                Name::new(format!("UI Icon: {label}")),
                AnimatedIcon { label },
                ImageNode {
                    image: atlas.image.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: 0,
                    }),
                    ..default()
                },
                Node {
                    width: px(80.0),
                    height: px(80.0),
                    ..default()
                },
                SpritesheetAnimationBundle::new(library, AnimationTarget::state("anim")),
            ))
            .id();
        commands.entity(icon_col).add_child(icon);

        let label_node = commands
            .spawn((
                Name::new(format!("Label: {label}")),
                Text::new(label.to_string()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.85)),
            ))
            .id();
        commands.entity(icon_col).add_child(label_node);

        commands.entity(row).add_child(icon_col);
    }

    // Instruction text
    let instructions = commands
        .spawn((
            Name::new("Instructions"),
            Text::new("Sprite sheets can drive UI elements via ImageNode, not just world Sprites."),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
        ))
        .id();
    commands.entity(container).add_child(instructions);

    let overlay = spawn_overlay(&mut commands, "spritesheet UI animation");
    commands.entity(overlay).insert(Overlay);
}

fn update_overlay(
    icons: Query<(&SpritesheetAnimator, &AnimatedIcon)>,
    mut text: Single<&mut Text, With<Overlay>>,
) {
    let mut body = String::from(
        "Animated UI ImageNode elements using sprite sheet clips.\n\
         Each icon cycles through dungeon tile frames.\n",
    );

    for (animator, icon) in &icons {
        body.push_str(&format!(
            "\n{:<14} frame {}  atlas {}  loops {}",
            icon.label, animator.current_frame, animator.atlas_index, animator.completed_loops,
        ));
    }

    write_overlay(&mut text, "spritesheet UI animation", &body);
}
