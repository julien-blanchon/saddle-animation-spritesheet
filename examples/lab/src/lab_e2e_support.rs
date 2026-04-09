use bevy::prelude::*;
use saddle_bevy_e2e::action::Action;

use crate::{HeroMode, LabDiagnostics, LabControl};

pub fn request_mode(mode: HeroMode) -> Action {
    Action::Custom(Box::new(move |world: &mut World| {
        let mut control = world.resource_mut::<LabControl>();
        control.auto = false;
        control.request(mode);
    }))
}

pub fn wait_for_mode(mode: HeroMode, max_frames: u32) -> Action {
    let mode_name = mode.as_str();
    let clip_name = format!("{mode_name}_clip");
    Action::WaitUntil {
        label: format!("hero reached {mode_name} mode").into(),
        condition: Box::new(move |world: &World| {
            world
                .get_resource::<LabDiagnostics>()
                .is_some_and(|diagnostics| {
                    diagnostics.hero_state == mode_name || diagnostics.hero_clip == clip_name
                })
        }),
        max_frames,
    }
}

pub fn wait_for_impact_marker(max_frames: u32) -> Action {
    Action::WaitUntil {
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
        max_frames,
    }
}

pub fn wait_for_idle_after_one_shot(max_frames: u32) -> Action {
    Action::WaitUntil {
        label: "hero returned to idle".into(),
        condition: Box::new(|world: &World| {
            world
                .get_resource::<LabDiagnostics>()
                .is_some_and(|diagnostics| {
                    diagnostics.impact_events >= 1 && diagnostics.hero_clip == "idle_clip"
                })
        }),
        max_frames,
    }
}
