//! Representation stability and robustness classification.
//!
//! This module provides the marker types used by [`crate::RustSpec::Layout`].
//! Stability describes whether Rust guarantees the representation shape.
//! Robustness describes whether the represented value space has trap values that
//! require validity care.
use core::{convert::Infallible, ops::Add};

/// Closed family of representation-layout classifications.
#[sealed::sealed]
pub trait LayoutSpec {}

/// Closed family of trap-representation classifications.
#[sealed::sealed]
pub trait TrapSpec {}

/// Marker for a type that doesn't have a guaranteed representation and requires explicit conversion.
pub struct Unstable<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for a type that is transmuted to another type and thus delegates its conversion.
pub struct Stable<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for a robust type that does not require validity conversion.
pub enum Robust {}

/// Marker for a non-robust type that is still transmuted by the ABI layer.
pub enum NonRobust {}

#[sealed::sealed]
impl TrapSpec for Robust {}

#[sealed::sealed]
impl TrapSpec for NonRobust {}

#[sealed::sealed]
impl<K: TrapSpec> LayoutSpec for Unstable<K> {}

#[sealed::sealed]
impl<K: TrapSpec> LayoutSpec for Stable<K> {}

impl Add for Robust {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl Add for NonRobust {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl Add<NonRobust> for Robust {
    type Output = NonRobust;

    fn add(self, _: NonRobust) -> Self::Output {
        unreachable!()
    }
}

impl Add<Robust> for NonRobust {
    type Output = Self;

    fn add(self, _: Robust) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<Unstable<U>> for Stable<K>
where
    K: Add<U>,
{
    type Output = Unstable<<K as Add<U>>::Output>;

    fn add(self, _: Unstable<U>) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<Stable<U>> for Unstable<K>
where
    K: Add<U>,
{
    type Output = Unstable<<K as Add<U>>::Output>;

    fn add(self, _: Stable<U>) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<Unstable<U>> for Unstable<K>
where
    K: Add<U>,
{
    type Output = Unstable<<K as Add<U>>::Output>;

    fn add(self, _: Unstable<U>) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<Stable<U>> for Stable<K>
where
    K: Add<U>,
{
    type Output = Stable<<K as Add<U>>::Output>;

    fn add(self, _: Stable<U>) -> Self::Output {
        unreachable!()
    }
}
