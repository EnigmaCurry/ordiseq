//use crate::error::OrdiseqError;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// Represents time in ticks within a (MIDI) sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub ticks: u32,
}

// Adding ticks to Time
impl Add<u32> for Time {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self {
            ticks: self.ticks + rhs,
        }
    }
}

// Subtracting ticks from Time
impl Sub<u32> for Time {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self {
            ticks: self.ticks.saturating_sub(rhs),
        }
    }
}

// Adding ticks to Time in-place
impl AddAssign<u32> for Time {
    fn add_assign(&mut self, rhs: u32) {
        self.ticks += rhs;
    }
}

// Subtracting ticks from Time in-place
impl SubAssign<u32> for Time {
    fn sub_assign(&mut self, rhs: u32) {
        self.ticks = self.ticks.saturating_sub(rhs);
    }
}

// Multiplying Time by a float
impl Mul<f32> for Time {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            ticks: (self.ticks as f32 * rhs).round() as u32,
        }
    }
}

// Dividing Time by a float
impl Div<f32> for Time {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            ticks: (self.ticks as f32 / rhs).round() as u32,
        }
    }
}

// Multiplying Time by a float in-place
impl MulAssign<f32> for Time {
    fn mul_assign(&mut self, rhs: f32) {
        self.ticks = (self.ticks as f32 * rhs).round() as u32;
    }
}

// Dividing Time by a float in-place
impl DivAssign<f32> for Time {
    fn div_assign(&mut self, rhs: f32) {
        self.ticks = (self.ticks as f32 / rhs).round() as u32;
    }
}
