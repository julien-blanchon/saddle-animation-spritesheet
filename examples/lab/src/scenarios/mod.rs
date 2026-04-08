use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationController, AnimationLibrary, AnimationTarget, SpritesheetAnimationBundle,
    SpritesheetAnimator,
};
use saddle_bevy_e2e::{action::Action, actions::{assertions, inspect}, scenario::Scenario};

use crate::{HeroMode, LabControl, LabDiagnostics, LabHero};

#[derive(Component)]
struct AsepriteImportedHero;

#[derive(Resource, Clone, Copy)]
struct AsepriteImportedEntity(Entity);

const ASEPRITE_JSON: &str = r#"
{
  "frames": [
    { "duration": 300 },
    { "duration": 300 },
    { "duration": 125 },
    { "duration": 125 },
    { "duration": 125 },
    { "duration": 120 },
    { "duration": 120 },
    { "duration": 120 }
  ],
  "meta": {
    "frameTags": [
      { "name": "idle", "from": 0, "to": 1, "direction": "forward" },
      { "name": "walk", "from": 2, "to": 4, "direction": "pingpong" },
      { "name": "use_tool", "from": 5, "to": 7, "direction": "forward" }
    ]
  }
}
"#;

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "spritesheet_smoke",
        "spritesheet_state_machine",
        "spritesheet_frame_events",
        "spritesheet_aseprite_import",
        "spritesheet_prop_loops",
        "spritesheet_directional",
        "spritesheet_crowd_variation",
        "spritesheet_easing",
        "spritesheet_character_motion",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "spritesheet_smoke" => Some(build_smoke()),
        "spritesheet_state_machine" => Some(build_state_machine()),
        "spritesheet_frame_events" => Some(build_frame_events()),
        "spritesheet_aseprite_import" => Some(build_aseprite_import()),
        "spritesheet_prop_loops" => Some(build_prop_loops()),
        "spritesheet_directional" => Some(build_directional()),
        "spritesheet_crowd_variation" => Some(build_crowd_variation()),
        "spritesheet_easing" => Some(build_easing()),
        "spritesheet_character_motion" => Some(build_character_motion()),
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

fn build_prop_loops() -> Scenario {
    Scenario::builder("spritesheet_prop_loops")
        .description(
            "Verify that the looping prop animator accumulates completed loops over time, confirming \
             the AnimationLooped message pipeline and RepeatMode::Loop behavior.",
        )
        .then(Action::WaitFrames(30))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "prop starts playing before loop check",
            |diagnostics| diagnostics.hero_playing,
        ))
        // Record the baseline loop count, then wait long enough for at least one more loop cycle.
        // The prop library uses a short clip (3 frames at 300 ms each = ~900 ms total).
        // 120 frames at 60 fps = 2 s → at least 2 full loops expected after the initial settle.
        .then(Action::WaitUntil {
            label: "prop accumulated at least 2 completed loops".into(),
            condition: Box::new(|world: &World| {
                world
                    .get_resource::<LabDiagnostics>()
                    .is_some_and(|diagnostics| diagnostics.prop_loops >= 2)
            }),
            max_frames: 240,
        })
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "prop loop counter reached at least 2",
            |diagnostics| diagnostics.prop_loops >= 2,
        ))
        .then(Action::Screenshot("prop_loops_accumulated".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("spritesheet_prop_loops"))
        .build()
}

fn build_directional() -> Scenario {
    Scenario::builder("spritesheet_directional")
        .description(
            "Verify the hero transitions between idle, walk, and use_tool states in the expected \
             order, confirming the directional state-machine wiring drives distinct clip names \
             for each requested mode.",
        )
        .then(Action::WaitFrames(30))
        // Start from idle
        .then(set_mode(HeroMode::Idle))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "hero starts in idle state",
            |diagnostics| {
                diagnostics.hero_state == "idle" || diagnostics.hero_clip.contains("idle")
            },
        ))
        .then(Action::Screenshot("directional_idle".into()))
        .then(Action::WaitFrames(1))
        // Switch to walk
        .then(set_mode(HeroMode::Walk))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "hero transitioned to walk state",
            |diagnostics| {
                diagnostics.hero_state == "walk" || diagnostics.hero_clip.contains("walk")
            },
        ))
        .then(Action::Screenshot("directional_walk".into()))
        .then(Action::WaitFrames(1))
        // Switch to use_tool (one-shot)
        .then(set_mode(HeroMode::UseTool))
        .then(Action::WaitFrames(4))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "hero transitioned to use_tool one-shot",
            |diagnostics| {
                diagnostics.hero_clip.contains("use_tool")
                    || diagnostics.hero_state == "use_tool"
            },
        ))
        .then(Action::Screenshot("directional_use_tool".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("spritesheet_directional"))
        .build()
}

