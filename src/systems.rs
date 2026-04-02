use std::collections::{HashMap, hash_map::Entry};

use bevy::{
    asset::{AssetEvent, AssetId},
    camera::visibility::ViewVisibility,
    prelude::*,
};

use crate::{
    components::{
        AnimationControlCommand, AnimationController, AnimationIssue, PlaybackState,
        SpritesheetAnimationSource, SpritesheetAnimator, StartOffset,
    },
    config::{
        AnimationClip, AnimationClipId, AnimationEventMarker, AnimationLibrary,
        AnimationLibraryValidationError, AnimationState, AnimationStateId, AnimationTarget,
        InterruptPolicy, PlaybackDirection, RepeatMode, TransitionDefinition,
    },
    events::{AnimationChanged, AnimationEventFired, AnimationFinished, AnimationLooped},
    transition::{
        RequestedTransitionResult, select_finished_transition, select_requested_transition,
    },
    util::{clamp01, entity_seeded_normalized},
};

#[derive(Resource, Default)]
pub(crate) struct RuntimeActive;

#[derive(Resource, Default)]
pub(crate) struct AnimationLibraryCaches {
    entries: HashMap<AssetId<AnimationLibrary>, CachedLibraryEntry>,
}

impl AnimationLibraryCaches {
    fn get_or_build<'a>(
        &'a mut self,
        handle: &Handle<AnimationLibrary>,
        libraries: &Assets<AnimationLibrary>,
    ) -> Option<&'a CachedLibraryEntry> {
        let id = handle.id();

        if let Entry::Vacant(slot) = self.entries.entry(id) {
            let entry = libraries
                .get(handle)
                .map(CachedLibraryEntry::from_library)?;
            slot.insert(entry);
        }

        self.entries.get(&id)
    }

    fn invalidate(&mut self, id: AssetId<AnimationLibrary>) {
        self.entries.remove(&id);
    }
}

#[derive(Component, Default)]
pub(crate) struct AnimatorRuntime {
    sequence_cursor: usize,
    accumulator_seconds: f32,
    finished_this_tick: bool,
    buffered_messages: Vec<BufferedMessage>,
}

#[derive(Clone)]
struct CachedStep {
    logical_frame: usize,
    atlas_index: usize,
    duration_seconds: f32,
    events: Vec<AnimationEventMarker>,
}

#[derive(Clone)]
struct CachedPlayback {
    target: AnimationTarget,
    state: Option<AnimationStateId>,
    clip: AnimationClipId,
    repeat: RepeatMode,
    interrupt_policy: InterruptPolicy,
    sequence: Vec<CachedStep>,
    prefix_seconds: Vec<f32>,
    cycle_duration_seconds: f32,
    logical_frame_count: usize,
}

impl CachedPlayback {
    fn current_step(&self, cursor: usize) -> &CachedStep {
        &self.sequence[cursor]
    }

    fn normalized_time(&self, cursor: usize, accumulator_seconds: f32) -> f32 {
        if self.cycle_duration_seconds <= f32::EPSILON {
            return 0.0;
        }

        clamp01((self.prefix_seconds[cursor] + accumulator_seconds) / self.cycle_duration_seconds)
    }

    fn first_sequence_index_for_frame(&self, logical_frame: usize) -> usize {
        self.sequence
            .iter()
            .position(|step| step.logical_frame == logical_frame)
            .unwrap_or_default()
    }
}

struct CachedLibrary {
    default_target: Option<AnimationTarget>,
    clip_playbacks: HashMap<AnimationClipId, CachedPlayback>,
    state_playbacks: HashMap<AnimationStateId, CachedPlayback>,
    transitions: Vec<TransitionDefinition>,
}

