# Saddle Animation Spritesheet

Reusable spritesheet animation runtime for Bevy 2D sprites. The crate owns clip definition, runtime playback state, request resolution, transition handling, and frame-event emission around `TextureAtlasLayout` / `TextureAtlas` without importing any project-specific game code.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
saddle-animation-spritesheet = { git = "https://github.com/julien-blanchon/saddle-animation-spritesheet" }
```

```rust,no_run
use bevy::prelude::*;
use saddle_animation_spritesheet::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpritesheetPlugin::always_on(Update))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut libraries: ResMut<Assets<AnimationLibrary>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(24),
        4,
        1,
        None,
        None,
    ));
    let library = libraries.add(
        AnimationLibrary::new("example")
            .with_default_target(AnimationTarget::state("idle"))
            .add_clip(AnimationClip::from_indices("idle_clip", [0, 1]))
            .add_clip(AnimationClip::from_indices("walk_clip", [2, 3]))
            .add_state(AnimationState::new("idle", "idle_clip"))
            .add_state(AnimationState::new("walk", "walk_clip")),
    );

    commands.spawn((
        Name::new("Animated Sprite"),
        Sprite::from_atlas_image(
            Handle::<Image>::default(),
            TextureAtlas { layout, index: 0 },
        ),
        SpritesheetAnimationBundle::new(library, AnimationTarget::state("idle")),
    ));
}
```

## Public API

| Type | Purpose |
| --- | --- |
| `SpritesheetPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `SpritesheetSystems` | Public ordering hooks: `ResolveRequests`, `AdvanceTime`, `ApplyTransitions`, `EmitEvents`, `WriteSpriteFrame` |
| `AnimationLibrary` | Authoring asset: named clips, states, transitions, and optional default target |
| `AnimationClip` / `ClipFrame` | Clip frame list, timing, repeat mode, direction, interrupt policy, and optional per-frame events |
| `AnimationState` / `PlaybackOverride` | Named logical state that points at a clip and can override clip playback defaults |
| `AnimationController` | External control surface for requested targets, commands, queue policy, start offsets, and same-target behavior |
| `SpritesheetAnimator` | Runtime status: current target, state, clip, logical frame, atlas index, elapsed time, loop count, and last issue |
| `AnimationClip::from_row` / `from_column` | Grid-layout helpers for extracting clips from row-based or column-based sprite sheets |
| `Easing` / `EasingVariety` | Rich easing curves (Linear, Quadratic, Cubic, Quartic, Quintic, Exponential, Circular, Sin, with In/Out/InOut) |
| `AnimationLibrary::from_aseprite_json` | Imports Aseprite-exported JSON into clips/states using tag ranges and per-frame durations |
| `AnimationEventFired` | Buffered message emitted when the runtime crosses a frame marker |
| `AnimationLooped` / `AnimationFinished` / `AnimationChanged` | Buffered lifecycle messages for loop completion, one-shot completion, and target/clip changes |

## Required Entity Components

The runtime expects:

- `SpritesheetAnimationSource`
- `AnimationController`
- `SpritesheetAnimator`
- `Sprite` with `sprite.texture_atlas = Some(TextureAtlas { layout, index, .. })`

`SpritesheetAnimationBundle` provides the three runtime components. The sprite image and atlas layout stay owned by the consumer.

## How Clips Are Defined

Supported v0.1 clip authoring:

- contiguous frame ranges via `AnimationClip::from_range`
- explicit atlas index lists via `AnimationClip::from_indices`
- fully custom frames via `AnimationClip::from_frames`
- Aseprite-exported JSON via `AnimationLibrary::from_aseprite_json("name", json_str)`

Clip playback can be configured with:

- `RepeatMode::Loop` or `RepeatMode::Once`
- `PlaybackDirection::Forward`, `Reverse`, or `PingPong`
- `FrameTiming::FramesPerSecond` or `SecondsPerFrame`
- per-frame duration overrides via `ClipFrame::with_duration_seconds`
- frame markers via `ClipFrame::with_event`

## How Transitions Are Configured

The crate keeps transitions small and explicit:

- requests target either a logical state or a clip
- `TransitionDefinition::requested` handles request-time rerouting or guarded exits
- `TransitionDefinition::finished` handles deterministic fallback after a one-shot completes
- guards are expressed with `minimum_elapsed_seconds` and optional `NormalizedTimeWindow`

