# `saddle-animation-spritesheet-lab`

Crate-local verification app for `saddle-animation-spritesheet`.

The lab keeps everything inside the shared crate boundary:

- one hero driven by the logical state machine
- one looping prop using a separate library
- a crowd strip using entity-seeded start offsets and speed variation
- an on-screen diagnostics overlay for BRP and screenshot checks

## Run

```bash
cargo run -p saddle-animation-spritesheet-lab
```

## E2E Scenarios

```bash
cargo run -p saddle-animation-spritesheet-lab --features e2e -- spritesheet_smoke
cargo run -p saddle-animation-spritesheet-lab --features e2e -- spritesheet_state_machine
cargo run -p saddle-animation-spritesheet-lab --features e2e -- spritesheet_frame_events
cargo run -p saddle-animation-spritesheet-lab --features e2e -- spritesheet_aseprite_import
```

## BRP / Debug

The lab exposes BRP extras on port `15712` by default when built with the default `dev` feature:

```bash
brp app launch saddle-animation-spritesheet-lab
brp world query bevy_ecs::name::Name bevy_sprite::Sprite
brp extras screenshot /tmp/saddle_animation_spritesheet_lab.png
brp extras shutdown
```

Useful named entities:

- `Lab Hero`
- `Animated Prop`
- `Crowd Member 1` ... `Crowd Member 9`
- `Overlay`

## Notes

- The atlas image is generated procedurally at runtime, so the lab does not depend on external art assets.
- `SPRITESHEET_AUTO_EXIT_SECONDS=3 cargo run -p saddle-animation-spritesheet-lab` is useful for short non-interactive smoke launches.