impl CachedLibrary {
    fn from_library(library: &AnimationLibrary) -> Result<Self, AnimationLibraryValidationError> {
        library.validate()?;

        let clip_lookup = library
            .clips
            .iter()
            .map(|clip| (clip.id.clone(), clip))
            .collect::<HashMap<_, _>>();

        let mut clip_playbacks = HashMap::new();
        for clip in &library.clips {
            let target = AnimationTarget::Clip(clip.id.clone());
            clip_playbacks.insert(clip.id.clone(), build_playback(target, None, clip, None));
        }

        let mut state_playbacks = HashMap::new();
        for state in &library.states {
            let clip = clip_lookup
                .get(&state.clip)
                .expect("validated state clip should exist");
            let target = AnimationTarget::State(state.id.clone());
            state_playbacks.insert(
                state.id.clone(),
                build_playback(target, Some(state), clip, Some(&state.playback)),
            );
        }

        Ok(Self {
            default_target: library.default_target.clone(),
            clip_playbacks,
            state_playbacks,
            transitions: library.transitions.clone(),
        })
    }

    fn playback_for_target(&self, target: &AnimationTarget) -> Option<&CachedPlayback> {
        match target {
            AnimationTarget::Clip(id) => self.clip_playbacks.get(id),
            AnimationTarget::State(id) => self.state_playbacks.get(id),
        }
    }
}

enum CachedLibraryEntry {
    Ready(CachedLibrary),
    Invalid,
}

impl CachedLibraryEntry {
    fn from_library(library: &AnimationLibrary) -> Self {
        match CachedLibrary::from_library(library) {
            Ok(cache) => Self::Ready(cache),
            Err(error) => {
                warn_once!("spritesheet library '{}' is invalid: {error}", library.name);
                Self::Invalid
            }
        }
    }
}

#[derive(Clone)]
enum BufferedMessage {
    Changed(AnimationChanged),
    Event(AnimationEventFired),
    Looped(AnimationLooped),
    Finished(AnimationFinished),
}

pub(crate) fn activate_players(mut commands: Commands) {
    commands.insert_resource(RuntimeActive);
}

pub(crate) fn deactivate_players(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SpritesheetAnimator), With<AnimatorRuntime>>,
) {
    commands.remove_resource::<RuntimeActive>();

    for (entity, mut animator) in &mut query {
        animator.playback_state = PlaybackState::Stopped;
        animator.current_target = None;
        animator.current_state = None;
        animator.current_clip = None;
        animator.current_frame = 0;
        animator.atlas_index = 0;
        animator.normalized_time = 0.0;
        animator.elapsed_seconds = 0.0;
        animator.frame_elapsed_seconds = 0.0;
        animator.completed_loops = 0;
        animator.last_issue = None;
        commands.entity(entity).remove::<AnimatorRuntime>();
    }
}

pub(crate) fn sync_library_events(
    mut events: MessageReader<AssetEvent<AnimationLibrary>>,
    mut caches: ResMut<AnimationLibraryCaches>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::Removed { id }
            | AssetEvent::Unused { id }
            | AssetEvent::LoadedWithDependencies { id } => *id,
        };
        caches.invalidate(id);
    }
}

pub(crate) fn initialize_new_players(
    active: Option<Res<RuntimeActive>>,
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<SpritesheetAnimationSource>,
            With<AnimationController>,
            With<SpritesheetAnimator>,
            Without<AnimatorRuntime>,
        ),
    >,
) {
    if active.is_none() {
        return;
    }

    for entity in &query {
        commands.entity(entity).insert(AnimatorRuntime::default());
    }
}

