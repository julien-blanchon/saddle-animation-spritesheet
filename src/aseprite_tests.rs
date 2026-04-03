use crate::{AnimationLibrary, PlaybackDirection};

#[test]
fn imports_tagged_aseprite_json_into_clips_and_states() {
    let json = r#"
    {
        "frames": {
            "hero 0.aseprite": { "duration": 80 },
            "hero 1.aseprite": { "duration": 90 },
            "hero 2.aseprite": { "duration": 100 },
            "hero 3.aseprite": { "duration": 110 }
        },
        "meta": {
            "frameTags": [
                { "name": "idle", "from": 0, "to": 1, "direction": "forward" },
                { "name": "turn", "from": 1, "to": 3, "direction": "pingpong" }
            ]
        }
    }
    "#;

    let library = AnimationLibrary::from_aseprite_json("hero", json).expect("import should work");

    assert_eq!(library.clips.len(), 2);
    assert_eq!(library.states.len(), 2);
    assert_eq!(
        library.default_target,
        Some(crate::AnimationTarget::state("idle"))
    );

    let idle = library
        .clips
        .iter()
        .find(|clip| clip.id.as_str() == "idle")
        .expect("idle clip should exist");
    assert_eq!(idle.frames[0].atlas_index, 0);
    assert_eq!(idle.frames[1].atlas_index, 1);

    let turn = library
        .clips
        .iter()
        .find(|clip| clip.id.as_str() == "turn")
        .expect("turn clip should exist");
    assert_eq!(turn.direction, PlaybackDirection::PingPong);
    assert_eq!(turn.frames[0].atlas_index, 1);
    assert_eq!(turn.frames[2].atlas_index, 3);
}

#[test]
fn imports_untagged_aseprite_json_as_single_default_clip() {
    let json = r#"
    {
        "frames": [
            { "duration": 50 },
            { "duration": 60 },
            { "duration": 70 }
        ],
        "meta": {}
    }
    "#;

    let library = AnimationLibrary::from_aseprite_json("full_sheet", json)
        .expect("import should succeed");

    assert_eq!(library.clips.len(), 1);
    assert!(library.states.is_empty());
    assert_eq!(
        library.default_target,
        Some(crate::AnimationTarget::clip("default"))
    );
    assert_eq!(library.clips[0].frames[2].atlas_index, 2);
}
