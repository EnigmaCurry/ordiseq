//! # The Scale Omnibus
//!
//! [The Scale Omnibus](http://www.saxopedia.com/the-scale-omnibus/)
//! ([wayback](https://web.archive.org/web/20200220013047/http://www.saxopedia.com/the-scale-omnibus/))
//! is a book written by Francesco Balena, which is a catalouge of
//! musical scales and their intervals. The `scale_omnibus` crate
//! provides this data in the form of a Rust library.
//!
//! This library contains YAML data compiled by Corey Hoard:
//! [ioanszilagyi/scale_omnibus](https://github.com/ioanszilagyi/scale_omnibus)
//!
//! ## Features
//!
//! - More than 1000 musical scales.
//! - Retrieve scales directly by name.
//! - Search for scales based on any criteria, such as origin, name
//! substring match, or the number of intervals.
//!
//! ## Key Structures and Functions
//!
//! ### Structures
//! - [`Scale`]: Represents a musical scale with optional properties like intervals, notes, and origin.
//!
//! ### Functions
//! - [`get_scale`]: Retrieve a scale by name.
//! - [`get_scale_names`]: Get a list of all available scale names.
//! - [`filter_scales`]: Apply a custom filter to retrieve a subset of scales.
//! - [`find_scales_with_intervals_greater_than`]: Find scales with more than a specified number of intervals.
//! - [`find_scales_by_origin`]: Find scales associated with a specific origin.
//! - [`find_scales_with_up_down_intervals`]: Find scales that define both ascending and descending intervals.
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;

/// Represents a musical scale with various properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scale {
    /// The name of the scale (e.g., "Major", "Dorian").
    pub name: String,

    /// The intervals of the scale in semitones, if defined.
    ///
    /// Example: A Major scale might have intervals `[2, 2, 1, 2, 2, 2, 1]`.
    pub intervals: Option<Vec<u8>>,

    /// The non-standard ascending intervals of the scale in semitones, if distinct from the main intervals.
    ///
    /// Example: For an Enigmatic scale, this might be `[1, 3, 2, 2, 2, 1, 1]`.
    pub intervals_ascending: Option<Vec<u8>>,

    /// The non-standard descending intervals of the scale in semitones, if distinct from the main intervals.
    ///
    /// Example: For an Enigmatic scale, this might be `[1, 3, 1, 3, 2, 1, 1]`.
    pub intervals_descending: Option<Vec<u8>>,

    /// The notes of the scale, expressed as offset from root, if defined.
    ///
    /// Example: A C Major scale might have notes `[0, 2, 4, 5, 7, 9, 11]` corresponding to `C, D, E, F, G, A, B`.
    pub notes: Option<Vec<u8>>,

    /// The non-standard ascending notes of the scale, if distinct.
    ///
    /// Example: For an Enigmatic scale, this might be `[0, 1, 4, 6, 8, 10, 11]`.
    pub notes_ascending: Option<Vec<u8>>,

    /// The non-standard descending notes of the scale, if distinct.
    ///
    /// Example: For an Enigmatic scale, this might be `[0, 1, 4, 5, 8, 10, 11]`.
    pub notes_descending: Option<Vec<u8>>,

    /// The origin or cultural association of the scale, if available.
    ///
    /// Example: A scale might have an origin like `"Egypt"` or `"India"`.
    pub origin: Option<String>,
}

#[derive(Debug, Error)]
pub enum ScaleOmnibusError {
    #[error("Scale not found: {0}")]
    ScaleNotFoundError(String),
}

