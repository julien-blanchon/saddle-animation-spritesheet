use bevy::prelude::*;

use crate::config::{AnimationClipId, AnimationEventMarker, AnimationStateId, AnimationTarget};

#[derive(Message, Clone, Debug, PartialEq)]
pub struct AnimationEventFired {
    pub entity: Entity,
    pub target: AnimationTarget,
    pub state: Option<AnimationStateId>,
    pub clip: AnimationClipId,
    pub frame: usize,
    pub atlas_index: usize,
    pub marker: AnimationEventMarker,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct AnimationLooped {
    pub entity: Entity,
    pub target: AnimationTarget,
    pub state: Option<AnimationStateId>,
    pub clip: AnimationClipId,
    pub completed_loops: u32,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct AnimationFinished {
    pub entity: Entity,
    pub target: AnimationTarget,
    pub state: Option<AnimationStateId>,
    pub clip: AnimationClipId,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct AnimationChanged {
    pub entity: Entity,
    pub previous_target: Option<AnimationTarget>,
    pub target: AnimationTarget,
    pub previous_clip: Option<AnimationClipId>,
    pub clip: AnimationClipId,
    pub restarted: bool,
}
