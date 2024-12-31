pub fn generate_euclidean_rhythm(steps: usize, pulses: usize, velocity: f64) -> Vec<(bool, f64)> {
    if steps == 0 {
        return vec![]; // No steps, empty rhythm
    }

    if pulses == 0 {
        return vec![(false, 0.0); steps]; // No pulses, all false
    }

    if pulses >= steps {
        return vec![(true, velocity); steps]; // More or equal pulses than steps, all true
    }

    // Create rhythm using Euclidean distribution
    let mut rhythm = vec![(false, 0.0); steps];
    for i in 0..steps {
        if (i * pulses) % steps < pulses {
            rhythm[i] = (true, velocity); // Add a hit
        }
    }

    rhythm
}

// fn build_pattern(pattern: &mut Vec<bool>, counts: &[usize], remainder: &[usize], level: usize) {
//     if level == -1_isize as usize {
//         pattern.push(false);
//     } else if level == 0 {
//         pattern.push(true);
//     } else {
//         for _ in 0..counts[level] {
//             build_pattern(pattern, counts, remainder, level - 1);
//         }
//         if remainder[level] > 0 {
//             build_pattern(pattern, counts, remainder, level - 2);
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::generate_euclidean_rhythm;

    #[test]
    fn test_even_distribution() {
        // Test with 8 steps and 3 pulses
        let steps = 8;
        let pulses = 3;
        let velocity = 0.8;

        let result = generate_euclidean_rhythm(steps, pulses, velocity);
        let expected = vec![
            (true, 0.8),
            (false, 0.0),
            (false, 0.0),
            (true, 0.8),
            (false, 0.0),
            (false, 0.0),
            (true, 0.8),
            (false, 0.0),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_full_pulses() {
        // Test where pulses equal steps (every step is a hit)
        let steps = 8;
        let pulses = 8;
        let velocity = 1.0;

        let result = generate_euclidean_rhythm(steps, pulses, velocity);
        let expected = vec![
            (true, 1.0),
            (true, 1.0),
            (true, 1.0),
            (true, 1.0),
            (true, 1.0),
            (true, 1.0),
            (true, 1.0),
            (true, 1.0),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_pulses() {
        // Test where there are no pulses (no hits)
        let steps = 8;
        let pulses = 0;
        let velocity = 0.5;

        let result = generate_euclidean_rhythm(steps, pulses, velocity);
        let expected = vec![
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_single_pulse() {
        // Test where there is only one pulse
        let steps = 8;
        let pulses = 1;
        let velocity = 0.7;

        let result = generate_euclidean_rhythm(steps, pulses, velocity);
        let expected = vec![
            (true, 0.7),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
            (false, 0.0),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_more_pulses_than_steps() {
        // Test where pulses exceed steps
        let steps = 8;
        let pulses = 10;
        let velocity = 0.9;

        let result = generate_euclidean_rhythm(steps, pulses, velocity);
        let expected = vec![
            (true, 0.9),
            (true, 0.9),
            (true, 0.9),
            (true, 0.9),
            (true, 0.9),
            (true, 0.9),
            (true, 0.9),
            (true, 0.9),
        ]; // All steps should be true since pulses exceed steps.

        assert_eq!(result, expected);
    }

    #[test]
    fn test_edge_case_one_step() {
        // Test with 1 step
        let steps = 1;
        let pulses = 1;
        let velocity = 1.0;

        let result = generate_euclidean_rhythm(steps, pulses, velocity);
        let expected = vec![(true, 1.0)]; // Single step, single pulse.

        assert_eq!(result, expected);
    }
}
