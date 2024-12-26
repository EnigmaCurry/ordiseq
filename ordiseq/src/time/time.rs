//use crate::error::OrdiseqError;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// Represents time in ticks within a (MIDI) sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub ticks: u32,
}

// Implement addition of Time and any type that can be added to u32
impl<T> Add<T> for Time
where
    T: Into<u32>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Time {
            ticks: self.ticks + rhs.into(),
        }
    }
}

// Implement subtraction of Time and any type that can be subtracted from u32
impl<T> Sub<T> for Time
where
    T: Into<u32>,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Time {
            ticks: self.ticks.saturating_sub(rhs.into()),
        } // Prevent underflow
    }
}

// Implement multiplication of Time and any type that can be multiplied with u32
impl<T> Mul<T> for Time
where
    T: Into<u32>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Time {
            ticks: self.ticks * rhs.into(),
        }
    }
}

// Implement division of Time and any type that can be divided with u32
impl<T> Div<T> for Time
where
    T: Into<u32>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let divisor = rhs.into();
        if divisor == 0 {
            panic!("Attempt to divide by zero");
        }
        Time {
            ticks: self.ticks / divisor,
        }
    }
}

// Implement addition assignment for Time and any type that can be added to u32
impl<T> AddAssign<T> for Time
where
    T: Into<u32>,
{
    fn add_assign(&mut self, rhs: T) {
        self.ticks += rhs.into();
    }
}

// Implement subtraction assignment for Time and any type that can be subtracted from u32
impl<T> SubAssign<T> for Time
where
    T: Into<u32>,
{
    fn sub_assign(&mut self, rhs: T) {
        self.ticks = self.ticks.saturating_sub(rhs.into());
    }
}

// Implement multiplication assignment for Time and any type that can be multiplied with u32
impl<T> MulAssign<T> for Time
where
    T: Into<u32>,
{
    fn mul_assign(&mut self, rhs: T) {
        self.ticks *= rhs.into();
    }
}

// Implement division assignment for Time and any type that can be divided with u32
impl<T> DivAssign<T> for Time
where
    T: Into<u32>,
{
    fn div_assign(&mut self, rhs: T) {
        let divisor = rhs.into();
        if divisor == 0 {
            panic!("Attempt to divide by zero");
        }
        self.ticks /= divisor;
    }
}
