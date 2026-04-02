use bevy::prelude::*;
use saddle_saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};

use crate::{HeroMode, LabControl, LabDiagnostics, LabHero};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "spritesheet_smoke",
        "spritesheet_state_machine",
        "spritesheet_frame_events",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "spritesheet_smoke" => Some(build_smoke()),
        "spritesheet_state_machine" => Some(build_state_machine()),
        "spritesheet_frame_events" => Some(build_frame_events()),
        _ => None,
    }
}

fn set_mode(mode: HeroMode) -> Action {
    Action::Custom(Box::new(move |world: &mut World| {
        let mut control = world.resource_mut::<LabControl>();
        control.auto = false;
        control.request(mode);
    }))
}

fn build_smoke() -> Scenario {
    Scenario::builder("spritesheet_smoke")
        .description(
            "Boot the crate-local lab, wait for the runtime to settle, assert the hero and crowd diagnostics are live, then capture a baseline screenshot.",
        )
        .then(Action::WaitFrames(45))
        .then(assertions::entity_exists::<LabHero>("hero entity exists"))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "diagnostics reflect active playback",
            |diagnostics| {
                !diagnostics.hero_clip.is_empty()
                    && diagnostics.hero_playing
                    && diagnostics.crowd_phase_span > 0.15
            },
        ))
        .then(assertions::log_summary("spritesheet_smoke summary"))
        .then(Action::Screenshot("spritesheet_smoke".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_state_machine() -> Scenario {
    Scenario::builder("spritesheet_state_machine")
        .description(
            "Drive the hero into a locked one-shot, assert the clip changes to use_tool, then verify it falls back to idle and capture both checkpoints.",
        )
        .then(Action::WaitFrames(30))
        .then(set_mode(HeroMode::UseTool))
        .then(Action::WaitFrames(2))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "hero entered the one-shot clip",
            |diagnostics| diagnostics.hero_clip == "use_tool_clip",
        ))
        .then(Action::Screenshot("spritesheet_state_machine_enter".into()))
        .then(Action::WaitUntil {
            label: "hero returned to idle".into(),
            condition: Box::new(|world: &World| {
                world
                    .get_resource::<LabDiagnostics>()
                    .is_some_and(|diagnostics| {
                        diagnostics.impact_events >= 1 && diagnostics.hero_clip == "idle_clip"
                    })
            }),
            max_frames: 120,
        })
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "hero returned to idle after impact",
            |diagnostics| diagnostics.impact_events >= 1 && diagnostics.hero_clip == "idle_clip",
        ))
        .then(assertions::log_summary("spritesheet_state_machine summary"))
        .then(Action::Screenshot("spritesheet_state_machine_recover".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_frame_events() -> Scenario {
    Scenario::builder("spritesheet_frame_events")
        .description(
            "Trigger the one-shot, wait until the impact marker fires, assert the marker payload landed on frame 1, and capture the reactive frame.",
        )
        .then(Action::WaitFrames(30))
        .then(set_mode(HeroMode::UseTool))
        .then(Action::WaitUntil {
            label: "impact marker fired".into(),
            condition: Box::new(|world: &World| {
                world
                    .get_resource::<LabDiagnostics>()
                    .is_some_and(|diagnostics| {
                        diagnostics.impact_events >= 1
                            && diagnostics.last_event == "impact"
                            && diagnostics.last_event_frame == 1
                    })
            }),
            max_frames: 120,
        })
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "impact marker fired on frame 1",
            |diagnostics| {
                diagnostics.impact_events >= 1
                    && diagnostics.last_event == "impact"
                    && diagnostics.last_event_frame == 1
            },
        ))
        .then(Action::Screenshot("spritesheet_frame_events_impact".into()))
        .then(Action::WaitFrames(12))
        .then(Action::Screenshot("spritesheet_frame_events_after".into()))
        .then(assertions::log_summary("spritesheet_frame_events summary"))
        .then(Action::WaitFrames(1))
        .build()
}
