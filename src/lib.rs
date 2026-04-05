mod aseprite;
mod components;
mod config;
mod easing;
mod events;
mod systems;
mod transition;
mod util;

pub use aseprite::AsepriteImportError;
pub use components::{
    AnimationControlCommand, AnimationController, AnimationIssue, AnimationTickPolicy,
    PendingRequestPolicy, PlaybackState, SameTargetPolicy, SpritesheetAnimationBundle,
    SpritesheetAnimationSource, SpritesheetAnimator, StartOffset,
};
pub use config::{
    AnimationClip, AnimationClipId, AnimationEventMarker, AnimationEventPayload, AnimationLibrary,
    AnimationLibraryValidationError, AnimationState, AnimationStateId, AnimationTarget, ClipFrame,
    FrameTiming, InterruptPolicy, NormalizedTimeWindow, PlaybackDirection, PlaybackOverride,
    RepeatMode, TransitionDefinition, TransitionSource, TransitionTrigger,
};
pub use easing::{Easing, EasingVariety};
pub use events::{AnimationChanged, AnimationEventFired, AnimationFinished, AnimationLooped};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpritesheetSystems {
    ResolveRequests,
    AdvanceTime,
    ApplyTransitions,
    EmitEvents,
    WriteSpriteFrame,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct SpritesheetPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl SpritesheetPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for SpritesheetPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for SpritesheetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AnimationLibrary>()
            .init_resource::<systems::AnimationLibraryCaches>()
            .add_message::<AnimationEventFired>()
            .add_message::<AnimationLooped>()
            .add_message::<AnimationFinished>()
            .add_message::<AnimationChanged>()
            .register_type::<AnimationClip>()
            .register_type::<AnimationClipId>()
            .register_type::<AnimationControlCommand>()
            .register_type::<AnimationController>()
            .register_type::<AnimationEventMarker>()
            .register_type::<AnimationEventPayload>()
            .register_type::<AnimationIssue>()
            .register_type::<AnimationLibrary>()
            .register_type::<AnimationState>()
            .register_type::<AnimationStateId>()
            .register_type::<AnimationTarget>()
            .register_type::<AnimationTickPolicy>()
            .register_type::<ClipFrame>()
            .register_type::<FrameTiming>()
            .register_type::<InterruptPolicy>()
            .register_type::<NormalizedTimeWindow>()
            .register_type::<PendingRequestPolicy>()
            .register_type::<PlaybackDirection>()
            .register_type::<PlaybackOverride>()
            .register_type::<PlaybackState>()
            .register_type::<RepeatMode>()
            .register_type::<SameTargetPolicy>()
            .register_type::<SpritesheetAnimationSource>()
            .register_type::<SpritesheetAnimator>()
            .register_type::<StartOffset>()
            .register_type::<TransitionDefinition>()
            .register_type::<TransitionSource>()
            .register_type::<TransitionTrigger>()
            .register_type::<Easing>()
            .register_type::<EasingVariety>()
            .add_systems(self.activate_schedule, systems::activate_players)
            .add_systems(self.deactivate_schedule, systems::deactivate_players)
            .configure_sets(
                self.update_schedule,
                (
                    SpritesheetSystems::ResolveRequests,
                    SpritesheetSystems::AdvanceTime,
                    SpritesheetSystems::ApplyTransitions,
                    SpritesheetSystems::EmitEvents,
                    SpritesheetSystems::WriteSpriteFrame,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    (
                        systems::sync_library_events,
                        systems::initialize_new_players,
                        systems::resolve_requests,
                    )
                        .chain()
                        .in_set(SpritesheetSystems::ResolveRequests),
                    systems::advance_time.in_set(SpritesheetSystems::AdvanceTime),
                    systems::apply_finished_transitions
                        .in_set(SpritesheetSystems::ApplyTransitions),
                    systems::emit_messages.in_set(SpritesheetSystems::EmitEvents),
                    systems::write_sprite_frames.in_set(SpritesheetSystems::WriteSpriteFrame),
                ),
            );
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;

#[cfg(test)]
#[path = "transition_tests.rs"]
mod transition_tests;
