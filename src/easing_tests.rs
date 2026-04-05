use super::*;

#[test]
fn linear_is_identity() {
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        assert!((Easing::Linear.apply(t) - t).abs() < f32::EPSILON);
    }
}

#[test]
fn all_easings_are_zero_at_zero() {
    for variety in [
        EasingVariety::Quadratic,
        EasingVariety::Cubic,
        EasingVariety::Quartic,
        EasingVariety::Quintic,
        EasingVariety::Exponential,
        EasingVariety::Circular,
        EasingVariety::Sin,
    ] {
        assert!(
            Easing::In(variety).apply(0.0).abs() < 0.01,
            "In({variety:?}) at 0"
        );
        assert!(
            Easing::Out(variety).apply(0.0).abs() < 0.01,
            "Out({variety:?}) at 0"
        );
        assert!(
            Easing::InOut(variety).apply(0.0).abs() < 0.01,
            "InOut({variety:?}) at 0"
        );
    }
}

#[test]
fn all_easings_are_one_at_one() {
    for variety in [
        EasingVariety::Quadratic,
        EasingVariety::Cubic,
        EasingVariety::Quartic,
        EasingVariety::Quintic,
        EasingVariety::Exponential,
        EasingVariety::Circular,
        EasingVariety::Sin,
    ] {
        assert!(
            (Easing::In(variety).apply(1.0) - 1.0).abs() < 0.01,
            "In({variety:?}) at 1"
        );
        assert!(
            (Easing::Out(variety).apply(1.0) - 1.0).abs() < 0.01,
            "Out({variety:?}) at 1"
        );
        assert!(
            (Easing::InOut(variety).apply(1.0) - 1.0).abs() < 0.01,
            "InOut({variety:?}) at 1"
        );
    }
}

#[test]
fn in_out_symmetry_at_midpoint() {
    for variety in [
        EasingVariety::Quadratic,
        EasingVariety::Cubic,
        EasingVariety::Quartic,
        EasingVariety::Quintic,
        EasingVariety::Circular,
        EasingVariety::Sin,
    ] {
        let mid = Easing::InOut(variety).apply(0.5);
        assert!(
            (mid - 0.5).abs() < 0.01,
            "InOut({variety:?}) at 0.5 = {mid}"
        );
    }
}

#[test]
fn ease_in_quadratic_known_values() {
    let ease = Easing::In(EasingVariety::Quadratic);
    assert!((ease.apply(0.5) - 0.25).abs() < f32::EPSILON);
}

#[test]
fn clamps_out_of_range() {
    let ease = Easing::In(EasingVariety::Cubic);
    assert!(ease.apply(-0.5).abs() < f32::EPSILON);
    assert!((ease.apply(1.5) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn monotonically_increasing() {
    for variety in [
        EasingVariety::Quadratic,
        EasingVariety::Cubic,
        EasingVariety::Quartic,
        EasingVariety::Quintic,
        EasingVariety::Circular,
        EasingVariety::Sin,
    ] {
        for easing in [
            Easing::In(variety),
            Easing::Out(variety),
            Easing::InOut(variety),
        ] {
            let mut prev = easing.apply(0.0);
            for i in 1..=20 {
                let t = i as f32 / 20.0;
                let val = easing.apply(t);
                assert!(
                    val >= prev - f32::EPSILON,
                    "{easing:?} not monotonic at t={t}: {prev} -> {val}"
                );
                prev = val;
            }
        }
    }
}