pub(crate) fn resolve_requests(
    active: Option<Res<RuntimeActive>>,
    libraries: Res<Assets<AnimationLibrary>>,
    mut caches: ResMut<AnimationLibraryCaches>,
    mut query: Query<(
        Entity,
        &SpritesheetAnimationSource,
        &mut AnimationController,
        &mut SpritesheetAnimator,
        &mut AnimatorRuntime,
    )>,
) {
    if active.is_none() {
        return;
    }

    for (entity, source, mut controller, mut animator, mut runtime) in &mut query {
        runtime.finished_this_tick = false;

        let Some(entry) = caches.get_or_build(&source.library, &libraries) else {
            animator.last_issue = Some(AnimationIssue::MissingLibrary);
            continue;
        };

        let cache = match entry {
            CachedLibraryEntry::Ready(cache) => cache,
            CachedLibraryEntry::Invalid => {
                animator.last_issue = Some(AnimationIssue::InvalidLibrary);
                continue;
            }
        };

        let command = std::mem::take(&mut controller.command);

        match command {
            AnimationControlCommand::None => {}
            AnimationControlCommand::Pause => {
                if animator.playback_state == PlaybackState::Playing {
                    animator.playback_state = PlaybackState::Paused;
                }
            }
            AnimationControlCommand::Resume => {
                if matches!(
                    animator.playback_state,
                    PlaybackState::Paused | PlaybackState::Stopped
                ) {
                    animator.playback_state = PlaybackState::Playing;
                }
            }
            AnimationControlCommand::Stop => {
                stop_current(cache, &mut animator, &mut runtime);
            }
            AnimationControlCommand::Restart => {
                restart_current(entity, cache, &mut animator, &mut runtime);
            }
            AnimationControlCommand::SeekFrame(frame) => {
                seek_current_frame(cache, frame, &mut animator, &mut runtime);
            }
            AnimationControlCommand::SeekNormalized(normalized_time) => {
                seek_current_normalized(cache, normalized_time, &mut animator, &mut runtime);
            }
            AnimationControlCommand::Play(target) => {
                let _ = process_request(
                    entity,
                    cache,
                    target,
                    Some(controller.start_offset.clone()),
                    true,
                    &mut controller,
                    &mut animator,
                    &mut runtime,
                );
            }
        }

        if animator.current_target.is_none() {
            let target = controller
                .requested_target
                .clone()
                .or_else(|| controller.default_target.clone())
                .or_else(|| cache.default_target.clone());

            if let Some(target) = target {
                let _ = switch_to_target(
                    entity,
                    cache,
                    target,
                    Some(controller.start_offset.clone()),
                    false,
                    &mut animator,
                    &mut runtime,
                );
            } else {
                animator.playback_state = PlaybackState::Stopped;
                animator.last_issue = Some(AnimationIssue::MissingTarget);
            }
            continue;
        }

        if let Some(current_target) = animator.current_target.clone()
            && cache.playback_for_target(&current_target).is_none()
        {
            animator.last_issue = Some(AnimationIssue::MissingTarget);

            if let Some(fallback_target) = controller
                .pending_target
                .clone()
                .or_else(|| controller.requested_target.clone())
                .or_else(|| controller.default_target.clone())
                .or_else(|| cache.default_target.clone())
            {
                let _ = switch_to_target(
                    entity,
                    cache,
                    fallback_target,
                    None,
                    false,
                    &mut animator,
                    &mut runtime,
                );
            }
            continue;
        }

        if let Some(target) = controller.pending_target.clone() {
            let _ = process_request(
                entity,
                cache,
                target,
                Some(controller.start_offset.clone()),
                false,
                &mut controller,
                &mut animator,
                &mut runtime,
            );
            continue;
        }

        if controller.is_changed() {
            let desired_target = controller
                .requested_target
                .clone()
                .or_else(|| controller.default_target.clone())
                .or_else(|| cache.default_target.clone());

            match desired_target {
                Some(target) => {
                    if animator.current_target.as_ref() != Some(&target) {
                        let _ = process_request(
                            entity,
                            cache,
                            target,
                            Some(controller.start_offset.clone()),
                            true,
                            &mut controller,
                            &mut animator,
                            &mut runtime,
                        );
                    }
                }
                None => controller.pending_target = None,
            }
        }
    }
}

