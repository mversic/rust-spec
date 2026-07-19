//! Internal Representation (IR) of Rust types during conversion into FFI types.
//!
//! While you can implement [`crate::ExternC`] directly on your type, it is often
//! preferable to map it into IR by implementing [`Ir`]. This approach gives you
//! automatic, correct, and zero-cost conversions from IR to the equivalent C type.
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::{convert::Infallible, ops::Add};

use crate::TypeSpec;

/// Marker for a type that doesn't have a guaranteed representation and requires explicit conversion.
pub struct Unstable<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for a type that is transmuted to another type and thus delegates its conversion.
pub struct Stable<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for a robust [`crate::ReprC`] type that does not require conversion
pub enum Robust {}

/// Marker for a non-robust type that is still transmuted by the ABI layer.
pub enum NonRobust {}

#[cfg(feature = "alloc")]
unsafe impl<R> TypeSpec for Vec<R> {
    type Repr = crate::repr::Unstable<crate::repr::NonRobust>;
    type Size = crate::size::Sized<crate::size::NonZst>;
    type Niche = crate::niche::WithNiche<crate::niche::Custom>;
    type Mutability = crate::mutability::Exclusive;
}

impl Add for Robust {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl Add for crate::repr::NonRobust {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}

impl Add<crate::repr::NonRobust> for Robust {
    type Output = crate::repr::NonRobust;

    fn add(self, _: crate::repr::NonRobust) -> Self::Output {
        unreachable!()
    }
}

impl Add<Robust> for crate::repr::NonRobust {
    type Output = Self;

    fn add(self, _: Robust) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<crate::repr::Unstable<U>> for crate::repr::Stable<K>
where
    K: Add<U>,
{
    type Output = crate::repr::Unstable<<K as Add<U>>::Output>;

    fn add(self, _: crate::repr::Unstable<U>) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<crate::repr::Stable<U>> for crate::repr::Unstable<K>
where
    K: Add<U>,
{
    type Output = crate::repr::Unstable<<K as Add<U>>::Output>;

    fn add(self, _: crate::repr::Stable<U>) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<crate::repr::Unstable<U>> for crate::repr::Unstable<K>
where
    K: Add<U>,
{
    type Output = crate::repr::Unstable<<K as Add<U>>::Output>;

    fn add(self, _: crate::repr::Unstable<U>) -> Self::Output {
        unreachable!()
    }
}

impl<K, U> Add<crate::repr::Stable<U>> for crate::repr::Stable<K>
where
    K: Add<U>,
{
    type Output = crate::repr::Stable<<K as Add<U>>::Output>;

    fn add(self, _: crate::repr::Stable<U>) -> Self::Output {
        unreachable!()
    }
}
