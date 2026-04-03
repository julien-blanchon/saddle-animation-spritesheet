# Architecture

## Data Model

The crate splits authoring data from runtime state:

- `AnimationLibrary` is the authoring asset: clips, states, transitions, and optional default target
- `AnimationClip` and `ClipFrame` describe atlas indices, playback timing, and frame markers
- `AnimationState` names a logical state and can override clip defaults without duplicating clip data
- `AnimationController` is the external request surface
- `SpritesheetAnimator` is the runtime status surface exposed to other systems
- an internal `AnimatorRuntime` component stores the active sequence cursor, per-frame accumulator, and buffered messages
- an internal `AnimationLibraryCaches` resource resolves validated libraries into playback-ready tables

`AnimationLibrary` can be authored manually in Rust or imported from Aseprite JSON via `AnimationLibrary::from_aseprite_json`. The importer is deliberately narrow: it turns tag ranges and frame durations into the crate's native clip/state model, but it does not own atlas generation or sprite image loading.

Consumers own the atlas image handle and `TextureAtlasLayout` handle. The crate only resolves clip-to-atlas indices and writes the chosen atlas index back to `Sprite`.

## System Ordering

The runtime is deliberately staged through public system sets:

1. `ResolveRequests`
2. `AdvanceTime`
3. `ApplyTransitions`
4. `EmitEvents`
5. `WriteSpriteFrame`

That split keeps behavior predictable and gives callers clear ordering hooks with `configure_sets`.

### `ResolveRequests`

This phase:

- activates newly added players
- invalidates library caches when `AssetEvent<AnimationLibrary>` arrives
- validates the current library handle
- applies direct controller commands (`Pause`, `Resume`, `Play`, `Seek`, etc.)
- resolves requested targets against non-interruptible clips and request-time transitions
- creates or updates the active playback selection

### `AdvanceTime`

This phase:

- reads `Time::delta()`
- applies `SpritesheetAnimator::speed_multiplier`
- optionally skips ticking when `AnimationTickPolicy::WhenVisible` says the entity should stay frozen
- advances through the resolved playback sequence
- emits frame-marker messages as frames are entered
- emits loop and finished lifecycle messages

Progression is deterministic with variable `delta`: the runtime repeatedly consumes the remaining frame budget until the incoming time slice is exhausted.

### `ApplyTransitions`

This phase only runs after a clip finishes during the current tick. It evaluates:

1. finished transitions from the current target
2. any queued pending request
3. current requested/default target fallback

Finished transitions therefore win over queued requests when both exist.

### `EmitEvents`

Frame markers and lifecycle notifications are buffered in the runtime component first, then flushed in a dedicated phase. This keeps message ordering stable relative to request resolution and frame writes.

### `WriteSpriteFrame`

This phase writes `SpritesheetAnimator::atlas_index` to `Sprite.texture_atlas.index` when:

- the entity has a `Sprite`
- the sprite has a `TextureAtlas`
- the atlas layout asset exists
- the atlas index is within layout bounds

Failures are non-panicking and reported through `SpritesheetAnimator::last_issue`.

## Clip Resolution Flow

At cache-build time, the crate resolves each target into a playback table:

- target (`state` or `clip`)
- resolved clip id
- resolved playback settings
- the concrete sequence of runtime steps
- per-step prefix durations
- total cycle duration

`PlaybackDirection::PingPong` is expanded into a concrete sequence:

- looping ping-pong avoids duplicating the starting endpoint on the return leg
- one-shot ping-pong includes the return-to-start step so the clip visibly comes back before finishing

## Transition Flow

The state model is intentionally compact:

- current target
- requested target
- optional queued pending target
- transition definitions
- per-clip interrupt policy

Resolution order during a request:

1. if the target matches the current target, `SameTargetPolicy` decides whether to keep progress or restart
2. if the current playback is locked and not finished, the request is queued according to `PendingRequestPolicy`
3. otherwise the crate looks for the highest-priority matching requested transition
4. if no matching transition exists, the requested target is used directly

Resolution order after a clip finishes:

1. highest-priority matching finished transition
2. queued pending target
3. requested/default target fallback

## Event Emission Semantics

The crate emits buffered Bevy messages:

- `AnimationEventFired`
- `AnimationLooped`
- `AnimationFinished`
- `AnimationChanged`

Semantics:

- frame markers fire when the runtime enters a frame
- markers on the initial frame fire when a clip is selected
- paused playback does not re-fire the same marker
- large `delta` values can cross multiple frames in one update; each crossed frame emits at most one marker in order
- `AnimationChanged` is emitted whenever target selection changes or an explicit restart reselects the current target

## Sprite Writeback Semantics

The runtime does not mutate clip definitions or atlas layouts at runtime. It only writes:

- `SpritesheetAnimator` status fields
- `Sprite.texture_atlas.index`

No per-entity allocations occur on the hot path other than the message payloads already buffered inside the runtime component.

## Missing Asset and Hot Reload Behavior

When the library handle is missing or invalid:

- the runtime does not panic
- `SpritesheetAnimator::last_issue` is updated
- the player remains inert until a valid library becomes available

When an `AnimationLibrary` asset changes:

- the cache entry is invalidated on `AssetEvent<AnimationLibrary>`
- the next request or tick rebuilds the cached playback data
- active runtimes are sanitized against the rebuilt playback sequence before further advancement

The crate does not own image or atlas-layout loading, so missing sprite-atlas prerequisites are reported as issues instead of triggering internal asset loads.