pub(crate) fn advance_time(
    active: Option<Res<RuntimeActive>>,
    time: Res<Time>,
    libraries: Res<Assets<AnimationLibrary>>,
    mut caches: ResMut<AnimationLibraryCaches>,
    mut query: Query<(
        Entity,
        &SpritesheetAnimationSource,
        &mut SpritesheetAnimator,
        &mut AnimatorRuntime,
        Option<&Visibility>,
        Option<&ViewVisibility>,
    )>,
) {
    if active.is_none() {
        return;
    }

    let delta_seconds = time.delta_secs();

    for (entity, source, mut animator, mut runtime, visibility, view_visibility) in &mut query {
        if animator.playback_state != PlaybackState::Playing {
            continue;
        }

        if !should_tick(&animator, visibility, view_visibility) {
            continue;
        }

        let Some(entry) = caches.get_or_build(&source.library, &libraries) else {
            animator.last_issue = Some(AnimationIssue::MissingLibrary);
            continue;
        };

        let cache = match entry {
            CachedLibraryEntry::Ready(cache) => cache,
            CachedLibraryEntry::Invalid => {
                animator.last_issue = Some(AnimationIssue::InvalidLibrary);
                continue;
            }
        };

        let Some(playback) = current_playback(cache, &animator) else {
            animator.last_issue = Some(AnimationIssue::MissingTarget);
            continue;
        };

        sanitize_runtime(playback, &mut animator, &mut runtime);

        let scaled_delta = (delta_seconds * animator.speed_multiplier.max(0.0)).max(0.0);
        if !scaled_delta.is_finite() || scaled_delta <= f32::EPSILON {
            continue;
        }

        let mut remaining_seconds = scaled_delta;

        while remaining_seconds > 0.0 {
            let step = playback.current_step(runtime.sequence_cursor);
            let step_remaining = (step.duration_seconds - runtime.accumulator_seconds).max(0.0);

            if remaining_seconds + f32::EPSILON < step_remaining {
                runtime.accumulator_seconds += remaining_seconds;
                animator.elapsed_seconds += remaining_seconds;
                animator.frame_elapsed_seconds = runtime.accumulator_seconds;
                animator.normalized_time =
                    playback.normalized_time(runtime.sequence_cursor, runtime.accumulator_seconds);
                break;
            }

            animator.elapsed_seconds += step_remaining;
            remaining_seconds -= step_remaining;
            runtime.accumulator_seconds = 0.0;

            let is_last_step = runtime.sequence_cursor + 1 >= playback.sequence.len();
            if is_last_step {
                match playback.repeat {
                    RepeatMode::Loop => {
                        animator.completed_loops += 1;
                        runtime
                            .buffered_messages
                            .push(BufferedMessage::Looped(AnimationLooped {
                                entity,
                                target: playback.target.clone(),
                                state: playback.state.clone(),
                                clip: playback.clip.clone(),
                                completed_loops: animator.completed_loops,
                            }));

                        runtime.sequence_cursor = 0;
                        sync_animator_from_runtime(playback, &mut animator, &runtime);
                        buffer_current_frame_events(playback, &mut runtime, entity);
                    }
                    RepeatMode::Once => {
                        animator.playback_state = PlaybackState::Finished;
                        animator.frame_elapsed_seconds = step.duration_seconds;
                        animator.normalized_time = 1.0;
                        runtime.finished_this_tick = true;
                        runtime.buffered_messages.push(BufferedMessage::Finished(
                            AnimationFinished {
                                entity,
                                target: playback.target.clone(),
                                state: playback.state.clone(),
                                clip: playback.clip.clone(),
                            },
                        ));
                        break;
                    }
                }
            } else {
                runtime.sequence_cursor += 1;
                sync_animator_from_runtime(playback, &mut animator, &runtime);
                buffer_current_frame_events(playback, &mut runtime, entity);
            }
        }
    }
}