static SCALES: OnceLock<HashMap<String, Scale>> = OnceLock::new();
fn load_scales() -> &'static HashMap<String, Scale> {
    SCALES.get_or_init(|| {
        const SCALES_YAML: &str = include_str!("../data/scales.yaml");

        let scales: Vec<serde_yaml::Value> =
            serde_yaml::from_str(SCALES_YAML).expect("Invalid YAML format");

        let mut scale_map: HashMap<String, Scale> = HashMap::new();
        let mut name_counts: HashMap<String, usize> = HashMap::new();

        for item in scales {
            if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                let key = name.to_lowercase();

                let parse_optional_vec_u8 = |key: &str| -> Option<Vec<u8>> {
                    item.get(key)?
                        .as_sequence()?
                        .iter()
                        .filter_map(|n| n.as_u64().map(|n| n as u8))
                        .collect::<Vec<u8>>()
                        .into()
                };

                let mut scale = Scale {
                    name: name.to_string(),
                    intervals: parse_optional_vec_u8("intervals"),
                    intervals_ascending: parse_optional_vec_u8("intervals_ascending"),
                    intervals_descending: parse_optional_vec_u8("intervals_descending"),
                    notes: parse_optional_vec_u8("notes"),
                    notes_ascending: parse_optional_vec_u8("notes_ascending"),
                    notes_descending: parse_optional_vec_u8("notes_descending"),
                    origin: item
                        .get("origin")
                        .and_then(|o| o.as_str().map(|s| s.to_string())),
                };

                let mut unique_key = key.clone();
                if let Entry::Occupied(mut count) = name_counts.entry(key.clone()) {
                    *count.get_mut() += 1;
                    let suffix = format!(" ({})", count.get());
                    scale.name = format!("{}{}", name, suffix);
                    unique_key = format!("{}{}", key, suffix);
                } else {
                    name_counts.insert(key.clone(), 0);
                }

                scale_map.insert(unique_key, scale);
            }
        }

        scale_map
    })
}

/// Retrieves a musical scale by its exact name.
///
/// # Arguments
///
/// * `name` - The exact name of the scale to retrieve (case insensitive).
///
/// # Returns
///
/// * `Ok(&Scale)` if the scale is found.
/// * `Err(ScaleOmnibusError::ScaleNotFoundError)` if the scale is not found.
///
/// # Example
/// ```rust
/// let scale = scale_omnibus::get_scale("major").unwrap();
/// assert_eq!(scale.name, "Majorz");
/// assert_eq!(scale.intervals, Some(vec![2, 2, 1, 2, 2, 2, 1]));
/// ```
pub fn get_scale(name: &str) -> Result<&Scale, ScaleOmnibusError> {
    let scales = load_scales();
    scales
        .get(&name.to_lowercase())
        .ok_or_else(|| ScaleOmnibusError::ScaleNotFoundError(name.to_string()))
}

/// Returns a vector of all available scale names.
///
/// # Returns
///
/// A `Vec<String>` containing the names of all scales.
///
/// # Example
/// ```rust
/// // Get the names of all the scales in the database (lower case keys):
/// let names: Vec<String> = scale_omnibus::get_scale_names();
/// assert!(names.len() > 1000); // There's over 1000 scales.
/// assert!(names.contains(&"bebop major".to_string())); // keys are lowercase
/// ```
pub fn get_scale_names() -> Vec<String> {
    let scales = load_scales();
    scales.keys().cloned().collect()
}

/// Filters scales based on a provided closure.
///
/// # Arguments
///
/// * `filter` - A closure that takes a `&Scale` and returns `true` if the scale matches the filter.
///
/// # Returns
///
/// A `Vec<Scale>` containing the filtered scales.
///
/// # Example
/// ```rust
/// // Find all the scales with "Major" in the name:
/// let major_scales = scale_omnibus::filter_scales(|scale| scale.name.contains("Major")).unwrap();
/// // Bebop Major is among them:
/// assert!(major_scales.contains(&scale_omnibus::get_scale("Bebop Major").unwrap()));
///
/// // Find all the scales with 12 intervals:
/// let filtered_scales = scale_omnibus::filter_scales(|scale| {
///         scale
///             .intervals
///             .as_ref()
///             .map_or(false, |intervals| intervals.len() == 12)
/// }).unwrap();
/// // Well, there's only one of those:
/// assert_eq!(filtered_scales, vec![scale_omnibus::get_scale("Chromatic").unwrap().clone()]);
/// ```
pub fn filter_scales<F>(filter: F) -> Result<Vec<Scale>, ScaleOmnibusError>
where
    F: Fn(&Scale) -> bool,
{
    let scales = load_scales();
    Ok(scales
        .values()
        .filter(|&scale| filter(scale))
        .cloned()
        .collect::<Vec<Scale>>())
}

