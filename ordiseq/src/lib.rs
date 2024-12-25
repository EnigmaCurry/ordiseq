//! # Ordiseq
//!
//! [![Latest Version](https://img.shields.io/crates/v/ordiseq.svg)](https://crates.io/crates/ordiseq)
//! [![Rust Documentation](https://docs.rs/ordiseq/badge.svg)](https://docs.rs/ordiseq)
//!
//! ALPHA: _ordiseq_ is an experimental MIDI sequencer library for Rust.
//!
//! ## Features
//!
//!   * Integrated with the
//!   [kord](https://crates.io/crates/kord)::[klib](https://docs.rs/kord/latest/klib/)
//!   crate for music theory, handling
//!   [`NamedPitches`](crate::prelude::NamedPitch),
//!   [`Notes`](crate::prelude::Note),
//!   [`Octaves`](crate::prelude::Octave),
//!   [`Intervals`](crate::prelude::Interval),
//!   [`Chords`](crate::prelude::Chord) and more.
//!
//!   * Includes more than 1000 musical
//!   [`Scales`](crate::prelude::Scale) from [The Scale
//! Omnibus](https://www.saxopedia.com/the-scale-omnibus)
//! ([wayback](https://web.archive.org/web/20200220013047/http://www.saxopedia.com/the-scale-omnibus/))
//!
//! ## Getting started
//!
//! ## Examples
pub mod prelude;
pub mod scales;
pub mod sequence;
pub mod util;