pub(crate) fn apply_finished_transitions(
    active: Option<Res<RuntimeActive>>,
    libraries: Res<Assets<AnimationLibrary>>,
    mut caches: ResMut<AnimationLibraryCaches>,
    mut query: Query<(
        Entity,
        &SpritesheetAnimationSource,
        &mut AnimationController,
        &mut SpritesheetAnimator,
        &mut AnimatorRuntime,
    )>,
) {
    if active.is_none() {
        return;
    }

    for (entity, source, mut controller, mut animator, mut runtime) in &mut query {
        if !runtime.finished_this_tick {
            continue;
        }
        runtime.finished_this_tick = false;

        let Some(entry) = caches.get_or_build(&source.library, &libraries) else {
            animator.last_issue = Some(AnimationIssue::MissingLibrary);
            continue;
        };

        let cache = match entry {
            CachedLibraryEntry::Ready(cache) => cache,
            CachedLibraryEntry::Invalid => {
                animator.last_issue = Some(AnimationIssue::InvalidLibrary);
                continue;
            }
        };

        let Some(current_target) = animator.current_target.clone() else {
            continue;
        };

        if let Some(transition) = select_finished_transition(
            &cache.transitions,
            &current_target,
            animator.elapsed_seconds,
            animator.normalized_time,
        ) {
            let _ = switch_to_target(
                entity,
                cache,
                transition.target.clone(),
                None,
                false,
                &mut animator,
                &mut runtime,
            );
            continue;
        }

        if let Some(target) = controller.pending_target.clone() {
            let _ = process_request(
                entity,
                cache,
                target,
                Some(controller.start_offset.clone()),
                false,
                &mut controller,
                &mut animator,
                &mut runtime,
            );
            continue;
        }

        if let Some(target) = controller
            .requested_target
            .clone()
            .or_else(|| controller.default_target.clone())
            .or_else(|| cache.default_target.clone())
            && animator.current_target.as_ref() != Some(&target)
        {
            let _ = process_request(
                entity,
                cache,
                target,
                Some(controller.start_offset.clone()),
                false,
                &mut controller,
                &mut animator,
                &mut runtime,
            );
        }
    }
}

pub(crate) fn emit_messages(
    active: Option<Res<RuntimeActive>>,
    mut event_messages: MessageWriter<AnimationEventFired>,
    mut changed_messages: MessageWriter<AnimationChanged>,
    mut finished_messages: MessageWriter<AnimationFinished>,
    mut looped_messages: MessageWriter<AnimationLooped>,
    mut query: Query<&mut AnimatorRuntime>,
) {
    if active.is_none() {
        return;
    }

    for mut runtime in &mut query {
        for message in runtime.buffered_messages.drain(..) {
            match message {
                BufferedMessage::Changed(message) => {
                    changed_messages.write(message);
                }
                BufferedMessage::Event(message) => {
                    event_messages.write(message);
                }
                BufferedMessage::Finished(message) => {
                    finished_messages.write(message);
                }
                BufferedMessage::Looped(message) => {
                    looped_messages.write(message);
                }
            }
        }
    }
}

pub(crate) fn write_sprite_frames(
    active: Option<Res<RuntimeActive>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    mut query: Query<(Option<&mut Sprite>, &mut SpritesheetAnimator), With<AnimatorRuntime>>,
) {
    if active.is_none() {
        return;
    }

    for (sprite, mut animator) in &mut query {
        let Some(mut sprite) = sprite else {
            animator.last_issue = Some(AnimationIssue::MissingSprite);
            continue;
        };

        let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
            animator.last_issue = Some(AnimationIssue::MissingSpriteAtlas);
            continue;
        };

        let Some(layout) = atlases.get(&texture_atlas.layout) else {
            animator.last_issue = Some(AnimationIssue::MissingAtlasLayout);
            continue;
        };

        if animator.atlas_index >= layout.len() {
            animator.last_issue = Some(AnimationIssue::AtlasIndexOutOfRange);
            continue;
        }

        if texture_atlas.index != animator.atlas_index {
            texture_atlas.index = animator.atlas_index;
        }

        if matches!(
            animator.last_issue,
            Some(
                AnimationIssue::MissingSprite
                    | AnimationIssue::MissingSpriteAtlas
                    | AnimationIssue::MissingAtlasLayout
                    | AnimationIssue::AtlasIndexOutOfRange
            )
        ) {
            animator.last_issue = None;
        }
    }
}

