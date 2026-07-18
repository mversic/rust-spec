//! Logic related to the conversion of [`Option<T>`] to and from FFI-compatible representation

use core::{convert::Infallible, ops::Add};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

use disjoint_impls::disjoint_impls;

use crate::size::{MetaSized, NonZst, SizeFamily, Thin, Zst};

/// Marker for a type that has a niche value.
pub struct WithNiche<K>(core::marker::PhantomData<K>, Infallible);

/// Marker for a single stable (compiler guaranteed) niche value (e.g. `&u32`).
///
/// Only a handful of [`crate::ir::ReprC`] types have a stable niche
pub enum Stable {}

/// Marker for a custom defined (by this crate) niche (e.g. `[NonZeroU8; 2]`).
pub enum Custom {}

/// Marker for a type that has no trap representations and therefore no niche value
pub enum WithoutNiche {}

disjoint_impls! {
    /// Niche kind of the type in the internal representation [IR](`crate::ir::Repr`)
    pub trait NicheFamily {
        /// The niche class of the type.
        ///
        /// - If `Self` doesn't have any niche value, set [`NicheFamily::Kind`] to [`WithoutNiche`].
        ///   `Option<T>` will be serialized as [`crate::option::ReprCOption`]
        ///
        /// - If `Self` has a compiler guaranteed niche value, set [`NicheFamily::Kind`] to [`WithNiche<crate::niche::Stable>`].
        ///   `Option<T>` will be blindly transmuted into the underlying [`ReprC`] type
        ///
        /// - Otherwise, if `Self` has at least one trap, set [`NicheFamily::Kind`] to [`WithNiche<crate::niche::Custom>`].
        ///   `Option<T>` will be serialized into a [`T::CType`] with a manually set niche value
        type Kind;
    }

    impl<R: NicheFamily<Kind = WithoutNiche>> NicheFamily for [R] {
        type Kind = WithoutNiche;
    }
    impl<R: NicheFamily<Kind = WithNiche<K>>, K> NicheFamily for [R] {
        type Kind = WithNiche<crate::niche::Custom>;
    }

    impl<R: SizeFamily<Kind = MetaSized<K>> + ?Sized, K> NicheFamily for &R {
        type Kind = WithNiche<crate::niche::Custom>;
    }
    impl<R: SizeFamily<Kind: Thin> + ?Sized> NicheFamily for &R {
        type Kind = WithNiche<crate::niche::Stable>;
    }

    impl<R: SizeFamily<Kind = MetaSized<K>> + ?Sized, K> NicheFamily for &mut R {
        type Kind = WithNiche<crate::niche::Custom>;
    }
    impl<R: SizeFamily<Kind: Thin> + ?Sized> NicheFamily for &mut R {
        type Kind = WithNiche<crate::niche::Stable>;
    }

    #[cfg(feature = "alloc")]
    impl<R: SizeFamily<Kind = MetaSized<K>> + ?Sized, K> NicheFamily for Box<R> {
        type Kind = WithNiche<crate::niche::Custom>;
    }
    #[cfg(feature = "alloc")]
    impl<R: SizeFamily<Kind = crate::size::Sized<S>>, S> NicheFamily for Box<R> {
        type Kind = WithNiche<crate::niche::Stable>;
    }

    impl<R: NicheFamily<Kind = WithoutNiche>, const N: usize> NicheFamily for [R; N] {
        type Kind = WithoutNiche;
    }
    impl<R: NicheFamily<Kind = WithNiche<K>>, K, const N: usize> NicheFamily for [R; N] {
        type Kind = WithNiche<crate::niche::Custom>;
    }

    impl<R: NicheFamily<Kind = WithoutNiche>> NicheFamily for Option<R> {
        type Kind = WithNiche<crate::niche::Custom>;
    }
    impl<R: NicheFamily<Kind = WithNiche<crate::niche::Stable>>> NicheFamily for Option<R> {
        type Kind = WithoutNiche;
    }
    // TODO: IMHO compiler should be able to resolve circular dependencies here, but it doesn't work for now so I've bounded previous with Niche
    // This issue could be of some help: https://github.com/mversic/co3/issues/33. This seems to be a limitation of the compiler known as
    // circular/cyclic resolution or (co)inductive cycle. The case shown here creates a cycle but only one solution is possible afaik
    //impl<R: NicheFamily<Kind = WithNiche<crate::niche::Custom>>> NicheFamily for Option<R> where Option<Self>: ReprFamily<Kind: ReprRustOrTransmutedNonRobust> {
    //    type Kind = WithNiche<crate::niche::Custom>;
    //}
    //impl<R: NicheFamily<Kind = WithNiche<crate::niche::Custom>>> NicheFamily for Option<R> where Option<Self>: ReprFamily<Kind = Unstable> {
    //    type Kind = WithoutNiche;
    //}
    //impl<R: NicheFamily<Kind = WithNiche<crate::niche::Custom>>> NicheFamily for Option<R> where Self: Niche {
    //    type Kind = WithNiche<crate::niche::Custom>;
    //}

    impl<
        R: NicheFamily<Kind = WithoutNiche>,
        E: NicheFamily<Kind = WithoutNiche>,
    > NicheFamily for Result<R, E> {
        type Kind = WithNiche<crate::niche::Custom>;
    }
    impl<
        R: SizeFamily<Kind = crate::size::Sized<NonZst>> + NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        E: SizeFamily<Kind = crate::size::Sized<Zst>> + NicheFamily,
    > NicheFamily for Result<R, E> {
        type Kind = WithoutNiche;
    }
    // FIXME:
    //impl<
    //    R: SizeFamily<Kind = crate::size::Sized<Zst>> + NicheFamily,
    //    E: SizeFamily<Kind = crate::size::Sized<NonZst>> + NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
    //> NicheFamily for Result<R, E> {
    //    type Kind = WithoutNiche;
    //}
    // TODO: Implement for all niche optimized Results
}