/// Finds scales with a minimum number of intervals
///
/// # Arguments
///
/// * `min_intervals` - The minimum number of intervals required.
///
/// # Returns
///
/// A `Vec<Scale>` containing the scales that match the criteria.
///
/// # Example
/// ```rust
/// // Find the largest scales, are there any with greater than 11 intervals?:
/// let complex_scales = scale_omnibus::find_scales_with_intervals_greater_than(11).unwrap();
/// // There's only one that fits that criteria (Chromatic):
/// assert_eq!(complex_scales, vec![scale_omnibus::get_scale("Chromatic").unwrap().clone()]);
/// ```
pub fn find_scales_with_intervals_greater_than(
    min_intervals: usize,
) -> Result<Vec<Scale>, ScaleOmnibusError> {
    filter_scales(|scale| {
        scale
            .intervals
            .as_ref()
            .map_or(false, |intervals| intervals.len() > min_intervals)
    })
}

/// Finds scales originating from a specified origin.
///
/// # Arguments
///
/// * `origin` - The origin of the scales to filter.
///
/// # Returns
///
/// A `Vec<Scale>` containing the scales that match the origin.
///
/// # Example
/// ```rust
/// // Get all the scales catalogued with the origin of Egypt:
/// let egyptian_scales = scale_omnibus::find_scales_by_origin("Egypt").unwrap();
/// // Niavent is among them:
/// assert!(egyptian_scales.contains(&scale_omnibus::get_scale("Niavent").unwrap()));
/// ```
pub fn find_scales_by_origin(origin: &str) -> Result<Vec<Scale>, ScaleOmnibusError> {
    filter_scales(|scale| {
        scale
            .origin
            .as_ref()
            .map_or(false, |o| o.to_lowercase() == origin.to_lowercase())
    })
}

