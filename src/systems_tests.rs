use std::time::Duration;

use bevy::{
    asset::{AssetEvent, AssetPlugin},
    ecs::message::{MessageCursor, Messages},
    image::{Image, TextureAtlas, TextureAtlasLayout},
    prelude::*,
    time::TimeUpdateStrategy,
};

use crate::{
    AnimationChanged, AnimationClip, AnimationControlCommand, AnimationController,
    AnimationEventFired, AnimationEventMarker, AnimationFinished, AnimationLibrary,
    AnimationLooped, AnimationState, AnimationTarget, ClipFrame, FrameTiming, InterruptPolicy,
    PendingRequestPolicy, PlaybackDirection, PlaybackState, RepeatMode, SpritesheetAnimationBundle,
    SpritesheetAnimationSource, SpritesheetAnimator, SpritesheetPlugin, StartOffset,
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        SpritesheetPlugin::default(),
    ))
    .init_asset::<TextureAtlasLayout>()
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app
}

fn update_with_delta(app: &mut App, delta: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(delta);
    app.update();
}

fn initialize_player(app: &mut App) {
    update_with_delta(app, Duration::ZERO);
}

fn read_messages<T: Message + Clone>(app: &App, cursor: &mut MessageCursor<T>) -> Vec<T> {
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

fn add_layout(app: &mut App, frame_count: usize) -> Handle<TextureAtlasLayout> {
    let columns = frame_count.max(1) as u32;
    app.world_mut()
        .resource_mut::<Assets<TextureAtlasLayout>>()
        .add(TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            columns,
            1,
            None,
            None,
        ))
}

fn add_library(app: &mut App, library: AnimationLibrary) -> Handle<AnimationLibrary> {
    app.world_mut()
        .resource_mut::<Assets<AnimationLibrary>>()
        .add(library)
}

fn spawn_sprite_player(
    app: &mut App,
    library: Handle<AnimationLibrary>,
    layout: Handle<TextureAtlasLayout>,
    default_target: AnimationTarget,
) -> Entity {
    app.world_mut()
        .spawn((
            Name::new("Animated Sprite"),
            Sprite::from_atlas_image(
                Handle::<Image>::default(),
                TextureAtlas { layout, index: 0 },
            ),
            SpritesheetAnimationBundle::new(library, default_target),
        ))
        .id()
}

fn spawn_headless_player(
    app: &mut App,
    library: Handle<AnimationLibrary>,
    default_target: AnimationTarget,
) -> Entity {
    app.world_mut()
        .spawn((
            Name::new("Headless Animator"),
            SpritesheetAnimationSource::new(library),
            AnimationController::default().with_default_target(default_target),
            SpritesheetAnimator::default(),
        ))
        .id()
}

fn animator(app: &App, entity: Entity) -> SpritesheetAnimator {
    app.world()
        .entity(entity)
        .get::<SpritesheetAnimator>()
        .expect("entity should have a SpritesheetAnimator")
        .clone()
}

fn controller(app: &App, entity: Entity) -> AnimationController {
    app.world()
        .entity(entity)
        .get::<AnimationController>()
        .expect("entity should have an AnimationController")
        .clone()
}

fn sprite_atlas_index(app: &App, entity: Entity) -> usize {
    app.world()
        .entity(entity)
        .get::<Sprite>()
        .and_then(|sprite| sprite.texture_atlas.as_ref())
        .map(|atlas| atlas.index)
        .expect("sprite should have a texture atlas")
}

fn set_requested_target(app: &mut App, entity: Entity, target: AnimationTarget) {
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<AnimationController>()
        .expect("entity should have an AnimationController")
        .set_target(target);
}

fn set_command(app: &mut App, entity: Entity, command: AnimationControlCommand) {
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<AnimationController>()
        .expect("entity should have an AnimationController")
        .command = command;
}

fn basic_state_library(
    clip: AnimationClip,
    state_id: &str,
    default_target: AnimationTarget,
) -> AnimationLibrary {
    let clip_id = clip.id.clone();
    AnimationLibrary::new("test_library")
        .with_default_target(default_target)
        .add_clip(clip)
        .add_state(AnimationState::new(state_id, clip_id))
}