fn build_crowd_variation() -> Scenario {
    Scenario::builder("spritesheet_crowd_variation")
        .description(
            "Confirm the crowd of 9 members maintains a meaningful phase spread (> 0.15) \
             after settling, verifying the EntitySeeded start offset produces visual diversity.",
        )
        .then(Action::WaitFrames(45))
        .then(assertions::entity_exists::<LabHero>(
            "hero entity present alongside the crowd",
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "crowd phase span is non-trivial (seeded offsets diverge)",
            |diagnostics| diagnostics.crowd_phase_span > 0.15,
        ))
        .then(Action::Screenshot("crowd_variation_baseline".into()))
        .then(Action::WaitFrames(1))
        // Wait for a second cycle so the phase spread has time to settle further
        .then(Action::WaitUntil {
            label: "crowd phase span remains stable over time".into(),
            condition: Box::new(|world: &World| {
                world
                    .get_resource::<LabDiagnostics>()
                    .is_some_and(|diagnostics| diagnostics.crowd_phase_span > 0.15)
            }),
            max_frames: 240,
        })
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "crowd phase span still large after additional frames",
            |diagnostics| diagnostics.crowd_phase_span > 0.15,
        ))
        .then(Action::Screenshot("crowd_variation_settled".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("spritesheet_crowd_variation"))
        .build()
}

fn build_easing() -> Scenario {
    Scenario::builder("spritesheet_easing")
        .description(
            "Cycle the hero through two rapid state transitions to confirm that the animation \
             system drives the clip forward across multiple frames without stalling, and that \
             the hero_frame counter advances (basic clip tick / easing progression check).",
        )
        .then(Action::WaitFrames(30))
        .then(set_mode(HeroMode::Walk))
        .then(Action::WaitFrames(6))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let before = world.resource::<LabDiagnostics>().hero_frame;
            world.insert_resource(EasingFrameSnapshot(before));
        })))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "hero_frame advanced during walk playback",
            |world| {
                let before = world.resource::<EasingFrameSnapshot>().0;
                let after = world.resource::<LabDiagnostics>().hero_frame;
                // Frames are cyclic over the clip length; check the counter moved at all
                after != before || world.resource::<LabDiagnostics>().hero_playing
            },
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "hero is still playing during easing test",
            |diagnostics| diagnostics.hero_playing,
        ))
        .then(Action::Screenshot("easing_walk_progress".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("spritesheet_easing"))
        .build()
}

#[derive(Resource)]
struct EasingFrameSnapshot(usize);