/// Finds scales that have divergent ascending and descending intervals.
///
/// # Returns
///
/// A `Vec<Scale>` containing the scales that have different ascending and descending intervals defined.
///
/// # Example
/// ```rust
/// // Find all the scales that have "ascending_intervals" defined:
/// let dual_interval_scales = scale_omnibus::find_scales_with_up_down_intervals().unwrap();
/// // Raga Gowla is among them:
/// assert!(dual_interval_scales.contains(&scale_omnibus::get_scale("Raga Gowla").unwrap()));
/// ```
pub fn find_scales_with_up_down_intervals() -> Result<Vec<Scale>, ScaleOmnibusError> {
    filter_scales(|scale| scale.intervals_ascending.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_scale_names() -> Result<(), Box<dyn std::error::Error>> {
        let names = get_scale_names();
        let n = names.len();
        assert!(
            names.len() > 1000,
            "There are some missing scales, because only {n} scales were found."
        );
        assert!(names.contains(&"major".to_string()));
        assert!(names.contains(&"superlocrian #6".to_string()));
        Ok(())
    }

    #[test]
    fn test_get_scale() -> Result<(), Box<dyn std::error::Error>> {
        let bebop_major = get_scale("BeBop majoR")?; // input case doesnt matter
        assert_eq!(bebop_major.name, "Bebop Major");
        assert_eq!(bebop_major.intervals, Some(vec![2, 2, 1, 2, 1, 1, 2, 1]));
        assert_eq!(bebop_major.notes, Some(vec![0, 2, 4, 5, 7, 8, 9, 11]));
        assert_eq!(bebop_major.notes_ascending, None);
        assert_eq!(bebop_major.intervals_ascending, None);
        assert_eq!(bebop_major.notes_descending, None);
        assert_eq!(bebop_major.intervals_descending, None);
        Ok(())
    }

    #[test]
    fn test_edge_name() -> Result<(), Box<dyn std::error::Error>> {
        let superlocrian = get_scale("superlocrian #6")?;
        assert_eq!(superlocrian.name, "Superlocrian #6");
        assert_eq!(superlocrian.intervals, Some(vec![1, 2, 1, 2, 3, 1, 2]));
        assert_eq!(superlocrian.notes, Some(vec![0, 1, 3, 4, 6, 9, 10]));
        assert_eq!(superlocrian.notes_ascending, None);
        assert_eq!(superlocrian.intervals_ascending, None);
        assert_eq!(superlocrian.notes_descending, None);
        assert_eq!(superlocrian.intervals_descending, None);
        Ok(())
    }

    #[test]
    fn test_conflict_name() -> Result<(), Box<dyn std::error::Error>> {
        // There are a few name collisions in the scales.
        // This library handles them by giving unique parentheticals:
        let messiaen_2nd_one = get_scale("Messiaen 2nd Mode")?;
        let messiaen_2nd_two = get_scale("Messiaen 2nd Mode (1)")?;
        assert_eq!(messiaen_2nd_one.name, "Messiaen 2nd Mode");
        assert_eq!(messiaen_2nd_two.name, "Messiaen 2nd Mode (1)");
        Ok(())
    }

    #[test]
    fn test_notes_ascending() -> Result<(), Box<dyn std::error::Error>> {
        let enigmatic = get_scale("Enigmatic")?;
        assert_eq!(enigmatic.name, "Enigmatic");
        assert_eq!(enigmatic.intervals, None);
        assert_eq!(
            enigmatic.intervals_ascending,
            Some(vec![1, 3, 2, 2, 2, 1, 1])
        );
        assert_eq!(
            enigmatic.intervals_descending,
            Some(vec![1, 3, 1, 3, 2, 1, 1])
        );
        assert_eq!(enigmatic.notes_ascending, Some(vec![0, 1, 4, 6, 8, 10, 11]));
        assert_eq!(
            enigmatic.notes_descending,
            Some(vec![0, 1, 4, 5, 8, 10, 11])
        );
        assert_eq!(enigmatic.notes, None);
        Ok(())
    }

    #[test]
    fn test_filter_scales_by_name() -> Result<(), Box<dyn std::error::Error>> {
        // Find some scales with "major" in the name:
        let filtered_scales = filter_scales(|scale| scale.name.to_lowercase().contains("major"))?;

        let bebop_major = get_scale("Bebop major")?;
        let aeolian_major = get_scale("Aeolian Major")?;
        let major_pentatonic = get_scale("Major Pentatonic b7 #9")?;

        println!("{filtered_scales:?}");
        assert!(filtered_scales.contains(&bebop_major));
        assert!(filtered_scales.contains(&aeolian_major));
        assert!(filtered_scales.contains(&major_pentatonic));

        Ok(())
    }

    #[test]
    fn test_filter_scales_by_number_of_intervals() -> Result<(), Box<dyn std::error::Error>> {
        // The chromatic scale is the only scale with 12 intervals in it.
        // Filter all of the scales by intervals = 12 and we should find it:
        let filtered_scales = filter_scales(|scale| {
            scale
                .intervals
                .as_ref()
                .map_or(false, |intervals| intervals.len() == 12)
        })?;
        assert_eq!(
            filtered_scales.len(),
            1,
            "There should only be one scale with 12 intervals (chromatic)."
        );
        assert_eq!(&filtered_scales[0], get_scale("Chromatic")?);

        Ok(())
    }

    #[test]
    fn test_find_scales_with_intervals_greater_than() -> Result<(), Box<dyn std::error::Error>> {
        // Test for scales with more than 5 intervals
        let filtered_scales = find_scales_with_intervals_greater_than(5)?;

        // Dummy assertion: Replace with actual test data
        assert!(
            !filtered_scales.is_empty(),
            "No scales found with >5 intervals"
        );

        // Print the scales for debugging (optional)
        for scale in &filtered_scales {
            println!("Scale with >5 intervals: {:?}", scale.name);
        }

        Ok(())
    }

    #[test]
    fn test_find_scales_by_origin() -> Result<(), Box<dyn std::error::Error>> {
        // Test for scales originating from Egypt
        let filtered_scales = find_scales_by_origin("Egypt")?;

        // Dummy assertion: Replace with actual test data
        assert!(
            !filtered_scales.is_empty(),
            "No scales found originating from Egypt"
        );

        // Print the scales for debugging (optional)
        for scale in &filtered_scales {
            println!("Scale from Egypt: {:?}", scale.name);
        }

        Ok(())
    }

    #[test]
    fn test_find_scales_with_up_down_intervals() -> Result<(), Box<dyn std::error::Error>> {
        // Test for scales with ascending and descending intervals
        let filtered_scales = find_scales_with_up_down_intervals()?;

        assert!(
            !filtered_scales.is_empty(),
            "No scales found with different ascending and descending intervals"
        );

        for scale in &filtered_scales {
            assert_eq!(scale.intervals, None);
            assert_eq!(scale.notes, None);
        }

        Ok(())
    }
}
