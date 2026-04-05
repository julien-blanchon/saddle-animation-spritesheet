use bevy::prelude::*;
use std::f32::consts::PI;

/// Easing variety (curve shape) for non-linear easing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum EasingVariety {
    Quadratic,
    Cubic,
    Quartic,
    Quintic,
    Exponential,
    Circular,
    Sin,
}

/// Easing function applied to animation playback timing.
///
/// Easing remaps the linear progress `[0, 1]` through a curve, changing which
/// frames are displayed for longer or shorter durations. This does **not**
/// interpolate between frames — it redistributes the time spent on each frame.
///
/// - `In(variety)`: Slow start, fast finish
/// - `Out(variety)`: Fast start, slow finish
/// - `InOut(variety)`: Slow at both ends
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub enum Easing {
    #[default]
    Linear,
    In(EasingVariety),
    Out(EasingVariety),
    InOut(EasingVariety),
}

impl Easing {
    /// Apply the easing function to a normalized time value `t` in `[0, 1]`.
    ///
    /// Returns the eased value, also in `[0, 1]`.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::In(variety) => ease_in(variety, t),
            Self::Out(variety) => ease_out(variety, t),
            Self::InOut(variety) => ease_in_out(variety, t),
        }
    }
}

fn ease_in(variety: EasingVariety, t: f32) -> f32 {
    match variety {
        EasingVariety::Quadratic => t * t,
        EasingVariety::Cubic => t * t * t,
        EasingVariety::Quartic => t * t * t * t,
        EasingVariety::Quintic => t * t * t * t * t,
        EasingVariety::Exponential => {
            if t <= 0.0 {
                0.0
            } else {
                (2.0_f32).powf(10.0 * (t - 1.0))
            }
        }
        EasingVariety::Circular => 1.0 - (1.0 - t * t).sqrt(),
        EasingVariety::Sin => 1.0 - (t * PI / 2.0).cos(),
    }
}

fn ease_out(variety: EasingVariety, t: f32) -> f32 {
    1.0 - ease_in(variety, 1.0 - t)
}

fn ease_in_out(variety: EasingVariety, t: f32) -> f32 {
    if t < 0.5 {
        ease_in(variety, t * 2.0) / 2.0
    } else {
        1.0 - ease_in(variety, (1.0 - t) * 2.0) / 2.0
    }
}

#[cfg(test)]
#[path = "easing_tests.rs"]
mod tests;
