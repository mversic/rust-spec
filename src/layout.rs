//! Representation stability and trap/robustness classification.
//!
//! This module provides the marker types used by [`crate::RustSpec::Layout`].
//! Stability describes whether Rust guarantees the representation shape.
//! Robustness describes whether the represented value space has trap values that
//! require validity care.
use core::ops::Add;

use crate::{Stable, Unstable};

/// Closed family of representation-stability classifications.
#[sealed::sealed]
pub trait LayoutKind {}

/// Marker for a robust type that does not require validity conversion.
pub enum Robust {}

/// Marker for a non-robust type that is still transmuted by the ABI layer.
pub enum NonRobust {}

#[sealed::sealed]
impl LayoutKind for Stable {}

#[sealed::sealed]
impl LayoutKind for Unstable {}

impl<K> Add<K> for NonRobust {
    type Output = Self;

    fn add(self, _: K) -> Self::Output {
        unreachable!()
    }
}

impl Add<NonRobust> for Robust {
    type Output = NonRobust;

    fn add(self, _: NonRobust) -> Self::Output {
        unreachable!()
    }
}

impl Add for Robust {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl<K> Add<K> for Unstable {
    type Output = Self;

    fn add(self, _: K) -> Self::Output {
        unreachable!()
    }
}

impl Add<Unstable> for Stable {
    type Output = Unstable;

    fn add(self, _: Unstable) -> Self::Output {
        unreachable!()
    }
}

impl Add for Stable {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}
