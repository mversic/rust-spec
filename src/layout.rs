//! Internal Representation (IR) of Rust types during conversion into FFI types.
//!
//! While you can implement [`crate::ExternC`] directly on your type, it is often
//! preferable to map it into IR by implementing [`Ir`]. This approach gives you
//! automatic, correct, and zero-cost conversions from IR to the equivalent C type.
use core::{convert::Infallible, ops::Add};

/// Marker for a type that doesn't have a guaranteed representation and requires explicit conversion.
pub struct Unstable<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for a type that is transmuted to another type and thus delegates its conversion.
pub struct Stable<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for a robust [`crate::ReprC`] type that does not require conversion.
pub enum Robust {}

/// Marker for a non-robust type that is still transmuted by the ABI layer.
pub enum NonRobust {}

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