#[test]
fn looping_playback_updates_sprite_frame_and_emits_loop() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 3);
    let library = add_library(
        &mut app,
        basic_state_library(
            AnimationClip::from_indices("idle_clip", [0, 1, 2])
                .with_timing(FrameTiming::SecondsPerFrame(0.1)),
            "idle",
            AnimationTarget::state("idle"),
        ),
    );
    let entity = spawn_sprite_player(&mut app, library, layout, AnimationTarget::state("idle"));

    initialize_player(&mut app);
    let mut looped_cursor = MessageCursor::<AnimationLooped>::default();
    let _ = read_messages(&app, &mut looped_cursor);

    let initial = animator(&app, entity);
    assert_eq!(initial.current_frame, 0);
    assert_eq!(initial.atlas_index, 0);
    assert_eq!(sprite_atlas_index(&app, entity), 0);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 1);
    assert_eq!(sprite_atlas_index(&app, entity), 1);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 2);
    assert_eq!(sprite_atlas_index(&app, entity), 2);

    update_with_delta(&mut app, Duration::from_millis(100));
    let looped = animator(&app, entity);
    assert_eq!(looped.current_frame, 0);
    assert_eq!(looped.completed_loops, 1);
    assert_eq!(sprite_atlas_index(&app, entity), 0);

    let messages = read_messages(&app, &mut looped_cursor);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].completed_loops, 1);
}

#[test]
fn reverse_once_finishes_on_the_first_logical_frame() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 3);
    let library = add_library(
        &mut app,
        basic_state_library(
            AnimationClip::from_indices("reverse_clip", [0, 1, 2])
                .with_timing(FrameTiming::SecondsPerFrame(0.1))
                .with_direction(PlaybackDirection::Reverse)
                .with_repeat(RepeatMode::Once),
            "reverse",
            AnimationTarget::state("reverse"),
        ),
    );
    let entity = spawn_sprite_player(&mut app, library, layout, AnimationTarget::state("reverse"));

    initialize_player(&mut app);
    let mut finished_cursor = MessageCursor::<AnimationFinished>::default();
    let _ = read_messages(&app, &mut finished_cursor);

    assert_eq!(animator(&app, entity).current_frame, 2);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 1);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 0);

    update_with_delta(&mut app, Duration::from_millis(100));
    let finished = animator(&app, entity);
    assert_eq!(finished.current_frame, 0);
    assert_eq!(finished.playback_state, PlaybackState::Finished);
    assert_eq!(finished.atlas_index, 0);

    let messages = read_messages(&app, &mut finished_cursor);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].clip.as_str(), "reverse_clip");
}

#[test]
fn ping_pong_once_returns_to_start_before_finishing() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 3);
    let library = add_library(
        &mut app,
        basic_state_library(
            AnimationClip::from_indices("ping_pong_clip", [0, 1, 2])
                .with_timing(FrameTiming::SecondsPerFrame(0.1))
                .with_direction(PlaybackDirection::PingPong)
                .with_repeat(RepeatMode::Once),
            "ping_pong",
            AnimationTarget::state("ping_pong"),
        ),
    );
    let entity = spawn_sprite_player(
        &mut app,
        library,
        layout,
        AnimationTarget::state("ping_pong"),
    );

    initialize_player(&mut app);
    assert_eq!(animator(&app, entity).current_frame, 0);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 1);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 2);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 1);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 0);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(
        animator(&app, entity).playback_state,
        PlaybackState::Finished
    );
    assert_eq!(animator(&app, entity).current_frame, 0);
}