fn build_playback(
    target: AnimationTarget,
    state: Option<&AnimationState>,
    clip: &AnimationClip,
    playback_override: Option<&crate::config::PlaybackOverride>,
) -> CachedPlayback {
    let timing = playback_override
        .and_then(|override_| override_.timing)
        .unwrap_or(clip.timing);
    let repeat = playback_override
        .and_then(|override_| override_.repeat)
        .unwrap_or(clip.repeat);
    let direction = playback_override
        .and_then(|override_| override_.direction)
        .unwrap_or(clip.direction);
    let interrupt_policy = playback_override
        .and_then(|override_| override_.interrupt_policy)
        .unwrap_or(clip.interrupt_policy);

    let step_duration = timing.seconds_per_frame();
    let base_steps = clip
        .frames
        .iter()
        .enumerate()
        .map(|(logical_frame, frame)| CachedStep {
            logical_frame,
            atlas_index: frame.atlas_index,
            duration_seconds: frame.duration_seconds.unwrap_or(step_duration),
            events: frame.events.clone(),
        })
        .collect::<Vec<_>>();

    let sequence = build_sequence(&base_steps, direction, repeat);
    let mut prefix_seconds = Vec::with_capacity(sequence.len());
    let mut elapsed_seconds = 0.0;

    for step in &sequence {
        prefix_seconds.push(elapsed_seconds);
        elapsed_seconds += step.duration_seconds;
    }

    CachedPlayback {
        target,
        state: state.map(|state| state.id.clone()),
        clip: clip.id.clone(),
        repeat,
        interrupt_policy,
        sequence,
        prefix_seconds,
        cycle_duration_seconds: elapsed_seconds,
        logical_frame_count: clip.frames.len(),
    }
}

fn build_sequence(
    base_steps: &[CachedStep],
    direction: PlaybackDirection,
    repeat: RepeatMode,
) -> Vec<CachedStep> {
    if base_steps.len() <= 1 {
        return base_steps.to_vec();
    }

    match direction {
        PlaybackDirection::Forward => base_steps.to_vec(),
        PlaybackDirection::Reverse => base_steps.iter().rev().cloned().collect(),
        PlaybackDirection::PingPong => {
            let mut sequence = base_steps.to_vec();
            let tail_start = if repeat == RepeatMode::Once { 0 } else { 1 };
            for index in (tail_start..base_steps.len() - 1).rev() {
                sequence.push(base_steps[index].clone());
            }
            sequence
        }
    }
}

fn current_playback<'a>(
    cache: &'a CachedLibrary,
    animator: &SpritesheetAnimator,
) -> Option<&'a CachedPlayback> {
    animator
        .current_target
        .as_ref()
        .and_then(|target| cache.playback_for_target(target))
}

fn should_tick(
    animator: &SpritesheetAnimator,
    visibility: Option<&Visibility>,
    view_visibility: Option<&ViewVisibility>,
) -> bool {
    match animator.visibility_policy {
        crate::components::AnimationTickPolicy::Always => true,
        crate::components::AnimationTickPolicy::WhenVisible => {
            visibility.copied().unwrap_or(Visibility::Visible) != Visibility::Hidden
                && view_visibility
                    .copied()
                    .map(ViewVisibility::get)
                    .unwrap_or(true)
        }
    }
}

fn process_request(
    entity: Entity,
    cache: &CachedLibrary,
    target: AnimationTarget,
    start_offset: Option<StartOffset>,
    explicit_request: bool,
    controller: &mut AnimationController,
    animator: &mut SpritesheetAnimator,
    runtime: &mut AnimatorRuntime,
) -> bool {
    let Some(current_target) = animator.current_target.clone() else {
        return switch_to_target(
            entity,
            cache,
            target,
            start_offset,
            false,
            animator,
            runtime,
        );
    };

    if animator.current_target.as_ref() == Some(&target) {
        if explicit_request
            && controller.same_target_policy == crate::components::SameTargetPolicy::Restart
        {
            controller.pending_target = None;
            return switch_to_target(entity, cache, target, None, true, animator, runtime);
        }

        controller.pending_target = None;
        return false;
    }

    let Some(current_playback) = current_playback(cache, animator) else {
        animator.last_issue = Some(AnimationIssue::MissingTarget);
        return false;
    };

    let is_locked = current_playback.interrupt_policy == InterruptPolicy::LockUntilFinished
        && animator.playback_state != PlaybackState::Finished;

    let transition_result = select_requested_transition(
        &cache.transitions,
        &current_target,
        &target,
        animator.elapsed_seconds,
        animator.normalized_time,
    );

    if is_locked || matches!(transition_result, RequestedTransitionResult::Blocked) {
        queue_request(controller, target);
        return false;
    }

    controller.pending_target = None;

    let resolved_target = match transition_result {
        RequestedTransitionResult::Applicable(transition) => transition.target.clone(),
        RequestedTransitionResult::Blocked | RequestedTransitionResult::NoRule => target,
    };

    switch_to_target(
        entity,
        cache,
        resolved_target,
        start_offset,
        false,
        animator,
        runtime,
    )
}

