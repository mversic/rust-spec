//! Mutability classification of Rust types.

use core::ops::Add;

/// Marker for types whose whole ABI-exposed value may be mutated through shared access.
pub enum Interior {}

/// Marker for types whose ABI-exposed value requires exclusive access to mutate.
pub enum Exclusive {}

impl<K> Add<K> for Exclusive {
    type Output = Self;

    fn add(self, _: K) -> Self::Output {
        unreachable!()
    }
}

impl<K> Add<K> for Interior {
    type Output = K;

    fn add(self, _: K) -> Self::Output {
        unreachable!()
    }
}
