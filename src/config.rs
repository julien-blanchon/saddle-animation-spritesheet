use std::{collections::HashSet, fmt, ops::RangeInclusive};

use bevy::{asset::Asset, prelude::*};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub struct AnimationClipId(pub String);

impl AnimationClipId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AnimationClipId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for AnimationClipId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AnimationClipId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub struct AnimationStateId(pub String);

impl AnimationStateId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AnimationStateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for AnimationStateId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AnimationStateId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum AnimationTarget {
    State(AnimationStateId),
    Clip(AnimationClipId),
}

impl AnimationTarget {
    pub fn state(id: impl Into<AnimationStateId>) -> Self {
        Self::State(id.into())
    }

    pub fn clip(id: impl Into<AnimationClipId>) -> Self {
        Self::Clip(id.into())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum RepeatMode {
    #[default]
    Loop,
    Once,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum PlaybackDirection {
    #[default]
    Forward,
    Reverse,
    PingPong,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum FrameTiming {
    FramesPerSecond(f32),
    SecondsPerFrame(f32),
}

impl Default for FrameTiming {
    fn default() -> Self {
        Self::FramesPerSecond(12.0)
    }
}

impl FrameTiming {
    pub fn seconds_per_frame(self) -> f32 {
        match self {
            Self::FramesPerSecond(fps) => 1.0 / fps,
            Self::SecondsPerFrame(seconds) => seconds,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub enum AnimationEventPayload {
    #[default]
    None,
    Bool(bool),
    Int(i64),
    Float(f32),
    Text(String),
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct AnimationEventMarker {
    pub name: String,
    pub payload: AnimationEventPayload,
}

impl AnimationEventMarker {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            payload: AnimationEventPayload::None,
        }
    }

    pub fn with_payload(mut self, payload: AnimationEventPayload) -> Self {
        self.payload = payload;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
pub struct ClipFrame {
    pub atlas_index: usize,
    pub duration_seconds: Option<f32>,
    pub events: Vec<AnimationEventMarker>,
}

impl ClipFrame {
    pub fn new(atlas_index: usize) -> Self {
        Self {
            atlas_index,
            duration_seconds: None,
            events: Vec::new(),
        }
    }

    pub fn with_duration_seconds(mut self, duration_seconds: f32) -> Self {
        self.duration_seconds = Some(duration_seconds);
        self
    }

    pub fn with_event(mut self, marker: AnimationEventMarker) -> Self {
        self.events.push(marker);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum InterruptPolicy {
    #[default]
    Interruptible,
    LockUntilFinished,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
pub struct AnimationClip {
    pub id: AnimationClipId,
    pub frames: Vec<ClipFrame>,
    pub timing: FrameTiming,
    pub repeat: RepeatMode,
    pub direction: PlaybackDirection,
    pub interrupt_policy: InterruptPolicy,
}

impl AnimationClip {
    pub fn from_frames(
        id: impl Into<AnimationClipId>,
        frames: impl IntoIterator<Item = ClipFrame>,
    ) -> Self {
        Self {
            id: id.into(),
            frames: frames.into_iter().collect(),
            timing: FrameTiming::default(),
            repeat: RepeatMode::default(),
            direction: PlaybackDirection::default(),
            interrupt_policy: InterruptPolicy::default(),
        }
    }

    pub fn from_indices(
        id: impl Into<AnimationClipId>,
        atlas_indices: impl IntoIterator<Item = usize>,
    ) -> Self {
        Self::from_frames(id, atlas_indices.into_iter().map(ClipFrame::new))
    }

    pub fn from_range(id: impl Into<AnimationClipId>, range: RangeInclusive<usize>) -> Self {
        Self::from_indices(id, range)
    }

    pub fn with_timing(mut self, timing: FrameTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn with_frames_per_second(mut self, frames_per_second: f32) -> Self {
        self.timing = FrameTiming::FramesPerSecond(frames_per_second);
        self
    }

    pub fn with_seconds_per_frame(mut self, seconds_per_frame: f32) -> Self {
        self.timing = FrameTiming::SecondsPerFrame(seconds_per_frame);
        self
    }

    pub fn with_repeat(mut self, repeat: RepeatMode) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn with_direction(mut self, direction: PlaybackDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_interrupt_policy(mut self, interrupt_policy: InterruptPolicy) -> Self {
        self.interrupt_policy = interrupt_policy;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct PlaybackOverride {
    pub timing: Option<FrameTiming>,
    pub repeat: Option<RepeatMode>,
    pub direction: Option<PlaybackDirection>,
    pub interrupt_policy: Option<InterruptPolicy>,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
pub struct AnimationState {
    pub id: AnimationStateId,
    pub clip: AnimationClipId,
    pub playback: PlaybackOverride,
}

impl AnimationState {
    pub fn new(id: impl Into<AnimationStateId>, clip: impl Into<AnimationClipId>) -> Self {
        Self {
            id: id.into(),
            clip: clip.into(),
            playback: PlaybackOverride::default(),
        }
    }

    pub fn with_timing(mut self, timing: FrameTiming) -> Self {
        self.playback.timing = Some(timing);
        self
    }

    pub fn with_repeat(mut self, repeat: RepeatMode) -> Self {
        self.playback.repeat = Some(repeat);
        self
    }

    pub fn with_direction(mut self, direction: PlaybackDirection) -> Self {
        self.playback.direction = Some(direction);
        self
    }

    pub fn with_interrupt_policy(mut self, interrupt_policy: InterruptPolicy) -> Self {
        self.playback.interrupt_policy = Some(interrupt_policy);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
pub struct NormalizedTimeWindow {
    pub start: f32,
    pub end: f32,
}

impl NormalizedTimeWindow {
    pub fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }

    pub fn contains(self, normalized_time: f32) -> bool {
        normalized_time >= self.start && normalized_time <= self.end
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum TransitionSource {
    Any,
    State(AnimationStateId),
    Clip(AnimationClipId),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq, Hash)]
pub enum TransitionTrigger {
    #[default]
    Requested,
    Finished,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
pub struct TransitionDefinition {
    pub source: TransitionSource,
    pub target: AnimationTarget,
    pub trigger: TransitionTrigger,
    pub priority: i32,
    pub minimum_elapsed_seconds: f32,
    pub exit_window: Option<NormalizedTimeWindow>,
}

impl TransitionDefinition {
    pub fn requested(source: TransitionSource, target: AnimationTarget) -> Self {
        Self {
            source,
            target,
            trigger: TransitionTrigger::Requested,
            priority: 0,
            minimum_elapsed_seconds: 0.0,
            exit_window: None,
        }
    }

    pub fn finished(source: TransitionSource, target: AnimationTarget) -> Self {
        Self {
            source,
            target,
            trigger: TransitionTrigger::Finished,
            priority: 0,
            minimum_elapsed_seconds: 0.0,
            exit_window: None,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_minimum_elapsed_seconds(mut self, minimum_elapsed_seconds: f32) -> Self {
        self.minimum_elapsed_seconds = minimum_elapsed_seconds;
        self
    }

    pub fn with_exit_window(mut self, exit_window: NormalizedTimeWindow) -> Self {
        self.exit_window = Some(exit_window);
        self
    }
}

#[derive(Asset, Clone, Debug, Default, Reflect)]
#[reflect(Debug, Default)]
pub struct AnimationLibrary {
    pub name: String,
    pub default_target: Option<AnimationTarget>,
    pub clips: Vec<AnimationClip>,
    pub states: Vec<AnimationState>,
    pub transitions: Vec<TransitionDefinition>,
}

impl AnimationLibrary {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default_target: None,
            clips: Vec::new(),
            states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn with_default_target(mut self, target: AnimationTarget) -> Self {
        self.default_target = Some(target);
        self
    }

    pub fn add_clip(mut self, clip: AnimationClip) -> Self {
        self.clips.push(clip);
        self
    }

    pub fn add_state(mut self, state: AnimationState) -> Self {
        self.states.push(state);
        self
    }

    pub fn add_transition(mut self, transition: TransitionDefinition) -> Self {
        self.transitions.push(transition);
        self
    }

    pub fn validate(&self) -> Result<(), AnimationLibraryValidationError> {
        let mut clip_ids = HashSet::new();
        for clip in &self.clips {
            if !clip_ids.insert(clip.id.clone()) {
                return Err(AnimationLibraryValidationError::DuplicateClipId(
                    clip.id.0.clone(),
                ));
            }
            if clip.frames.is_empty() {
                return Err(AnimationLibraryValidationError::EmptyClip(
                    clip.id.0.clone(),
                ));
            }
            validate_timing(clip.timing, clip.id.as_str())?;
            for frame in &clip.frames {
                if let Some(duration_seconds) = frame.duration_seconds
                    && duration_seconds <= 0.0
                {
                    return Err(AnimationLibraryValidationError::NonPositiveFrameDuration {
                        clip: clip.id.0.clone(),
                        atlas_index: frame.atlas_index,
                        duration_seconds,
                    });
                }
            }
            if clip.repeat == RepeatMode::Loop
                && clip.interrupt_policy == InterruptPolicy::LockUntilFinished
            {
                return Err(AnimationLibraryValidationError::NeverFinishingLock {
                    target: clip.id.0.clone(),
                });
            }
        }

        let mut state_ids = HashSet::new();
        for state in &self.states {
            if !state_ids.insert(state.id.clone()) {
                return Err(AnimationLibraryValidationError::DuplicateStateId(
                    state.id.0.clone(),
                ));
            }
            if !self.clips.iter().any(|clip| clip.id == state.clip) {
                return Err(AnimationLibraryValidationError::MissingClip {
                    owner: state.id.0.clone(),
                    clip: state.clip.0.clone(),
                });
            }
            if let Some(timing) = state.playback.timing {
                validate_timing(timing, state.id.as_str())?;
            }
            let clip = self
                .clips
                .iter()
                .find(|clip| clip.id == state.clip)
                .expect("validated clip should exist");
            let repeat = state.playback.repeat.unwrap_or(clip.repeat);
            let interrupt_policy = state
                .playback
                .interrupt_policy
                .unwrap_or(clip.interrupt_policy);
            if repeat == RepeatMode::Loop && interrupt_policy == InterruptPolicy::LockUntilFinished
            {
                return Err(AnimationLibraryValidationError::NeverFinishingLock {
                    target: state.id.0.clone(),
                });
            }
        }

        if let Some(default_target) = &self.default_target
            && !self.target_exists(default_target)
        {
            return Err(AnimationLibraryValidationError::MissingTarget(
                format_target(default_target),
            ));
        }

        for transition in &self.transitions {
            match &transition.source {
                TransitionSource::Any => {}
                TransitionSource::State(state)
                    if !self.states.iter().any(|entry| entry.id == *state) =>
                {
                    return Err(AnimationLibraryValidationError::MissingTarget(
                        state.0.clone(),
                    ));
                }
                TransitionSource::Clip(clip)
                    if !self.clips.iter().any(|entry| entry.id == *clip) =>
                {
                    return Err(AnimationLibraryValidationError::MissingTarget(
                        clip.0.clone(),
                    ));
                }
                _ => {}
            }
            if !self.target_exists(&transition.target) {
                return Err(AnimationLibraryValidationError::MissingTarget(
                    format_target(&transition.target),
                ));
            }
            if transition.minimum_elapsed_seconds < 0.0 {
                return Err(
                    AnimationLibraryValidationError::NegativeMinimumElapsedSeconds {
                        target: format_target(&transition.target),
                        minimum_elapsed_seconds: transition.minimum_elapsed_seconds,
                    },
                );
            }
            if let Some(window) = transition.exit_window {
                if !(0.0..=1.0).contains(&window.start) || !(0.0..=1.0).contains(&window.end) {
                    return Err(AnimationLibraryValidationError::InvalidExitWindow {
                        start: window.start,
                        end: window.end,
                    });
                }
                if window.start > window.end {
                    return Err(AnimationLibraryValidationError::InvalidExitWindow {
                        start: window.start,
                        end: window.end,
                    });
                }
            }
        }

        if self.default_target.is_none() && self.clips.is_empty() && self.states.is_empty() {
            return Err(AnimationLibraryValidationError::EmptyLibrary(
                self.name.clone(),
            ));
        }

        Ok(())
    }

    fn target_exists(&self, target: &AnimationTarget) -> bool {
        match target {
            AnimationTarget::State(id) => self.states.iter().any(|state| state.id == *id),
            AnimationTarget::Clip(id) => self.clips.iter().any(|clip| clip.id == *id),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationLibraryValidationError {
    EmptyLibrary(String),
    DuplicateClipId(String),
    DuplicateStateId(String),
    EmptyClip(String),
    MissingClip {
        owner: String,
        clip: String,
    },
    MissingTarget(String),
    InvalidFrameTiming {
        owner: String,
        timing: FrameTiming,
    },
    NonPositiveFrameDuration {
        clip: String,
        atlas_index: usize,
        duration_seconds: f32,
    },
    InvalidExitWindow {
        start: f32,
        end: f32,
    },
    NegativeMinimumElapsedSeconds {
        target: String,
        minimum_elapsed_seconds: f32,
    },
    NeverFinishingLock {
        target: String,
    },
}

impl fmt::Display for AnimationLibraryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLibrary(name) => write!(
                f,
                "animation library '{name}' does not define any clips or states"
            ),
            Self::DuplicateClipId(id) => write!(f, "duplicate clip id '{id}'"),
            Self::DuplicateStateId(id) => write!(f, "duplicate state id '{id}'"),
            Self::EmptyClip(id) => write!(f, "clip '{id}' does not contain any frames"),
            Self::MissingClip { owner, clip } => {
                write!(f, "'{owner}' references missing clip '{clip}'")
            }
            Self::MissingTarget(target) => {
                write!(f, "missing transition or default target '{target}'")
            }
            Self::InvalidFrameTiming { owner, timing } => {
                write!(f, "'{owner}' uses invalid frame timing {timing:?}")
            }
            Self::NonPositiveFrameDuration {
                clip,
                atlas_index,
                duration_seconds,
            } => write!(
                f,
                "clip '{clip}' uses non-positive duration {duration_seconds} for atlas index {atlas_index}"
            ),
            Self::InvalidExitWindow { start, end } => {
                write!(f, "invalid normalized exit window [{start}, {end}]")
            }
            Self::NegativeMinimumElapsedSeconds {
                target,
                minimum_elapsed_seconds,
            } => write!(
                f,
                "transition to '{target}' uses negative minimum elapsed seconds {minimum_elapsed_seconds}"
            ),
            Self::NeverFinishingLock { target } => write!(
                f,
                "'{target}' uses LockUntilFinished with looping playback, which can never release"
            ),
        }
    }
}

impl std::error::Error for AnimationLibraryValidationError {}

fn validate_timing(
    timing: FrameTiming,
    owner: &str,
) -> Result<(), AnimationLibraryValidationError> {
    let valid = match timing {
        FrameTiming::FramesPerSecond(value) => value.is_finite() && value > 0.0,
        FrameTiming::SecondsPerFrame(value) => value.is_finite() && value > 0.0,
    };

    if valid {
        Ok(())
    } else {
        Err(AnimationLibraryValidationError::InvalidFrameTiming {
            owner: owner.to_string(),
            timing,
        })
    }
}

fn format_target(target: &AnimationTarget) -> String {
    match target {
        AnimationTarget::State(id) => format!("state:{}", id.0),
        AnimationTarget::Clip(id) => format!("clip:{}", id.0),
    }
}
