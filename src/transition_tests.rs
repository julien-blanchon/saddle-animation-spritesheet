use crate::{
    config::{
        AnimationTarget, NormalizedTimeWindow, TransitionDefinition, TransitionSource,
        TransitionTrigger,
    },
    transition::{
        RequestedTransitionResult, select_finished_transition, select_requested_transition,
    },
};

#[test]
fn requested_transition_uses_highest_priority_match() {
    let idle = AnimationTarget::state("idle");
    let walk = AnimationTarget::state("walk");
    let run = AnimationTarget::state("run");

    let transitions = vec![
        TransitionDefinition::requested(TransitionSource::State("idle".into()), walk.clone())
            .with_priority(1),
        TransitionDefinition::requested(TransitionSource::State("idle".into()), walk.clone())
            .with_priority(5),
        TransitionDefinition::requested(TransitionSource::State("idle".into()), run)
            .with_priority(10),
    ];

    let selected = select_requested_transition(&transitions, &idle, &walk, 0.0, 0.0);

    assert!(matches!(
        selected,
        RequestedTransitionResult::Applicable(TransitionDefinition { priority: 5, .. })
    ));
}

#[test]
fn requested_transition_reports_blocked_when_guards_do_not_match() {
    let idle = AnimationTarget::state("idle");
    let walk = AnimationTarget::state("walk");
    let transitions = vec![
        TransitionDefinition::requested(TransitionSource::State("idle".into()), walk.clone())
            .with_minimum_elapsed_seconds(0.5)
            .with_exit_window(NormalizedTimeWindow::new(0.75, 0.9)),
    ];

    let selected = select_requested_transition(&transitions, &idle, &walk, 0.1, 0.25);
    assert_eq!(selected, RequestedTransitionResult::Blocked);
}

#[test]
fn requested_transition_returns_no_rule_when_target_is_not_configured() {
    let idle = AnimationTarget::state("idle");
    let walk = AnimationTarget::state("walk");
    let run = AnimationTarget::state("run");
    let transitions = vec![
        TransitionDefinition::requested(TransitionSource::State("idle".into()), walk)
            .with_priority(5),
    ];

    let selected = select_requested_transition(&transitions, &idle, &run, 0.0, 0.0);
    assert_eq!(selected, RequestedTransitionResult::NoRule);
}

#[test]
fn finished_transition_prefers_highest_priority_rule_that_matches_guards() {
    let attack = AnimationTarget::state("attack");
    let idle = AnimationTarget::state("idle");
    let recoil = AnimationTarget::state("recoil");

    let transitions = vec![
        TransitionDefinition {
            source: TransitionSource::State("attack".into()),
            target: idle,
            trigger: TransitionTrigger::Finished,
            priority: 1,
            minimum_elapsed_seconds: 0.0,
            exit_window: None,
        },
        TransitionDefinition {
            source: TransitionSource::State("attack".into()),
            target: recoil.clone(),
            trigger: TransitionTrigger::Finished,
            priority: 5,
            minimum_elapsed_seconds: 0.0,
            exit_window: Some(NormalizedTimeWindow::new(0.95, 1.0)),
        },
    ];

    let selected = select_finished_transition(&transitions, &attack, 1.0, 1.0)
        .expect("a finished transition should be selected");

    assert_eq!(selected.target, recoil);
}

#[test]
fn finished_transition_ignores_rules_with_non_matching_source() {
    let attack = AnimationTarget::state("attack");
    let transitions = vec![TransitionDefinition {
        source: TransitionSource::State("idle".into()),
        target: AnimationTarget::state("walk"),
        trigger: TransitionTrigger::Finished,
        priority: 3,
        minimum_elapsed_seconds: 0.0,
        exit_window: None,
    }];

    assert!(select_finished_transition(&transitions, &attack, 1.0, 1.0).is_none());
}
