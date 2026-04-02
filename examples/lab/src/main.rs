#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use saddle_animation_spritesheet_example_support as support;

use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_animation_spritesheet::{
    AnimationChanged, AnimationController, AnimationEventFired, AnimationLooped, AnimationTarget,
    SpritesheetAnimator, SpritesheetPlugin, StartOffset,
};
use support::{
    apply_example_defaults, main_library, make_demo_atlas, prop_library, spawn_actor,
    spawn_demo_backdrop, spawn_demo_camera, spawn_overlay, write_overlay,
};

#[derive(Component)]
pub struct LabHero;

#[derive(Component)]
pub struct LabProp;

#[derive(Component)]
pub struct CrowdMember;

#[derive(Component)]
struct Overlay;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum HeroMode {
    #[default]
    Idle,
    Walk,
    UseTool,
}

impl HeroMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Walk => "walk",
            Self::UseTool => "use_tool",
        }
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct LabControl {
    pub auto: bool,
    pub requested_mode: HeroMode,
    pub applied_mode: Option<HeroMode>,
}

impl Default for LabControl {
    fn default() -> Self {
        Self {
            auto: true,
            requested_mode: HeroMode::Idle,
            applied_mode: None,
        }
    }
}

impl LabControl {
    pub(crate) fn request(&mut self, mode: HeroMode) {
        self.requested_mode = mode;
        self.applied_mode = None;
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct LabDiagnostics {
    pub hero_clip: String,
    pub hero_state: String,
    pub hero_frame: usize,
    pub hero_playing: bool,
    pub impact_events: u32,
    pub last_event: String,
    pub last_event_frame: usize,
    pub prop_loops: u32,
    pub crowd_phase_span: f32,
    pub requested_mode: String,
}

impl Default for LabDiagnostics {
    fn default() -> Self {
        Self {
            hero_clip: String::new(),
            hero_state: String::new(),
            hero_frame: 0,
            hero_playing: false,
            impact_events: 0,
            last_event: String::new(),
            last_event_frame: 0,
            prop_loops: 0,
            crowd_phase_span: 0.0,
            requested_mode: HeroMode::Idle.as_str().into(),
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct LabEntities {
    hero: Entity,
    prop: Entity,
}

fn main() {
    let mut app = App::new();
    apply_example_defaults(&mut app, "spritesheet crate-local lab");
    app.insert_resource(LabControl::default());
    app.init_resource::<LabDiagnostics>();
    app.register_type::<LabControl>();
    app.register_type::<LabDiagnostics>();
    #[cfg(feature = "dev")]
    app.add_plugins(RemotePlugin::default());
    #[cfg(feature = "dev")]
    app.add_plugins(BrpExtrasPlugin::with_http_plugin(
        RemoteHttpPlugin::default().with_port(lab_brp_port()),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::SpritesheetLabE2EPlugin);
    app.add_plugins(SpritesheetPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            drive_auto_mode,
            apply_control_mode,
            settle_manual_one_shot,
            animate_hero_motion,
            record_messages,
            refresh_diagnostics,
            update_overlay,
        ),
    );
    app.run();
}

#[cfg(feature = "dev")]
fn lab_brp_port() -> u16 {
    std::env::var("BRP_EXTRAS_PORT")
        .or_else(|_| std::env::var("BRP_PORT"))
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(15_712)
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut libraries: ResMut<Assets<saddle_animation_spritesheet::AnimationLibrary>>,
) {
    spawn_demo_camera(&mut commands);
    spawn_demo_backdrop(&mut commands);

    commands.spawn((
        Name::new("Hero Panel"),
        Sprite::from_color(Color::srgba(0.99, 0.66, 0.2, 0.08), Vec2::new(420.0, 360.0)),
        Transform::from_xyz(-250.0, -80.0, -10.0),
    ));
    commands.spawn((
        Name::new("Prop Panel"),
        Sprite::from_color(Color::srgba(0.2, 0.78, 0.96, 0.08), Vec2::new(300.0, 280.0)),
        Transform::from_xyz(300.0, -110.0, -10.0),
    ));

    let atlas = make_demo_atlas(&mut images, &mut layouts);
    let main_library = libraries.add(main_library());
    let prop_library = libraries.add(prop_library());

    let hero = spawn_actor(
        &mut commands,
        "Lab Hero",
        &atlas,
        main_library.clone(),
        AnimationTarget::state("idle"),
        Vec3::new(-250.0, -135.0, 0.0),
        8.0,
        Color::WHITE,
    );
    commands.entity(hero).insert(LabHero);

    let prop = spawn_actor(
        &mut commands,
        "Animated Prop",
        &atlas,
        prop_library,
        AnimationTarget::state("loop"),
        Vec3::new(300.0, -110.0, 0.0),
        7.0,
        Color::srgb(0.95, 0.98, 1.0),
    );
    commands.entity(prop).insert(LabProp);

    for (index, x) in (-4..=4).enumerate() {
        let entity = spawn_actor(
            &mut commands,
            &format!("Crowd Member {}", index + 1),
            &atlas,
            main_library.clone(),
            AnimationTarget::state("walk"),
            Vec3::new(x as f32 * 88.0, 120.0, 0.0),
            5.2,
            Color::srgb(0.92, 0.96 - index as f32 * 0.03, 1.0 - index as f32 * 0.05),
        );
        commands.entity(entity).insert((
            CrowdMember,
            AnimationController {
                default_target: Some(AnimationTarget::state("walk")),
                start_offset: StartOffset::EntitySeeded,
                ..default()
            },
            SpritesheetAnimator::default().with_speed(0.8 + index as f32 * 0.06),
        ));
    }

    let overlay = spawn_overlay(&mut commands, "spritesheet crate-local lab");
    commands.entity(overlay).insert(Overlay);
    commands.insert_resource(LabEntities { hero, prop });
}

fn drive_auto_mode(time: Res<Time>, mut control: ResMut<LabControl>) {
    if !control.auto {
        return;
    }

    let phase = time.elapsed_secs().rem_euclid(6.0);
    let desired = if phase < 1.5 {
        HeroMode::Idle
    } else if phase < 3.7 {
        HeroMode::Walk
    } else if phase < 4.2 {
        HeroMode::UseTool
    } else {
        HeroMode::Idle
    };

    if control.requested_mode != desired {
        control.request(desired);
    }
}

fn apply_control_mode(
    mut control: ResMut<LabControl>,
    mut hero: Single<&mut AnimationController, With<LabHero>>,
) {
    if control.applied_mode == Some(control.requested_mode) {
        return;
    }

    match control.requested_mode {
        HeroMode::Idle => hero.set_target(AnimationTarget::state("idle")),
        HeroMode::Walk => hero.set_target(AnimationTarget::state("walk")),
        HeroMode::UseTool => {
            hero.play_state_once("use_tool");
            hero.requested_target = Some(AnimationTarget::state("idle"));
        }
    }

    control.applied_mode = Some(control.requested_mode);
}

fn settle_manual_one_shot(
    mut control: ResMut<LabControl>,
    hero: Single<&saddle_animation_spritesheet::SpritesheetAnimator, With<LabHero>>,
) {
    if control.auto || control.requested_mode != HeroMode::UseTool {
        return;
    }

    if hero.current_state.as_ref().map(|state| state.as_str()) != Some("use_tool")
        && control.applied_mode == Some(HeroMode::UseTool)
    {
        control.request(HeroMode::Idle);
    }
}

fn animate_hero_motion(
    time: Res<Time>,
    control: Res<LabControl>,
    mut hero: Single<&mut Transform, With<LabHero>>,
) {
    if control.requested_mode == HeroMode::Walk {
        let swing = (time.elapsed_secs() * 1.1).sin();
        hero.translation.x = -250.0 + swing * 130.0;
        hero.scale.x = if swing >= 0.0 { 8.0 } else { -8.0 };
    } else {
        hero.translation.x = -250.0;
        hero.scale.x = 8.0;
    }
}

fn record_messages(
    entities: Res<LabEntities>,
    mut diagnostics: ResMut<LabDiagnostics>,
    mut changed: MessageReader<AnimationChanged>,
    mut events: MessageReader<AnimationEventFired>,
    mut looped: MessageReader<AnimationLooped>,
) {
    for message in changed.read() {
        if message.entity == entities.hero {
            diagnostics.hero_clip = message.clip.as_str().to_string();
        }
    }

    for message in events.read() {
        if message.entity == entities.hero && message.marker.name == "impact" {
            diagnostics.impact_events += 1;
            diagnostics.last_event = message.marker.name.clone();
            diagnostics.last_event_frame = message.frame;
        }
    }

    for message in looped.read() {
        if message.entity == entities.prop {
            diagnostics.prop_loops = message.completed_loops;
        }
    }
}

fn refresh_diagnostics(
    control: Res<LabControl>,
    mut diagnostics: ResMut<LabDiagnostics>,
    hero: Single<&saddle_animation_spritesheet::SpritesheetAnimator, With<LabHero>>,
    crowd: Query<&saddle_animation_spritesheet::SpritesheetAnimator, With<CrowdMember>>,
) {
    diagnostics.hero_clip = hero
        .current_clip
        .as_ref()
        .map(|clip| clip.as_str().to_string())
        .unwrap_or_default();
    diagnostics.hero_state = hero
        .current_state
        .as_ref()
        .map(|state| state.as_str().to_string())
        .unwrap_or_default();
    diagnostics.hero_frame = hero.current_frame;
    diagnostics.hero_playing = hero.playback_state == saddle_animation_spritesheet::PlaybackState::Playing;
    diagnostics.requested_mode = control.requested_mode.as_str().to_string();

    let mut min_progress: f32 = 1.0;
    let mut max_progress: f32 = 0.0;
    for animator in &crowd {
        min_progress = min_progress.min(animator.normalized_time);
        max_progress = max_progress.max(animator.normalized_time);
    }
    diagnostics.crowd_phase_span = if max_progress >= min_progress {
        max_progress - min_progress
    } else {
        0.0
    };
}

fn update_overlay(diagnostics: Res<LabDiagnostics>, mut text: Single<&mut Text, With<Overlay>>) {
    write_overlay(
        &mut text,
        "spritesheet crate-local lab",
        format!(
            "Auto-driving hero, looping prop, and crowd-variation strip in one BRP-friendly scene.\nHero clip/state: {} / {}\nHero frame: {}  Playing: {}\nRequested mode: {}\nImpact events: {}  Last event: {} @ frame {}\nProp loops: {}\nCrowd phase span: {:.2}",
            diagnostics.hero_clip,
            diagnostics.hero_state,
            diagnostics.hero_frame,
            diagnostics.hero_playing,
            diagnostics.requested_mode,
            diagnostics.impact_events,
            diagnostics.last_event,
            diagnostics.last_event_frame,
            diagnostics.prop_loops,
            diagnostics.crowd_phase_span,
        ),
    );
}
