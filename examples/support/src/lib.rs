use std::{thread, time::Duration};

use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    winit::WinitSettings,
};
use saddle_animation_spritesheet::{
    AnimationClip, AnimationController, AnimationEventMarker, AnimationLibrary, AnimationState,
    AnimationTarget, AnimationTickPolicy, ClipFrame, FrameTiming, InterruptPolicy,
    PendingRequestPolicy, PlaybackDirection, RepeatMode, SameTargetPolicy,
    SpritesheetAnimationBundle, SpritesheetAnimator,
};
use saddle_pane::prelude::*;

const AUTO_EXIT_ENV: &str = "SPRITESHEET_AUTO_EXIT_SECONDS";
const FRAME_SIZE: UVec2 = UVec2::new(24, 24);

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Clone)]
pub struct DemoAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Spritesheet", position = "top-right")]
pub struct ExampleSpritesheetPane {
    #[pane(slider, min = 0.25, max = 3.0, step = 0.05)]
    pub speed_multiplier: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 1.0)]
    pub visibility_policy_index: f32,
    #[pane(slider, min = 0.0, max = 2.0, step = 1.0)]
    pub pending_request_policy_index: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 1.0)]
    pub same_target_policy_index: f32,
    #[pane(monitor)]
    pub current_frame: f32,
    #[pane(monitor)]
    pub atlas_index: f32,
    #[pane(monitor)]
    pub completed_loops: f32,
    #[pane(monitor)]
    pub current_clip: String,
    #[pane(monitor)]
    pub playback_state: String,
}

impl Default for ExampleSpritesheetPane {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            visibility_policy_index: 0.0,
            pending_request_policy_index: 0.0,
            same_target_policy_index: 0.0,
            current_frame: 0.0,
            atlas_index: 0.0,
            completed_loops: 0.0,
            current_clip: "none".to_string(),
            playback_state: "Stopped".to_string(),
        }
    }
}

pub fn apply_example_defaults(app: &mut App, title: &str) {
    app.insert_resource(ClearColor(Color::srgb(0.095, 0.11, 0.13)));
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: title.into(),
                    resolution: (1360, 820).into(),
                    ..default()
                }),
                ..default()
            }),
    );

    if let Some(seconds) = std::env::var(AUTO_EXIT_ENV)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .map(|value| value.max(0.1))
    {
        app.insert_resource(WinitSettings::continuous());
        app.insert_resource(AutoExitAfter(Timer::from_seconds(seconds, TimerMode::Once)));
        app.add_systems(Update, auto_exit_after);

        thread::spawn(move || {
            thread::sleep(Duration::from_secs_f32(seconds + 0.25));
            std::process::exit(0);
        });
    }
}

pub fn install_pane(app: &mut App) {
    if !app.is_plugin_added::<PanePlugin>() {
        app.add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ));
    }

    app.register_pane::<ExampleSpritesheetPane>().add_systems(
        Update,
        (sync_example_pane, update_example_pane_monitors).chain(),
    );
}

pub fn spawn_demo_camera(commands: &mut Commands) {
    commands.spawn((
        Name::new("Demo Camera"),
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 500.0),
    ));
}

