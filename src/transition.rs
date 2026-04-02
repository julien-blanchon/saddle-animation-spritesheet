use std::cmp::Reverse;

use crate::config::{
    AnimationTarget, NormalizedTimeWindow, TransitionDefinition, TransitionSource,
    TransitionTrigger,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RequestedTransitionResult<'a> {
    Applicable(&'a TransitionDefinition),
    Blocked,
    NoRule,
}

pub(crate) fn select_requested_transition<'a>(
    transitions: &'a [TransitionDefinition],
    current_target: &AnimationTarget,
    desired_target: &AnimationTarget,
    elapsed_seconds: f32,
    normalized_time: f32,
) -> RequestedTransitionResult<'a> {
    let mut candidates = transitions
        .iter()
        .filter(|transition| transition.trigger == TransitionTrigger::Requested)
        .filter(|transition| transition.target == *desired_target)
        .filter(|transition| source_matches(&transition.source, current_target))
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return RequestedTransitionResult::NoRule;
    }

    candidates.sort_by_key(|transition| Reverse(transition.priority));

    for transition in candidates {
        if guard_matches(
            transition.exit_window,
            transition.minimum_elapsed_seconds,
            elapsed_seconds,
            normalized_time,
        ) {
            return RequestedTransitionResult::Applicable(transition);
        }
    }

    RequestedTransitionResult::Blocked
}

pub(crate) fn select_finished_transition<'a>(
    transitions: &'a [TransitionDefinition],
    current_target: &AnimationTarget,
    elapsed_seconds: f32,
    normalized_time: f32,
) -> Option<&'a TransitionDefinition> {
    let mut candidates = transitions
        .iter()
        .filter(|transition| transition.trigger == TransitionTrigger::Finished)
        .filter(|transition| source_matches(&transition.source, current_target))
        .collect::<Vec<_>>();

    candidates.sort_by_key(|transition| Reverse(transition.priority));
    candidates.into_iter().find(|transition| {
        guard_matches(
            transition.exit_window,
            transition.minimum_elapsed_seconds,
            elapsed_seconds,
            normalized_time,
        )
    })
}

fn source_matches(source: &TransitionSource, current_target: &AnimationTarget) -> bool {
    match (source, current_target) {
        (TransitionSource::Any, _) => true,
        (TransitionSource::State(expected), AnimationTarget::State(current)) => expected == current,
        (TransitionSource::Clip(expected), AnimationTarget::Clip(current)) => expected == current,
        _ => false,
    }
}

fn guard_matches(
    exit_window: Option<NormalizedTimeWindow>,
    minimum_elapsed_seconds: f32,
    elapsed_seconds: f32,
    normalized_time: f32,
) -> bool {
    elapsed_seconds >= minimum_elapsed_seconds
        && exit_window
            .map(|window| window.contains(normalized_time))
            .unwrap_or(true)
}
