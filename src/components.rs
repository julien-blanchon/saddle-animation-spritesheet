use bevy::prelude::*;

use crate::config::{AnimationClipId, AnimationLibrary, AnimationStateId, AnimationTarget};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum AnimationTickPolicy {
    #[default]
    Always,
    WhenVisible,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum PendingRequestPolicy {
    #[default]
    Replace,
    KeepFirst,
    Discard,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum SameTargetPolicy {
    #[default]
    KeepProgress,
    Restart,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Finished,
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub enum StartOffset {
    #[default]
    None,
    Normalized(f32),
    EntitySeeded,
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub enum AnimationControlCommand {
    #[default]
    None,
    Play(AnimationTarget),
    Pause,
    Resume,
    Stop,
    Restart,
    SeekFrame(usize),
    SeekNormalized(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum AnimationIssue {
    MissingLibrary,
    InvalidLibrary,
    MissingTarget,
    MissingSprite,
    MissingSpriteAtlas,
    MissingAtlasLayout,
    AtlasIndexOutOfRange,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Debug)]
pub struct SpritesheetAnimationSource {
    pub library: Handle<AnimationLibrary>,
}

impl SpritesheetAnimationSource {
    pub fn new(library: Handle<AnimationLibrary>) -> Self {
        Self { library }
    }
}

#[derive(Component, Clone, Default, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct AnimationController {
    pub default_target: Option<AnimationTarget>,
    pub requested_target: Option<AnimationTarget>,
    pub pending_target: Option<AnimationTarget>,
    pub pending_request_policy: PendingRequestPolicy,
    pub same_target_policy: SameTargetPolicy,
    pub start_offset: StartOffset,
    pub command: AnimationControlCommand,
}

impl AnimationController {
    pub fn with_default_target(mut self, target: AnimationTarget) -> Self {
        self.default_target = Some(target);
        self
    }

    pub fn with_requested_target(mut self, target: AnimationTarget) -> Self {
        self.requested_target = Some(target);
        self
    }

    pub fn with_start_offset(mut self, start_offset: StartOffset) -> Self {
        self.start_offset = start_offset;
        self
    }

    pub fn set_target(&mut self, target: AnimationTarget) {
        self.requested_target = Some(target);
    }

    pub fn clear_target(&mut self) {
        self.requested_target = None;
    }

    pub fn clear_pending(&mut self) {
        self.pending_target = None;
    }

    pub fn play_state_once(&mut self, state: impl Into<AnimationStateId>) {
        self.command = AnimationControlCommand::Play(AnimationTarget::State(state.into()));
    }

    pub fn play_clip_once(&mut self, clip: impl Into<AnimationClipId>) {
        self.command = AnimationControlCommand::Play(AnimationTarget::Clip(clip.into()));
    }

    pub fn pause(&mut self) {
        self.command = AnimationControlCommand::Pause;
    }

    pub fn resume(&mut self) {
        self.command = AnimationControlCommand::Resume;
    }

    pub fn stop(&mut self) {
        self.command = AnimationControlCommand::Stop;
    }

    pub fn restart(&mut self) {
        self.command = AnimationControlCommand::Restart;
    }

    pub fn seek_to_frame(&mut self, frame: usize) {
        self.command = AnimationControlCommand::SeekFrame(frame);
    }

    pub fn seek_to_normalized_time(&mut self, normalized_time: f32) {
        self.command = AnimationControlCommand::SeekNormalized(normalized_time);
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct SpritesheetAnimator {
    pub playback_state: PlaybackState,
    pub speed_multiplier: f32,
    pub visibility_policy: AnimationTickPolicy,
    pub current_target: Option<AnimationTarget>,
    pub current_state: Option<AnimationStateId>,
    pub current_clip: Option<AnimationClipId>,
    pub current_frame: usize,
    pub atlas_index: usize,
    pub normalized_time: f32,
    pub elapsed_seconds: f32,
    pub frame_elapsed_seconds: f32,
    pub completed_loops: u32,
    pub last_issue: Option<AnimationIssue>,
}

impl Default for SpritesheetAnimator {
    fn default() -> Self {
        Self {
            playback_state: PlaybackState::Stopped,
            speed_multiplier: 1.0,
            visibility_policy: AnimationTickPolicy::Always,
            current_target: None,
            current_state: None,
            current_clip: None,
            current_frame: 0,
            atlas_index: 0,
            normalized_time: 0.0,
            elapsed_seconds: 0.0,
            frame_elapsed_seconds: 0.0,
            completed_loops: 0,
            last_issue: None,
        }
    }
}

impl SpritesheetAnimator {
    pub fn with_speed(mut self, speed_multiplier: f32) -> Self {
        self.speed_multiplier = speed_multiplier;
        self
    }

    pub fn with_visibility_policy(mut self, visibility_policy: AnimationTickPolicy) -> Self {
        self.visibility_policy = visibility_policy;
        self
    }
}

#[derive(Bundle, Default)]
pub struct SpritesheetAnimationBundle {
    pub source: SpritesheetAnimationSource,
    pub controller: AnimationController,
    pub animator: SpritesheetAnimator,
}

impl SpritesheetAnimationBundle {
    pub fn new(library: Handle<AnimationLibrary>, default_target: AnimationTarget) -> Self {
        Self {
            source: SpritesheetAnimationSource::new(library),
            controller: AnimationController::default().with_default_target(default_target),
            animator: SpritesheetAnimator::default(),
        }
    }
}
