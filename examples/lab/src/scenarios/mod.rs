use bevy::prelude::*;
use saddle_animation_spritesheet::{
    AnimationController, AnimationLibrary, AnimationTarget, SpritesheetAnimationBundle,
    SpritesheetAnimator,
};
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};

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
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "spritesheet_smoke" => Some(build_smoke()),
        "spritesheet_state_machine" => Some(build_state_machine()),
        "spritesheet_frame_events" => Some(build_frame_events()),
        "spritesheet_aseprite_import" => Some(build_aseprite_import()),
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