pub fn spawn_demo_backdrop(commands: &mut Commands) {
    commands.spawn((
        Name::new("Backdrop"),
        Sprite::from_color(Color::srgb(0.13, 0.16, 0.19), Vec2::new(1600.0, 900.0)),
        Transform::from_xyz(0.0, 0.0, -30.0),
    ));
    commands.spawn((
        Name::new("Stage Glow"),
        Sprite {
            color: Color::srgba(0.95, 0.55, 0.18, 0.16),
            custom_size: Some(Vec2::new(720.0, 320.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -40.0, -25.0),
    ));
    commands.spawn((
        Name::new("Floor"),
        Sprite::from_color(Color::srgb(0.2, 0.19, 0.17), Vec2::new(1600.0, 180.0)),
        Transform::from_xyz(0.0, -250.0, -20.0),
    ));
}

pub fn spawn_overlay(commands: &mut Commands, title: &str) -> Entity {
    commands
        .spawn((
            Name::new("Overlay"),
            Node {
                position_type: PositionType::Absolute,
                left: px(18.0),
                top: px(18.0),
                width: px(460.0),
                padding: UiRect::all(px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.8)),
            Text::new(title.to_string()),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id()
}

pub fn write_overlay(text: &mut Text, title: &str, body: impl AsRef<str>) {
    *text = Text::new(format!("{title}\n\n{}", body.as_ref()));
}

pub fn make_demo_atlas(
    images: &mut Assets<Image>,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> DemoAtlas {
    let frame_colors: [[u8; 4]; 8] = [
        [244, 191, 117, 255],
        [247, 144, 96, 255],
        [123, 201, 176, 255],
        [84, 160, 212, 255],
        [110, 117, 240, 255],
        [252, 236, 97, 255],
        [255, 115, 87, 255],
        [248, 248, 248, 255],
    ];
    let width = FRAME_SIZE.x * frame_colors.len() as u32;
    let height = FRAME_SIZE.y;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for (frame_index, color) in frame_colors.iter().enumerate() {
        let frame_origin = frame_index as u32 * FRAME_SIZE.x;
        for y in 0..FRAME_SIZE.y {
            for x in 0..FRAME_SIZE.x {
                let gx = frame_origin + x;
                let offset = ((y * width + gx) * 4) as usize;
                let border = x < 2 || y < 2 || x >= FRAME_SIZE.x - 2 || y >= FRAME_SIZE.y - 2;
                let checker = ((x / 4) + (y / 4) + frame_index as u32).is_multiple_of(2);
                let eye_band = y > 7 && y < 12 && x > 5 && x < 18;

                let (r, g, b) = if border {
                    (20, 20, 22)
                } else if eye_band && checker {
                    (
                        color[0].saturating_sub(60),
                        color[1].saturating_sub(40),
                        color[2].saturating_sub(20),
                    )
                } else if checker {
                    (
                        color[0].saturating_sub(18),
                        color[1].saturating_sub(18),
                        color[2].saturating_sub(18),
                    )
                } else {
                    (color[0], color[1], color[2])
                };

                data[offset] = r;
                data[offset + 1] = g;
                data[offset + 2] = b;
                data[offset + 3] = 255;
            }
        }
    }

    let image = images.add(Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        FRAME_SIZE,
        frame_colors.len() as u32,
        1,
        None,
        None,
    ));

    DemoAtlas { image, layout }
}

#[allow(dead_code)]
pub fn main_library() -> AnimationLibrary {
    AnimationLibrary::new("demo_main")
        .with_default_target(AnimationTarget::state("idle"))
        .add_clip(
            AnimationClip::from_indices("idle_clip", [0, 1])
                .with_timing(FrameTiming::SecondsPerFrame(0.3)),
        )
        .add_clip(
            AnimationClip::from_indices("walk_clip", [2, 3, 4, 3])
                .with_timing(FrameTiming::FramesPerSecond(8.0)),
        )
        .add_clip(
            AnimationClip::from_frames(
                "use_tool_clip",
                [
                    ClipFrame::new(5),
                    ClipFrame::new(6).with_event(AnimationEventMarker::named("impact")),
                    ClipFrame::new(7),
                ],
            )
            .with_timing(FrameTiming::SecondsPerFrame(0.12))
            .with_repeat(RepeatMode::Once)
            .with_interrupt_policy(InterruptPolicy::LockUntilFinished),
        )
        .add_state(AnimationState::new("idle", "idle_clip"))
        .add_state(AnimationState::new("walk", "walk_clip"))
        .add_state(AnimationState::new("use_tool", "use_tool_clip"))
        .add_transition(
            saddle_animation_spritesheet::TransitionDefinition::finished(
                saddle_animation_spritesheet::TransitionSource::State("use_tool".into()),
                AnimationTarget::state("idle"),
            ),
        )
}

#[allow(dead_code)]
pub fn prop_library() -> AnimationLibrary {
    AnimationLibrary::new("demo_prop")
        .with_default_target(AnimationTarget::state("loop"))
        .add_clip(
            AnimationClip::from_indices("prop_clip", [1, 2, 3, 2])
                .with_timing(FrameTiming::FramesPerSecond(6.0))
                .with_direction(PlaybackDirection::PingPong),
        )
        .add_state(AnimationState::new("loop", "prop_clip"))
}

/// Load the Gabe idle-run sprite sheet (7 frames of 24x24).
#[allow(dead_code)]
pub fn load_gabe_atlas(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> DemoAtlas {
    let image = asset_server.load("gabe-idle-run.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(24, 24),
        7,
        1,
        None,
        None,
    ));
    DemoAtlas { image, layout }
}

/// Load the Mani idle-run sprite sheet (7 frames of 24x24).
#[allow(dead_code)]
pub fn load_mani_atlas(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> DemoAtlas {
    let image = asset_server.load("mani-idle-run.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(24, 24),
        7,
        1,
        None,
        None,
    ));
    DemoAtlas { image, layout }
}

/// Load the Kenney character sprite sheet (8x4 grid of 48x48).
#[allow(dead_code)]
pub fn load_kenney_character_atlas(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> DemoAtlas {
    let image = asset_server.load("kenney-character.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(48, 48),
        8,
        4,
        None,
        None,
    ));
    DemoAtlas { image, layout }
}

/// Load the Kenney character directional sprite sheet (4x4 grid of 24x24).
/// Rows: 0=down, 1=right, 2=up, 3=left.  Columns: 4 walk frames.
#[allow(dead_code)]
pub fn load_kenney_directional_atlas(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> DemoAtlas {
    let image = asset_server.load("kenney-character-directional.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(24, 24),
        4,
        4,
        None,
        None,
    ));
    DemoAtlas { image, layout }
}

/// Load the Kenney tiny dungeon tile sheet (12x11 grid of 16x16).
#[allow(dead_code)]
pub fn load_kenney_dungeon_atlas(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> DemoAtlas {
    let image = asset_server.load("kenney-tiny-dungeon.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        12,
        11,
        None,
        None,
    ));
    DemoAtlas { image, layout }
}

/// Animation library for Gabe sprite: idle (frames 0-1), run (frames 2-6),
/// action one-shot with impact marker (frames 4-6).
#[allow(dead_code)]
pub fn gabe_library() -> AnimationLibrary {
    AnimationLibrary::new("gabe")
        .with_default_target(AnimationTarget::state("idle"))
        .add_clip(
            AnimationClip::from_indices("idle_clip", [0, 1])
                .with_timing(FrameTiming::SecondsPerFrame(0.3)),
        )
        .add_clip(
            AnimationClip::from_indices("run_clip", [2, 3, 4, 5, 6])
                .with_timing(FrameTiming::FramesPerSecond(10.0)),
        )
        .add_clip(
            AnimationClip::from_frames(
                "action_clip",
                [
                    ClipFrame::new(4),
                    ClipFrame::new(5).with_event(AnimationEventMarker::named("impact")),
                    ClipFrame::new(6),
                ],
            )
            .with_timing(FrameTiming::SecondsPerFrame(0.12))
            .with_repeat(RepeatMode::Once)
            .with_interrupt_policy(InterruptPolicy::LockUntilFinished),
        )
        .add_state(AnimationState::new("idle", "idle_clip"))
        .add_state(AnimationState::new("run", "run_clip"))
        .add_state(AnimationState::new("action", "action_clip"))
        .add_transition(
            saddle_animation_spritesheet::TransitionDefinition::finished(
                saddle_animation_spritesheet::TransitionSource::State("action".into()),
                AnimationTarget::state("idle"),
            ),
        )
}

/// Animation library for Mani sprite: same layout as Gabe.
#[allow(dead_code)]
pub fn mani_library() -> AnimationLibrary {
    AnimationLibrary::new("mani")
        .with_default_target(AnimationTarget::state("idle"))
        .add_clip(
            AnimationClip::from_indices("idle_clip", [0, 1])
                .with_timing(FrameTiming::SecondsPerFrame(0.35)),
        )
        .add_clip(
            AnimationClip::from_indices("run_clip", [2, 3, 4, 5, 6])
                .with_timing(FrameTiming::FramesPerSecond(10.0)),
        )
        .add_state(AnimationState::new("idle", "idle_clip"))
        .add_state(AnimationState::new("run", "run_clip"))
}

/// Animation library for Kenney character with 4-directional walk using rows.
/// Row 0 = down, row 1 = right, row 2 = up, row 3 = left (4 columns per row).
#[allow(dead_code)]
pub fn kenney_directional_library() -> AnimationLibrary {
    AnimationLibrary::new("kenney_directional")
        .with_default_target(AnimationTarget::state("down"))
        .add_clip(
            AnimationClip::from_row("down_clip", 4, 0, 4)
                .with_timing(FrameTiming::SecondsPerFrame(0.18)),
        )
        .add_clip(
            AnimationClip::from_row("right_clip", 4, 1, 4)
                .with_timing(FrameTiming::SecondsPerFrame(0.18)),
        )
        .add_clip(
            AnimationClip::from_row("up_clip", 4, 2, 4)
                .with_timing(FrameTiming::SecondsPerFrame(0.18)),
        )
        .add_clip(
            AnimationClip::from_row("left_clip", 4, 3, 4)
                .with_timing(FrameTiming::SecondsPerFrame(0.18)),
        )
        .add_state(AnimationState::new("down", "down_clip"))
        .add_state(AnimationState::new("right", "right_clip"))
        .add_state(AnimationState::new("up", "up_clip"))
        .add_state(AnimationState::new("left", "left_clip"))
}

pub fn spawn_actor(
    commands: &mut Commands,
    name: &str,
    atlas: &DemoAtlas,
    library: Handle<AnimationLibrary>,
    default_target: AnimationTarget,
    translation: Vec3,
    scale: f32,
    tint: Color,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_string()),
            Sprite {
                color: tint,
                ..Sprite::from_atlas_image(
                    atlas.image.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: 0,
                    },
                )
            },
            Transform::from_translation(translation).with_scale(Vec3::splat(scale)),
            SpritesheetAnimationBundle::new(library, default_target),
        ))
        .id()
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}