fn queue_request(controller: &mut AnimationController, target: AnimationTarget) {
    controller.pending_target = match controller.pending_request_policy {
        crate::components::PendingRequestPolicy::Replace => Some(target),
        crate::components::PendingRequestPolicy::KeepFirst => {
            controller.pending_target.take().or(Some(target))
        }
        crate::components::PendingRequestPolicy::Discard => controller.pending_target.clone(),
    };
}

fn switch_to_target(
    entity: Entity,
    cache: &CachedLibrary,
    target: AnimationTarget,
    start_offset: Option<StartOffset>,
    restarted: bool,
    animator: &mut SpritesheetAnimator,
    runtime: &mut AnimatorRuntime,
) -> bool {
    let Some(playback) = cache.playback_for_target(&target) else {
        animator.last_issue = Some(AnimationIssue::MissingTarget);
        return false;
    };

    let previous_target = animator.current_target.clone();
    let previous_clip = animator.current_clip.clone();

    animator.playback_state = PlaybackState::Playing;
    animator.current_target = Some(target);
    animator.current_state = playback.state.clone();
    animator.current_clip = Some(playback.clip.clone());
    animator.completed_loops = 0;
    animator.elapsed_seconds = 0.0;

    runtime.sequence_cursor = 0;
    runtime.accumulator_seconds = 0.0;
    runtime.finished_this_tick = false;

    if let Some(offset) = start_offset {
        match offset {
            StartOffset::None => {}
            StartOffset::Normalized(normalized_time) => {
                seek_to_normalized_internal(playback, normalized_time, animator, runtime);
            }
            StartOffset::EntitySeeded => {
                seek_to_normalized_internal(
                    playback,
                    entity_seeded_normalized(entity),
                    animator,
                    runtime,
                );
            }
        }
    } else {
        sync_animator_from_runtime(playback, animator, runtime);
    }

    runtime
        .buffered_messages
        .push(BufferedMessage::Changed(AnimationChanged {
            entity,
            previous_target,
            target: playback.target.clone(),
            previous_clip,
            clip: playback.clip.clone(),
            restarted,
        }));
    buffer_current_frame_events(playback, runtime, entity);
    animator.last_issue = None;
    true
}

fn restart_current(
    entity: Entity,
    cache: &CachedLibrary,
    animator: &mut SpritesheetAnimator,
    runtime: &mut AnimatorRuntime,
) {
    let Some(target) = animator.current_target.clone() else {
        return;
    };

    let _ = switch_to_target(entity, cache, target, None, true, animator, runtime);
}

fn seek_current_frame(
    cache: &CachedLibrary,
    logical_frame: usize,
    animator: &mut SpritesheetAnimator,
    runtime: &mut AnimatorRuntime,
) {
    let Some(playback) = current_playback(cache, animator) else {
        animator.last_issue = Some(AnimationIssue::MissingTarget);
        return;
    };

    let clamped_frame = logical_frame.min(playback.logical_frame_count.saturating_sub(1));
    runtime.sequence_cursor = playback.first_sequence_index_for_frame(clamped_frame);
    runtime.accumulator_seconds = 0.0;
    animator.frame_elapsed_seconds = 0.0;
    sync_animator_from_runtime(playback, animator, runtime);
}

