//! Soundfont loading and discovery.

use super::error::SynthError;
use rustysynth::SoundFont;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Common system directories to search for soundfonts.
pub const SEARCH_DIRS: &[&str] = &[
    "/usr/share/soundfonts",
    "/usr/share/sounds/sf2",
    "/usr/share/sounds/sf3",
    "/usr/local/share/soundfonts",
    "/usr/local/share/sounds/sf2",
];

/// Preferred soundfont filenames in order of preference.
pub const PREFERRED_SOUNDFONTS: &[&str] = &[
    "FluidR3_GM.sf2",
    "FluidR3_GM.sf3",
    "default.sf2",
    "TimGM6mb.sf2",
    "GeneralUser_GS.sf2",
    "freepats-general-midi.sf2",
];

/// A loaded soundfont ready for synthesis.
///
/// This wraps `rustysynth::SoundFont` in an `Arc` for sharing across threads,
/// and tracks the path it was loaded from.
#[derive(Clone)]
pub struct SoundFontSource {
    /// The loaded soundfont.
    inner: Arc<SoundFont>,
    /// Path the soundfont was loaded from.
    path: PathBuf,
}

impl SoundFontSource {
    /// Load a soundfont from the given path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, SynthError> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path).map_err(|e| SynthError::SoundFontOpenError {
            path: path.clone(),
            source: e,
        })?;

        let soundfont =
            SoundFont::new(&mut file).map_err(|e| SynthError::SoundFontParseError {
                path: path.clone(),
                message: e.to_string(),
            })?;

        Ok(Self {
            inner: Arc::new(soundfont),
            path,
        })
    }

    /// Search for a soundfont in common system locations.
    ///
    /// This searches:
    /// 1. `SOUNDFONT_PATH` environment variable (if set, loads directly from path)
    /// 2. `~/soundfonts` (if home directory exists)
    /// 3. Common system directories (see [`SEARCH_DIRS`])
    ///
    /// Within each directory, it first looks for preferred soundfonts
    /// (see [`PREFERRED_SOUNDFONTS`]), then falls back to any `.sf2` or `.sf3` file.
    pub fn default_search() -> Result<Self, SynthError> {
        // Check environment variable first
        if let Ok(path) = std::env::var("SOUNDFONT_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Self::from_path(path);
            }
        }
        Self::search(&Self::default_search_dirs())
    }

    /// Search for a soundfont in the specified directories.
    pub fn search(dirs: &[PathBuf]) -> Result<Self, SynthError> {
        if let Some(path) = find_soundfont_in_dirs(dirs) {
            Self::from_path(path)
        } else {
            Err(SynthError::NoSoundFontFound {
                searched: dirs.to_vec(),
            })
        }
    }

    /// Get the default search directories.
    pub fn default_search_dirs() -> Vec<PathBuf> {
        let mut dirs: Vec<PathBuf> = Vec::new();

        // Add ~/soundfonts if home directory exists
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("soundfonts"));
        }

        // Add system directories
        for dir in SEARCH_DIRS {
            dirs.push(PathBuf::from(dir));
        }

        dirs
    }

    /// Get the path this soundfont was loaded from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get a reference to the inner soundfont.
    pub fn inner(&self) -> &Arc<SoundFont> {
        &self.inner
    }
}

impl std::fmt::Debug for SoundFontSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundFontSource")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

/// Search for a soundfont file in the given directories.
///
/// Returns the path to the first soundfont found, or `None` if no soundfont was found.
pub fn find_soundfont_in_dirs(search_dirs: &[PathBuf]) -> Option<PathBuf> {
    // First, try to find preferred soundfonts
    for name in PREFERRED_SOUNDFONTS {
        for dir in search_dirs {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // Fall back to any .sf2 or .sf3 file in search directories
    for dir in search_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "sf2" || ext == "sf3" {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

/// Find a default soundfont using the default search directories.
///
/// This is a convenience function equivalent to calling
/// `find_soundfont_in_dirs(&SoundFontSource::default_search_dirs())`.
pub fn find_default_soundfont() -> Option<PathBuf> {
    find_soundfont_in_dirs(&SoundFontSource::default_search_dirs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_search_dirs_includes_home() {
        let dirs = SoundFontSource::default_search_dirs();
        if dirs::home_dir().is_some() {
            assert!(dirs.iter().any(|d| d.ends_with("soundfonts")));
        }
    }

    #[test]
    fn test_default_search_dirs_includes_system() {
        let dirs = SoundFontSource::default_search_dirs();
        assert!(dirs.iter().any(|d| d == Path::new("/usr/share/soundfonts")));
    }
}