fn sync_example_pane(
    pane: Res<ExampleSpritesheetPane>,
    mut controllers: Query<&mut AnimationController>,
    mut animators: Query<&mut SpritesheetAnimator>,
) {
    let visibility_policy = match pane.visibility_policy_index.round() as i32 {
        1 => AnimationTickPolicy::WhenVisible,
        _ => AnimationTickPolicy::Always,
    };
    let pending_request_policy = match pane.pending_request_policy_index.round() as i32 {
        1 => PendingRequestPolicy::KeepFirst,
        2 => PendingRequestPolicy::Discard,
        _ => PendingRequestPolicy::Replace,
    };
    let same_target_policy = match pane.same_target_policy_index.round() as i32 {
        1 => SameTargetPolicy::Restart,
        _ => SameTargetPolicy::KeepProgress,
    };

    for mut controller in &mut controllers {
        controller.pending_request_policy = pending_request_policy;
        controller.same_target_policy = same_target_policy;
    }

    for mut animator in &mut animators {
        animator.speed_multiplier = pane.speed_multiplier.max(0.05);
        animator.visibility_policy = visibility_policy;
    }
}

fn update_example_pane_monitors(
    mut pane: ResMut<ExampleSpritesheetPane>,
    animators: Query<&SpritesheetAnimator>,
) {
    let Some(animator) = animators.iter().next() else {
        return;
    };

    pane.current_frame = animator.current_frame as f32;
    pane.atlas_index = animator.atlas_index as f32;
    pane.completed_loops = animator.completed_loops as f32;
    pane.current_clip = animator
        .current_clip
        .as_ref()
        .map_or_else(|| "none".to_string(), |clip| clip.as_str().to_string());
    pane.playback_state = format!("{:?}", animator.playback_state);
}