#[test]
fn per_frame_durations_and_seek_commands_are_applied() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 3);
    let clip = AnimationClip::from_frames(
        "timed_clip",
        [
            ClipFrame::new(0).with_duration_seconds(0.2),
            ClipFrame::new(1).with_duration_seconds(0.05),
            ClipFrame::new(2).with_duration_seconds(0.3),
        ],
    )
    .with_timing(FrameTiming::SecondsPerFrame(0.1));
    let library = add_library(
        &mut app,
        basic_state_library(clip, "timed", AnimationTarget::state("timed")),
    );
    let entity = spawn_sprite_player(&mut app, library, layout, AnimationTarget::state("timed"));

    initialize_player(&mut app);
    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 0);

    update_with_delta(&mut app, Duration::from_millis(100));
    assert_eq!(animator(&app, entity).current_frame, 1);

    set_command(&mut app, entity, AnimationControlCommand::SeekFrame(2));
    update_with_delta(&mut app, Duration::ZERO);
    assert_eq!(animator(&app, entity).current_frame, 2);
    assert_eq!(sprite_atlas_index(&app, entity), 2);

    set_command(
        &mut app,
        entity,
        AnimationControlCommand::SeekNormalized(0.05),
    );
    update_with_delta(&mut app, Duration::ZERO);
    assert_eq!(animator(&app, entity).current_frame, 0);

    set_command(
        &mut app,
        entity,
        AnimationControlCommand::SeekNormalized(0.75),
    );
    update_with_delta(&mut app, Duration::ZERO);
    assert_eq!(animator(&app, entity).current_frame, 2);
}

#[test]
fn locked_one_shot_queues_requested_state_until_finish() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 4);
    let library = add_library(
        &mut app,
        AnimationLibrary::new("state_machine")
            .with_default_target(AnimationTarget::state("idle"))
            .add_clip(
                AnimationClip::from_indices("idle_clip", [0, 1])
                    .with_timing(FrameTiming::SecondsPerFrame(0.1)),
            )
            .add_clip(
                AnimationClip::from_indices("action_clip", [2, 3])
                    .with_timing(FrameTiming::SecondsPerFrame(0.1))
                    .with_repeat(RepeatMode::Once)
                    .with_interrupt_policy(InterruptPolicy::LockUntilFinished),
            )
            .add_state(AnimationState::new("idle", "idle_clip"))
            .add_state(AnimationState::new("action", "action_clip")),
    );
    let entity = spawn_sprite_player(&mut app, library, layout, AnimationTarget::state("idle"));

    initialize_player(&mut app);
    set_requested_target(&mut app, entity, AnimationTarget::state("action"));
    update_with_delta(&mut app, Duration::ZERO);
    assert_eq!(
        animator(&app, entity).current_state.unwrap().as_str(),
        "action"
    );

    set_requested_target(&mut app, entity, AnimationTarget::state("idle"));
    update_with_delta(&mut app, Duration::from_millis(50));

    let mid_action = animator(&app, entity);
    let mid_controller = controller(&app, entity);
    assert_eq!(mid_action.current_state.unwrap().as_str(), "action");
    assert_eq!(
        mid_controller.pending_target,
        Some(AnimationTarget::state("idle"))
    );

    update_with_delta(&mut app, Duration::from_millis(150));

    let after_finish = animator(&app, entity);
    let after_controller = controller(&app, entity);
    assert_eq!(after_finish.current_state.unwrap().as_str(), "idle");
    assert_eq!(after_finish.playback_state, PlaybackState::Playing);
    assert_eq!(after_controller.pending_target, None);
}

#[test]
fn discard_pending_policy_preserves_existing_pending_target() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 4);
    let library = add_library(
        &mut app,
        AnimationLibrary::new("pending_policy")
            .with_default_target(AnimationTarget::state("idle"))
            .add_clip(
                AnimationClip::from_indices("idle_clip", [0, 1])
                    .with_timing(FrameTiming::SecondsPerFrame(0.1)),
            )
            .add_clip(
                AnimationClip::from_indices("action_clip", [2, 3])
                    .with_timing(FrameTiming::SecondsPerFrame(0.1))
                    .with_repeat(RepeatMode::Once)
                    .with_interrupt_policy(InterruptPolicy::LockUntilFinished),
            )
            .add_state(AnimationState::new("idle", "idle_clip"))
            .add_state(AnimationState::new("action", "action_clip"))
            .add_state(AnimationState::new("other", "idle_clip")),
    );
    let entity = spawn_sprite_player(&mut app, library, layout, AnimationTarget::state("idle"));

    initialize_player(&mut app);
    {
        let mut entity_mut = app.world_mut().entity_mut(entity);
        let mut controller = entity_mut
            .get_mut::<AnimationController>()
            .expect("entity should have an AnimationController");
        controller.pending_request_policy = PendingRequestPolicy::Discard;
        controller.set_target(AnimationTarget::state("action"));
    }
    update_with_delta(&mut app, Duration::ZERO);

    {
        let mut entity_mut = app.world_mut().entity_mut(entity);
        let mut controller = entity_mut
            .get_mut::<AnimationController>()
            .expect("entity should have an AnimationController");
        controller.pending_target = Some(AnimationTarget::state("idle"));
        controller.set_target(AnimationTarget::state("other"));
    }
    update_with_delta(&mut app, Duration::from_millis(50));

    let controller = controller(&app, entity);
    assert_eq!(
        controller.pending_target,
        Some(AnimationTarget::state("idle"))
    );
}

