# Configuration

This document covers the public configuration surface in v0.1.

## `AnimationLibrary`

Top-level authoring asset.

| Field | Default | Valid values | Runtime effect |
| --- | --- | --- | --- |
| `name` | `""` | any string | Diagnostic label for logs and debugging |
| `default_target` | `None` | existing state or clip target | Fallback selection when the controller does not request a target |
| `clips` | empty | unique clip ids | Defines all concrete frame sequences |
| `states` | empty | unique state ids that reference existing clips | Adds logical names and playback overrides |
| `transitions` | empty | valid target references and guard windows | Controls request-time and finish-time rerouting |

Validation rules:

- the library cannot be empty unless a valid `default_target` exists
- clip ids and state ids must be unique
- transition targets must exist
- `minimum_elapsed_seconds` must be non-negative
- exit windows must stay within `0.0 ..= 1.0`

## `AnimationClip`

Concrete frame animation definition.

| Field | Default | Valid values | Runtime effect |
| --- | --- | --- | --- |
| `id` | none | unique clip id | Runtime clip identity and message payload |
| `frames` | none | at least one frame | Ordered logical frames before direction expansion |
| `timing` | `FramesPerSecond(12.0)` | finite positive fps or seconds/frame | Default timing used for frames without per-frame overrides |
| `repeat` | `Loop` | `Loop`, `Once` | Whether the sequence repeats or finishes |
| `direction` | `Forward` | `Forward`, `Reverse`, `PingPong` | Expands the runtime playback sequence |
| `interrupt_policy` | `Interruptible` | `Interruptible`, `LockUntilFinished` | Whether blocked requests are allowed to interrupt the clip |

Additional validation:

- `frames` must not be empty
- `LockUntilFinished` is invalid when the resolved playback loops forever

## `ClipFrame`

Per-frame authoring entry.

| Field | Default | Valid values | Runtime effect |
| --- | --- | --- | --- |
| `atlas_index` | none | any `usize` that the consumer's layout supports | Index written into `Sprite.texture_atlas.index` |
| `duration_seconds` | `None` | positive finite `f32` | Overrides clip-level timing for that logical frame |
| `events` | empty | any number of markers | Emits `AnimationEventFired` when the frame is entered |

## `FrameTiming`

Uniform clip timing mode.

| Variant | Default | Valid values | Runtime effect |
| --- | --- | --- | --- |
| `FramesPerSecond(f32)` | default variant | finite value `> 0.0` | Converts fps into seconds per frame |
| `SecondsPerFrame(f32)` | none | finite value `> 0.0` | Uses fixed per-frame duration directly |

## `AnimationState`

Logical state that points at a clip and optionally overrides playback defaults.

| Field | Default | Valid values | Runtime effect |
| --- | --- | --- | --- |
| `id` | none | unique state id | Stable logical name for requests and transitions |
| `clip` | none | existing clip id | Underlying clip used by the state |
| `playback` | all overrides unset | valid override values | Changes timing, repeat mode, direction, or interrupt policy without duplicating clip data |

## `PlaybackOverride`

Per-state override block.

| Field | Default | Valid values | Runtime effect |
| --- | --- | --- | --- |
| `timing` | `None` | valid `FrameTiming` | Replaces clip timing |
| `repeat` | `None` | valid `RepeatMode` | Replaces clip repeat mode |
| `direction` | `None` | valid `PlaybackDirection` | Replaces clip direction |
| `interrupt_policy` | `None` | valid `InterruptPolicy` | Replaces clip interrupt behavior |

## `TransitionDefinition`

Explicit transition rule.

