use bevy::prelude::*;

pub(crate) fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

pub(crate) fn entity_seeded_normalized(entity: Entity) -> f32 {
    let bits = entity.to_bits();
    let scrambled =
        bits.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(17) ^ 0xD1B5_4A32_D192_ED03;
    let normalized = (scrambled & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32;
    clamp01(normalized)
}