#[test]
fn target_change_respects_controller_start_offset() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 4);
    let library = add_library(
        &mut app,
        AnimationLibrary::new("offset_switch")
            .with_default_target(AnimationTarget::state("idle"))
            .add_clip(
                AnimationClip::from_indices("idle_clip", [0, 1])
                    .with_timing(FrameTiming::SecondsPerFrame(0.2)),
            )
            .add_clip(
                AnimationClip::from_indices("walk_clip", [2, 3])
                    .with_timing(FrameTiming::SecondsPerFrame(0.2)),
            )
            .add_state(AnimationState::new("idle", "idle_clip"))
            .add_state(AnimationState::new("walk", "walk_clip")),
    );
    let entity = spawn_sprite_player(&mut app, library, layout, AnimationTarget::state("idle"));

    initialize_player(&mut app);
    {
        let mut entity_mut = app.world_mut().entity_mut(entity);
        let mut controller = entity_mut
            .get_mut::<AnimationController>()
            .expect("entity should have an AnimationController");
        controller.start_offset = StartOffset::Normalized(0.75);
    }

    set_requested_target(&mut app, entity, AnimationTarget::state("walk"));
    update_with_delta(&mut app, Duration::ZERO);

    let animator = animator(&app, entity);
    assert_eq!(
        animator.current_clip.as_ref().map(|clip| clip.as_str()),
        Some("walk_clip")
    );
    assert_eq!(animator.current_frame, 1);
    assert_eq!(animator.atlas_index, 3);
}

#[test]
fn frame_events_fire_once_and_do_not_repeat_while_paused() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 2);
    let clip = AnimationClip::from_frames(
        "event_clip",
        [
            ClipFrame::new(0),
            ClipFrame::new(1).with_event(AnimationEventMarker::named("impact")),
        ],
    )
    .with_timing(FrameTiming::SecondsPerFrame(0.1));
    let library = add_library(
        &mut app,
        basic_state_library(clip, "eventful", AnimationTarget::state("eventful")),
    );
    let entity = spawn_sprite_player(
        &mut app,
        library,
        layout,
        AnimationTarget::state("eventful"),
    );

    initialize_player(&mut app);
    let mut event_cursor = MessageCursor::<AnimationEventFired>::default();
    let _ = read_messages(&app, &mut event_cursor);

    update_with_delta(&mut app, Duration::from_millis(100));
    let first_events = read_messages(&app, &mut event_cursor);
    assert_eq!(first_events.len(), 1);
    assert_eq!(first_events[0].marker.name, "impact");

    set_command(&mut app, entity, AnimationControlCommand::Pause);
    update_with_delta(&mut app, Duration::ZERO);
    assert_eq!(animator(&app, entity).playback_state, PlaybackState::Paused);

    update_with_delta(&mut app, Duration::from_millis(500));
    assert!(read_messages(&app, &mut event_cursor).is_empty());
    assert_eq!(animator(&app, entity).current_frame, 1);
}