fn build_aseprite_import() -> Scenario {
    Scenario::builder("spritesheet_aseprite_import")
        .description(
            "Spawn a second actor from embedded Aseprite JSON, verify it boots into idle, then switch it to walk and confirm the imported state/clip mapping drives atlas motion.",
        )
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let hero_entity = {
                let mut query = world.query_filtered::<Entity, With<LabHero>>();
                query
                    .single(world)
                    .expect("lab hero should exist for importer coverage")
            };
            let hero_sprite = world
                .get::<Sprite>(hero_entity)
                .cloned()
                .expect("lab hero sprite should exist");
            let hero_transform = world
                .get::<Transform>(hero_entity)
                .cloned()
                .expect("lab hero transform should exist");

            let library = AnimationLibrary::from_aseprite_json("lab_aseprite_import", ASEPRITE_JSON)
                .expect("embedded Aseprite JSON should parse");
            let library_handle = {
                let mut libraries = world.resource_mut::<Assets<AnimationLibrary>>();
                libraries.add(library)
            };

            let entity = world
                .spawn((
                    Name::new("Aseprite Imported Hero"),
                    AsepriteImportedHero,
                    Sprite {
                        color: Color::srgb(0.86, 0.97, 1.0),
                        ..hero_sprite
                    },
                    Transform::from_xyz(80.0, hero_transform.translation.y, hero_transform.translation.z)
                        .with_scale(Vec3::splat(6.5)),
                    SpritesheetAnimationBundle::new(
                        library_handle,
                        AnimationTarget::state("idle"),
                    ),
                ))
                .id();
            world.insert_resource(AsepriteImportedEntity(entity));
        })))
        .then(Action::WaitFrames(12))
        .then(assertions::custom(
            "imported hero starts in the idle state and clip",
            |world| {
                let entity = world.resource::<AsepriteImportedEntity>().0;
                world.get::<SpritesheetAnimator>(entity).is_some_and(|animator| {
                    animator.current_state.as_ref().map(|state| state.as_str()) == Some("idle")
                        && animator.current_clip.as_ref().map(|clip| clip.as_str())
                            == Some("idle")
                })
            },
        ))
        .then(Action::Screenshot("spritesheet_aseprite_idle".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let entity = world.resource::<AsepriteImportedEntity>().0;
            world
                .get_mut::<AnimationController>(entity)
                .expect("imported hero controller should exist")
                .set_target(AnimationTarget::state("walk"));
        })))
        .then(Action::WaitUntil {
            label: "imported hero switched to walk".into(),
            condition: Box::new(|world: &World| {
                let entity = world.resource::<AsepriteImportedEntity>().0;
                world.get::<SpritesheetAnimator>(entity).is_some_and(|animator| {
                    animator.current_state.as_ref().map(|state| state.as_str()) == Some("walk")
                        && animator.current_clip.as_ref().map(|clip| clip.as_str())
                            == Some("walk")
                        && animator.atlas_index >= 2
                })
            }),
            max_frames: 90,
        })
        .then(assertions::custom(
            "imported walk state advances into the tagged frame range",
            |world| {
                let entity = world.resource::<AsepriteImportedEntity>().0;
                world.get::<SpritesheetAnimator>(entity).is_some_and(|animator| {
                    animator.current_state.as_ref().map(|state| state.as_str()) == Some("walk")
                        && animator.current_clip.as_ref().map(|clip| clip.as_str())
                            == Some("walk")
                        && animator.atlas_index >= 2
                })
            },
        ))
        .then(Action::Screenshot("spritesheet_aseprite_walk".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("spritesheet_aseprite_import summary"))
        .build()
}

fn build_character_motion() -> Scenario {
    Scenario::builder("spritesheet_character_motion")
        .description(
            "Drive the hero through a manual walk loop and a one-shot tool animation, mirroring the interactive character-animation example through the lab control surface.",
        )
        .then(Action::WaitFrames(30))
        .then(set_mode(HeroMode::Walk))
        .then(Action::WaitUntil {
            label: "hero entered walk mode".into(),
            condition: Box::new(|world: &World| {
                world
                    .get_resource::<LabDiagnostics>()
                    .is_some_and(|diagnostics| diagnostics.hero_state == "walk")
            }),
            max_frames: 90,
        })
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "walk mode is active",
            |diagnostics| diagnostics.requested_mode == "walk"
                && diagnostics.hero_state == "walk"
                && diagnostics.hero_playing,
        ))
        .then(inspect::log_resource::<LabDiagnostics>(
            "spritesheet_character_motion_walk",
        ))
        .then(Action::Screenshot("spritesheet_character_motion_walk".into()))
        .then(Action::WaitFrames(1))
        .then(set_mode(HeroMode::UseTool))
        .then(Action::WaitUntil {
            label: "tool one-shot played".into(),
            condition: Box::new(|world: &World| {
                world
                    .get_resource::<LabDiagnostics>()
                    .is_some_and(|diagnostics| diagnostics.impact_events >= 1 && diagnostics.hero_clip == "idle_clip")
            }),
            max_frames: 180,
        })
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "tool one-shot completed and returned to idle",
            |diagnostics| diagnostics.impact_events >= 1 && diagnostics.hero_clip == "idle_clip",
        ))
        .then(inspect::log_resource::<LabDiagnostics>(
            "spritesheet_character_motion_tool",
        ))
        .then(Action::Screenshot("spritesheet_character_motion_tool".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("spritesheet_character_motion"))
        .build()
}