#[cfg(feature = "alloc")]
impl<R> NicheFamily for Vec<R> {
    type Kind = WithNiche<crate::niche::Custom>;
}

impl NicheFamily for Option<bool> {
    type Kind = WithNiche<crate::niche::Custom>;
}
impl NicheFamily for Option<Option<bool>> {
    type Kind = WithNiche<crate::niche::Custom>;
}

impl Add for WithoutNiche {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        unreachable!()
    }
}
impl<K> Add<WithNiche<K>> for WithoutNiche {
    type Output = WithNiche<crate::niche::Custom>;

    fn add(self, _: WithNiche<K>) -> Self::Output {
        unreachable!()
    }
}
impl<K> Add<WithoutNiche> for WithNiche<K> {
    type Output = Self;

    fn add(self, _: WithoutNiche) -> Self::Output {
        unreachable!()
    }
}
impl<K, U> Add<WithNiche<U>> for WithNiche<K> {
    type Output = WithNiche<crate::niche::Custom>;

    fn add(self, _: WithNiche<U>) -> Self::Output {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZero;

    use static_assertions::assert_impl_all;

    use super::*;
    use crate::repr::{ReprFamily, Unstable};

    #[test]
    fn nested_option_niche_family() {
        assert_impl_all!(Option<bool>:
            ReprFamily<Kind = Unstable>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );

        assert_impl_all!(Option<Option<bool>>:
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
            ReprFamily<Kind = Unstable>,
        );

        assert_impl_all!(Option<(u8, NonZero<u8>)>:
            ReprFamily<Kind = Unstable>,
            // TODO: Depends on: https://github.com/mversic/co3/issues/33
            //NicheFamily<Kind = WithoutNiche>,
            //Niche<CType = ReprCTuple2<u8, u8>>,
        );
    }
}