#[test]
fn large_delta_crosses_multiple_frames_and_preserves_event_order() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 4);
    let clip = AnimationClip::from_frames(
        "burst_clip",
        [
            ClipFrame::new(0),
            ClipFrame::new(1).with_event(AnimationEventMarker::named("one")),
            ClipFrame::new(2).with_event(AnimationEventMarker::named("two")),
            ClipFrame::new(3),
        ],
    )
    .with_timing(FrameTiming::SecondsPerFrame(0.1));
    let library = add_library(
        &mut app,
        basic_state_library(clip, "burst", AnimationTarget::state("burst")),
    );
    let entity = spawn_sprite_player(&mut app, library, layout, AnimationTarget::state("burst"));

    initialize_player(&mut app);
    let mut event_cursor = MessageCursor::<AnimationEventFired>::default();
    let _ = read_messages(&app, &mut event_cursor);

    update_with_delta(&mut app, Duration::from_millis(250));

    let animator = animator(&app, entity);
    assert_eq!(animator.current_frame, 2);
    assert!((animator.frame_elapsed_seconds - 0.05).abs() < 0.0001);

    let events = read_messages(&app, &mut event_cursor);
    assert_eq!(
        events
            .iter()
            .map(|message| message.marker.name.as_str())
            .collect::<Vec<_>>(),
        vec!["one", "two"]
    );
}

#[test]
fn missing_sprite_is_reported_gracefully() {
    let mut app = test_app();
    let library = add_library(
        &mut app,
        basic_state_library(
            AnimationClip::from_indices("idle_clip", [0])
                .with_timing(FrameTiming::SecondsPerFrame(0.1)),
            "idle",
            AnimationTarget::state("idle"),
        ),
    );
    let entity = spawn_headless_player(&mut app, library, AnimationTarget::state("idle"));

    initialize_player(&mut app);

    assert_eq!(
        animator(&app, entity).last_issue,
        Some(crate::AnimationIssue::MissingSprite)
    );
}

#[test]
fn asset_reload_invalidates_cached_playback_data() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 8);
    let library = add_library(
        &mut app,
        basic_state_library(
            AnimationClip::from_indices("idle_clip", [0])
                .with_timing(FrameTiming::SecondsPerFrame(0.1)),
            "idle",
            AnimationTarget::state("idle"),
        ),
    );
    let entity = spawn_sprite_player(
        &mut app,
        library.clone(),
        layout,
        AnimationTarget::state("idle"),
    );

    initialize_player(&mut app);
    assert_eq!(animator(&app, entity).atlas_index, 0);

    {
        let mut libraries = app.world_mut().resource_mut::<Assets<AnimationLibrary>>();
        let library_asset = libraries
            .get_mut(&library)
            .expect("library asset should exist");
        library_asset.clips[0].frames[0].atlas_index = 7;
    }

    app.world_mut()
        .write_message(AssetEvent::Modified { id: library.id() });
    set_command(&mut app, entity, AnimationControlCommand::Restart);
    update_with_delta(&mut app, Duration::ZERO);

    assert_eq!(animator(&app, entity).atlas_index, 7);
    assert_eq!(sprite_atlas_index(&app, entity), 7);
}

#[test]
fn clip_changes_emit_changed_messages() {
    let mut app = test_app();
    let layout = add_layout(&mut app, 4);
    let library = add_library(
        &mut app,
        AnimationLibrary::new("changed_messages")
            .with_default_target(AnimationTarget::state("idle"))
            .add_clip(AnimationClip::from_indices("idle_clip", [0, 1]))
            .add_clip(AnimationClip::from_indices("work_clip", [2, 3]))
            .add_state(AnimationState::new("idle", "idle_clip"))
            .add_state(AnimationState::new("work", "work_clip")),
    );
    let entity = spawn_sprite_player(&mut app, library, layout, AnimationTarget::state("idle"));

    let mut changed_cursor = MessageCursor::<AnimationChanged>::default();
    initialize_player(&mut app);
    let initial = read_messages(&app, &mut changed_cursor);
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].clip.as_str(), "idle_clip");

    set_requested_target(&mut app, entity, AnimationTarget::state("work"));
    update_with_delta(&mut app, Duration::ZERO);

    let changed = read_messages(&app, &mut changed_cursor);
    assert_eq!(changed.len(), 1);
    assert_eq!(
        changed[0].previous_clip.as_ref().map(|clip| clip.as_str()),
        Some("idle_clip")
    );
    assert_eq!(changed[0].clip.as_str(), "work_clip");
}
