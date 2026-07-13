pub fn apply_gain(samples: &mut [f32], gain: f32) {
    for s in samples.iter_mut() {
        *s *= gain;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unity_gain_no_change() {
        let mut samples = vec![0.5, -0.5, 0.25, -0.25];
        let original = samples.clone();
        apply_gain(&mut samples, 1.0);
        assert_eq!(samples, original);
    }

    #[test]
    fn half_gain() {
        let mut samples = vec![0.5, -0.5, 0.25, -0.25];
        let original = samples.clone();
        apply_gain(&mut samples, 0.5);
        for (a, b) in samples.iter().zip(original.iter()) {
            assert!((a - b * 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn zero_gain_silences() {
        let mut samples = vec![0.5, -0.5, 0.25, -0.25];
        apply_gain(&mut samples, 0.0);
        for s in &samples {
            assert_eq!(*s, 0.0);
        }
    }

    #[test]
    fn empty_buffer_no_panic() {
        let mut empty: Vec<f32> = vec![];
        apply_gain(&mut empty, 2.0);
    }
}