No opaque graph editor or blend tree is involved in v0.1. Resolution is priority-ordered and deterministic.

## Control and Event Consumption

Use `AnimationController` for runtime control:

- set `requested_target` for steady-state selection
- use `play_state_once` / `play_clip_once` for one-shot requests
- use `pause`, `resume`, `stop`, `restart`, `seek_to_frame`, or `seek_to_normalized_time` for direct playback commands

Read buffered lifecycle messages with `MessageReader`:

```rust,ignore
fn consume_events(mut events: MessageReader<AnimationEventFired>) {
    for event in events.read() {
        if event.marker.name == "impact" {
            // trigger a sound, spawn a spark, advance a UI pulse, etc.
        }
    }
}
```

## Runtime Semantics

Important v0.1 behavior:

- `StartOffset::{EntitySeeded|Normalized}` is applied on first target selection and when a
  pending or explicit requested target resolves; finished-transition hops keep deterministic sequence
  starts.
- `PendingRequestPolicy::Replace` keeps only the newest blocked request
- `PendingRequestPolicy::KeepFirst` keeps the earliest blocked request until it resolves
- `PendingRequestPolicy::Discard` ignores new blocked requests while preserving any already queued target
- `SameTargetPolicy::Restart` only applies to explicit `Play(...)` commands, not passive `requested_target` reassignments
- frame markers fire when a frame is entered; they do not re-fire while playback is paused on the same frame

## Examples

Every shipped example uses **real sprite sheet assets** (Gabe, Mani, Kenney character, Kenney dungeon tiles) and includes `saddle-pane` controls for playback policy, timing, and live runtime monitors.

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Gabe idle/run selection driven by a simple motion signal | `cargo run -p saddle-animation-spritesheet-example-basic` |
| `character_animation` | Keyboard-controlled Gabe + auto-patrolling Mani with action one-shot | `cargo run -p saddle-animation-spritesheet-example-character-animation` |
| `state_machine` | Locked one-shot action with automatic fallback and lifecycle messages | `cargo run -p saddle-animation-spritesheet-example-state-machine` |
| `frame_events` | Event markers driving reactive beacon flash on impact | `cargo run -p saddle-animation-spritesheet-example-frame-events` |
| `directional` | Kenney character cycling through four directional clips | `cargo run -p saddle-animation-spritesheet-example-directional` |
| `crowd_variation` | Gabe and Mani alternating across a crowd with entity-seeded offsets | `cargo run -p saddle-animation-spritesheet-example-crowd-variation` |
| `easing_showcase` | Side-by-side comparison of all easing curves on the same clip | `cargo run -p saddle-animation-spritesheet-example-easing-showcase` |
| `ui_animation` | Animated UI ImageNode elements using Kenney dungeon tile sprite sheets | `cargo run -p saddle-animation-spritesheet-example-ui-animation` |
| `aseprite_import` | Embedded Aseprite JSON import over the Gabe sprite sheet | `cargo run -p saddle-animation-spritesheet-example-aseprite-import` |

## Crate-Local Lab

The richer verification app lives at `shared/animation/saddle-animation-spritesheet/examples/lab`:

```bash
cargo run -p saddle-animation-spritesheet-lab
```

Targeted E2E scenarios:

```bash
cargo run -p saddle-animation-spritesheet-lab --features e2e -- spritesheet_smoke
cargo run -p saddle-animation-spritesheet-lab --features e2e -- spritesheet_state_machine
cargo run -p saddle-animation-spritesheet-lab --features e2e -- spritesheet_frame_events
cargo run -p saddle-animation-spritesheet-lab --features e2e -- spritesheet_aseprite_import
```

## Limitations and Non-Goals

Current limitations:

- the runtime writes atlas indices for `Sprite` and `ImageNode` (UI elements are supported)
- transitions are instant in v0.1; there is no frame blending or cross-fade
- `from_aseprite_json` is intentionally lightweight: it imports frame durations and tag ranges, but does not own atlas-image generation or automatic event-marker authoring
- visibility-aware ticking is limited to `Always` vs `WhenVisible`

Intentional non-goals in v0.1:

- skeletal animation
- project-specific combat or platformer state machines
- a full editor/import pipeline
- owning sprite image loading or atlas-image generation

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
