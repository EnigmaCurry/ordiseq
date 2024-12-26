//! # Time
//!
//! This module provides functionality to handle time signatures in musical contexts.
//! It includes a `TimeSignature` struct to represent time signatures, parsing capabilities,
//! and a function to calculate a recommended ticks per quarter note (TPQN) for MIDI files.

use crate::error::OrdiseqError;
use std::convert::TryFrom;
use std::fmt;

/// Represents time in ticks within a (MIDI) sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub ticks: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Represents a musical time signature.
///
/// A time signature consists of two parts:
/// - `beats_per_bar`: The numerator, representing the number of beats in one measure.
/// - `beat_unit`: The denominator, indicating the note value that represents one beat
///   (e.g., 1=whole, 2=half, 4=quarter, 8=eighth, etc.).
pub struct TimeSignature {
    pub beats_per_bar: u8,
    pub beat_unit: u8,
    pub ticks_per_quarter_note: u32,
}

impl TimeSignature {
    /// Creates a new `TimeSignature` instance by parsing a string.
    ///
    /// # Arguments
    ///
    /// * `input` - A string representation of the time signature in "numerator/denominator" format.
    ///
    /// # Returns
    ///
    /// * `TimeSignature` - A new instance of `TimeSignature`
    ///
    /// # Panics
    ///
    /// This function will return an error if:
    /// - The input string is not in the correct "numerator/denominator" format.
    /// - The numerator or denominator cannot be parsed as a valid number.
    /// - The denominator is not a power of two.
    pub fn new(ts_str: &str, ticks_per_quarter_note: u32) -> Result<Self, OrdiseqError> {
        let parts: Vec<&str> = ts_str.split('/').collect();
        if parts.len() != 2 {
            return Err(OrdiseqError::InvalidTimeSignature(
                "Input must be in the format 'numerator/denominator'".to_string(),
            ));
        }

        let beats_per_bar: u8 = parts[0].parse().map_err(|_| {
            OrdiseqError::InvalidTimeSignature("Numerator must be a valid number".to_string())
        })?;

        let beat_unit: u8 = parts[1].parse().map_err(|_| {
            OrdiseqError::InvalidTimeSignature("Denominator must be a valid number".to_string())
        })?;

        if !beat_unit.is_power_of_two() {
            return Err(OrdiseqError::InvalidTimeSignature(
                "Denominator must be a power of two".to_string(),
            ));
        }

        Ok(TimeSignature {
            beats_per_bar,
            beat_unit,
            ticks_per_quarter_note,
        })
    }

    /// Calculate the length of one bar in ticks
    ///
    /// # Returns
    /// The length (Time) of one bar
    pub fn bar_time(&self) -> Time {
        let quarter_note_ratio = 4.0 / self.beat_unit as f32;
        let quarter_notes_per_bar = self.beats_per_bar as f32 * quarter_note_ratio;
        Time {
            ticks: (quarter_notes_per_bar * self.ticks_per_quarter_note as f32) as u32,
        }
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.beats_per_bar, self.beat_unit)
    }
}

/// Calculates a recommended TPQN (ticks per quarter note) for a given time signature.
/// Returns a value that is a multiple of common beat subdivisions.
#[allow(dead_code)]
pub fn calculate_tpqn(time_signature: TimeSignature) -> Option<u16> {
    // Validate input: numerator > 0 and denominator > 0 and is a power of 2
    if time_signature.beats_per_bar == 0
        || time_signature.beat_unit == 0
        || (time_signature.beat_unit & (time_signature.beat_unit - 1)) != 0
    {
        return None; // Invalid time signature
    }

    // Determine the base TPQN (e.g., for a standard 4/4, TPQN = 96 or 480)
    let base_tpqn = 96; // Common starting value for TPQN in MIDI files

    // Adjust for non-quarter note time_signature.beat_units
    // A quarter note is represented by a time_signature.beat_unit of 4
    let adjusted_tpqn = match time_signature.beat_unit {
        1 => base_tpqn * 4,  // Whole note
        2 => base_tpqn * 2,  // Half note
        4 => base_tpqn,      // Quarter note
        8 => base_tpqn / 2,  // Eighth note
        16 => base_tpqn / 4, // Sixteenth note
        _ => return None,    // Unsupported time_signature.beat_unit
    };

    // Make sure the result is divisible by common beat subdivisions (e.g., 12, 24, 48)
    Some(adjusted_tpqn * time_signature.beats_per_bar as u16)
}