fn seek_current_normalized(
    cache: &CachedLibrary,
    normalized_time: f32,
    animator: &mut SpritesheetAnimator,
    runtime: &mut AnimatorRuntime,
) {
    let Some(playback) = current_playback(cache, animator) else {
        animator.last_issue = Some(AnimationIssue::MissingTarget);
        return;
    };

    seek_to_normalized_internal(playback, normalized_time, animator, runtime);
}

fn seek_to_normalized_internal(
    playback: &CachedPlayback,
    normalized_time: f32,
    animator: &mut SpritesheetAnimator,
    runtime: &mut AnimatorRuntime,
) {
    let normalized_time = clamp01(normalized_time);
    if playback.sequence.is_empty() {
        return;
    }

    let target_seconds = if normalized_time >= 1.0 {
        (playback.cycle_duration_seconds - f32::EPSILON).max(0.0)
    } else {
        playback.cycle_duration_seconds * normalized_time
    };

    let mut cursor = playback.sequence.len() - 1;
    for index in 0..playback.sequence.len() {
        let step = &playback.sequence[index];
        let start_seconds = playback.prefix_seconds[index];
        let end_seconds = start_seconds + step.duration_seconds;
        if target_seconds < end_seconds || index + 1 == playback.sequence.len() {
            cursor = index;
            runtime.sequence_cursor = index;
            runtime.accumulator_seconds = (target_seconds - start_seconds).max(0.0);
            break;
        }
    }

    if runtime.sequence_cursor >= playback.sequence.len() {
        runtime.sequence_cursor = cursor;
        runtime.accumulator_seconds = 0.0;
    }

    animator.frame_elapsed_seconds = runtime.accumulator_seconds;
    animator.normalized_time = normalized_time;
    sync_animator_from_runtime(playback, animator, runtime);
}

fn stop_current(
    cache: &CachedLibrary,
    animator: &mut SpritesheetAnimator,
    runtime: &mut AnimatorRuntime,
) {
    animator.playback_state = PlaybackState::Stopped;
    animator.completed_loops = 0;
    animator.elapsed_seconds = 0.0;

    if let Some(playback) = current_playback(cache, animator) {
        runtime.sequence_cursor = 0;
        runtime.accumulator_seconds = 0.0;
        sync_animator_from_runtime(playback, animator, runtime);
    }
}

fn sync_animator_from_runtime(
    playback: &CachedPlayback,
    animator: &mut SpritesheetAnimator,
    runtime: &AnimatorRuntime,
) {
    let step = playback.current_step(runtime.sequence_cursor);
    animator.current_state = playback.state.clone();
    animator.current_clip = Some(playback.clip.clone());
    animator.current_frame = step.logical_frame;
    animator.atlas_index = step.atlas_index;
    animator.frame_elapsed_seconds = runtime.accumulator_seconds;
    animator.normalized_time = if animator.playback_state == PlaybackState::Finished {
        1.0
    } else {
        playback.normalized_time(runtime.sequence_cursor, runtime.accumulator_seconds)
    };
}

fn sanitize_runtime(
    playback: &CachedPlayback,
    animator: &mut SpritesheetAnimator,
    runtime: &mut AnimatorRuntime,
) {
    if runtime.sequence_cursor >= playback.sequence.len() {
        runtime.sequence_cursor = playback.sequence.len().saturating_sub(1);
        runtime.accumulator_seconds = 0.0;
    }

    let step = playback.current_step(runtime.sequence_cursor);
    if runtime.accumulator_seconds >= step.duration_seconds {
        runtime.accumulator_seconds = 0.0;
    }

    sync_animator_from_runtime(playback, animator, runtime);
}

fn buffer_current_frame_events(
    playback: &CachedPlayback,
    runtime: &mut AnimatorRuntime,
    entity: Entity,
) {
    let step = playback.current_step(runtime.sequence_cursor);
    for marker in &step.events {
        runtime
            .buffered_messages
            .push(BufferedMessage::Event(AnimationEventFired {
                entity,
                target: playback.target.clone(),
                state: playback.state.clone(),
                clip: playback.clip.clone(),
                frame: step.logical_frame,
                atlas_index: step.atlas_index,
                marker: marker.clone(),
            }));
    }
}