| Field | Default | Valid values | Runtime effect |
| --- | --- | --- | --- |
| `source` | none | `Any`, state source, or clip source | Which active target can trigger the rule |
| `target` | none | existing state or clip target | The resolved target if the rule matches |
| `trigger` | `Requested` from constructor helpers | `Requested`, `Finished` | Whether the rule applies during a request or after completion |
| `priority` | `0` | any `i32` | Higher priority wins among matching rules |
| `minimum_elapsed_seconds` | `0.0` | finite `>= 0.0` | Blocks the rule until enough clip time has elapsed |
| `exit_window` | `None` | normalized window within `0.0 ..= 1.0` | Restricts the rule to a normalized progress range |

## `AnimationController`

External control surface attached to each animated entity.

| Field | Default | Runtime effect |
| --- | --- | --- |
| `default_target` | `None` | Per-entity default selection before the library fallback is used |
| `requested_target` | `None` | Steady-state desired target |
| `pending_target` | `None` | Internally or externally queued target waiting for a lock to release |
| `pending_request_policy` | `Replace` | How blocked requests update `pending_target` |
| `same_target_policy` | `KeepProgress` | How explicit play requests behave when they target the current clip/state |
| `start_offset` | `StartOffset::None` | Applied when selecting a target from scratch or when an explicit target request is accepted |
| `command` | `AnimationControlCommand::None` | One-frame command channel for pause/resume/restart/seek/play |

### `PendingRequestPolicy`

| Variant | Effect |
| --- | --- |
| `Replace` | Keep only the newest blocked request |
| `KeepFirst` | Preserve the earliest blocked request until it resolves |
| `Discard` | Ignore new blocked requests while preserving any already queued target |

### `SameTargetPolicy`

| Variant | Effect |
| --- | --- |
| `KeepProgress` | Ignore explicit play requests that target the current selection |
| `Restart` | Reselect the current target and restart from the beginning |

### `StartOffset`

| Variant | Effect |
| --- | --- |
| `None` | Start at the first runtime step |
| `Normalized(f32)` | Seek to the requested normalized position on initial selection |
| `EntitySeeded` | Choose a stable per-entity normalized start offset for crowd variation |

### `AnimationControlCommand`

| Variant | Effect |
| --- | --- |
| `Play(AnimationTarget)` | Explicit play request, subject to interrupt and same-target rules |
| `Pause` / `Resume` | Toggles playback state without changing the selected clip |
| `Stop` | Stops playback and rewinds the active target to its first frame |
| `Restart` | Re-selects the current target from the start |
| `SeekFrame(usize)` | Jumps to a logical frame index, clamped to the clip length |
| `SeekNormalized(f32)` | Jumps to a normalized position in the resolved playback cycle |

## `SpritesheetAnimator`

Runtime status component.

| Field | Default | Runtime meaning |
| --- | --- | --- |
| `playback_state` | `Stopped` | Current runtime state (`Stopped`, `Playing`, `Paused`, `Finished`) |
| `speed_multiplier` | `1.0` | Multiplies incoming `delta` before advancement |
| `visibility_policy` | `Always` | Chooses whether off-screen entities continue ticking |
| `current_target` / `current_state` / `current_clip` | `None` | Current resolved selection |
| `current_frame` | `0` | Current logical frame index |
| `atlas_index` | `0` | Current atlas frame index written to the sprite |
| `normalized_time` | `0.0` | Normalized progress within the resolved runtime cycle |
| `elapsed_seconds` | `0.0` | Total time spent in the current playback selection |
| `frame_elapsed_seconds` | `0.0` | Time spent on the current runtime step |
| `completed_loops` | `0` | Number of completed loops for looping playback |
| `last_issue` | `None` | Non-panicking issue surface for missing assets or invalid runtime state |

### `AnimationTickPolicy`

| Variant | Effect |
| --- | --- |
| `Always` | Advance regardless of visibility |
| `WhenVisible` | Skip time advancement when visibility says the entity is hidden |

## Runtime Message Payloads

Buffered messages emitted by the crate:

- `AnimationEventFired`
- `AnimationLooped`
- `AnimationFinished`
- `AnimationChanged`

All messages include the entity id plus the resolved target and clip context for the event.