/// Return a common time 4/4 time signature with 96 ticks per quarter note
pub fn common_time() -> TimeSignature {
    TimeSignature::new("4/4", 96).expect("Expected common 4/4 time signature")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_time_signature() {
        let ts = TimeSignature::new("4/4").unwrap();
        assert_eq!(
            ts,
            TimeSignature {
                beats_per_bar: 4,
                beat_unit: 4
            }
        );
    }

    #[test]
    fn test_new_valid_odd_time_signature() {
        let ts = TimeSignature::new("7/8").unwrap();
        assert_eq!(
            ts,
            TimeSignature {
                beats_per_bar: 7,
                beat_unit: 8
            }
        );
    }

    #[test]
    fn test_tpqn_standard_time_signature() {
        let ts = TimeSignature::new("4/4").unwrap();
        assert_eq!(calculate_tpqn(ts), Some(384)); // 96 * 4 = 384
    }

    #[test]
    fn test_tpqn_eighth_note_time_signature() {
        let ts = TimeSignature::new("3/8").unwrap();
        assert_eq!(calculate_tpqn(ts), Some(144)); // 48 * 3 = 144
    }

    #[test]
    fn test_tpqn_half_note_time_signature() {
        let ts = TimeSignature::new("6/2").unwrap();
        assert_eq!(calculate_tpqn(ts), Some(1152)); // 192 * 6 = 1152
    }

    #[test]
    fn test_tpqn_whole_note_time_signature() {
        let ts = TimeSignature::new("1/1").unwrap();
        assert_eq!(calculate_tpqn(ts), Some(384)); // 384 * 1 = 384
    }

    #[test]
    fn test_tpqn_sixteenth_note_time_signature() {
        let ts = TimeSignature::new("7/16").unwrap();
        assert_eq!(calculate_tpqn(ts), Some(168)); // 24 * 7 = 168
    }

    #[test]
    fn test_tpqn_invalid_time_signature_beat_unit_not_power_of_two() {
        let ts = TimeSignature {
            beats_per_bar: 4,
            beat_unit: 3,
        };
        assert_eq!(calculate_tpqn(ts), None);
    }

    #[test]
    fn test_tpqn_invalid_time_signature_zero_beats_per_bar() {
        let ts = TimeSignature {
            beats_per_bar: 0,
            beat_unit: 4,
        };
        assert_eq!(calculate_tpqn(ts), None);
    }

    #[test]
    fn test_tpqn_invalid_time_signature_zero_beat_unit() {
        let ts = TimeSignature {
            beats_per_bar: 4,
            beat_unit: 0,
        };
        assert_eq!(calculate_tpqn(ts), None);
    }

    #[test]
    fn test_new_invalid_format() {
        let result = TimeSignature::new("4-4");
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Invalid time signature: Input must be in the format 'numerator/denominator'"
        );
    }

    #[test]
    fn test_new_invalid_numerator() {
        let result = TimeSignature::new("a/4");
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Invalid time signature: Numerator must be a valid number"
        );
    }

    #[test]
    fn test_new_invalid_denominator() {
        let result = TimeSignature::new("4/b");
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Invalid time signature: Denominator must be a valid number"
        );
    }

    #[test]
    fn test_new_invalid_denominator_not_power_of_two() {
        let result = TimeSignature::new("4/3");
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Invalid time signature: Denominator must be a power of two"
        );
    }
}
