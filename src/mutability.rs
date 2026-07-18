//! Mutability classification of Rust types.

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
use core::ops::Add;

use disjoint_impls::disjoint_impls;

use crate::{
    niche::{NicheFamily, WithNiche, WithoutNiche},
    size::{MetaSized, NonZst, SizeFamily, Thin, Zst},
};

/// Marker for types whose whole ABI-exposed value may be mutated through shared access.
pub enum Interior {}

/// Marker for types whose ABI-exposed value requires exclusive access to mutate.
pub enum Exclusive {}

disjoint_impls! {
    /// Classifies whether shared access may mutate the whole ABI-exposed value.
    pub trait MutabilityFamily {
        /// Mutability classification marker for this type.
        ///
        /// - Set to [`Interior`] if shared access may mutate the whole ABI-exposed value.
        /// - Set to [`Exclusive`] if mutation requires exclusive access.
        type Kind;
    }

    #[cfg(feature = "alloc")]
    impl<R: SizeFamily<Kind: Thin>> MutabilityFamily for Box<R>
    where
        R: MutabilityFamily,
    {
        type Kind = <R as MutabilityFamily>::Kind;
    }

    #[cfg(feature = "alloc")]
    impl<R: SizeFamily<Kind = MetaSized<S>> + ?Sized, S> MutabilityFamily for Box<R> {
        type Kind = Exclusive;
    }

    impl<R: MutabilityFamily + NicheFamily<Kind = WithNiche<N>>, N> MutabilityFamily for Option<R>
    {
        type Kind = <R as MutabilityFamily>::Kind;
    }
    impl<R: NicheFamily<Kind = WithoutNiche>> MutabilityFamily for Option<R>
    {
        type Kind = Exclusive;
    }

    impl<
        R: SizeFamily<Kind = crate::size::Sized<K>>,
        E: SizeFamily<Kind = crate::size::Sized<K>>,
        K
    > MutabilityFamily for Result<R, E>
    {
        type Kind = Exclusive;
    }
    impl<
        R: SizeFamily<Kind = crate::size::Sized<NonZst>> + NicheFamily<Kind = WithNiche<N>> + MutabilityFamily,
        E: SizeFamily<Kind = crate::size::Sized<Zst>>,
        N,
    > MutabilityFamily for Result<R, E>
    {
        type Kind = <R as MutabilityFamily>::Kind;
    }
    impl<
        R: SizeFamily<Kind = crate::size::Sized<NonZst>> + NicheFamily<Kind = WithoutNiche>,
        E: SizeFamily<Kind = crate::size::Sized<Zst>>,
    > MutabilityFamily for Result<R, E>
    where
        R: MutabilityFamily,
    {
        type Kind = Exclusive;
    }
    impl<
        R: SizeFamily<Kind = crate::size::Sized<Zst>>,
        E: SizeFamily<Kind = crate::size::Sized<NonZst>> + NicheFamily<Kind = WithNiche<N>> + MutabilityFamily,
        N,
    > MutabilityFamily for Result<R, E>
    {
        type Kind = <E as MutabilityFamily>::Kind;
    }
    impl<
        R: SizeFamily<Kind = crate::size::Sized<Zst>>,
        E: SizeFamily<Kind = crate::size::Sized<NonZst>> + NicheFamily<Kind = WithoutNiche>,
    > MutabilityFamily for Result<R, E>
    where
        E: MutabilityFamily,
    {
        type Kind = Exclusive;
    }
}

impl<R: MutabilityFamily + ?Sized> MutabilityFamily for &R {
    type Kind = R::Kind;
}

impl<R: MutabilityFamily + ?Sized> MutabilityFamily for &mut R {
    type Kind = R::Kind;
}

#[cfg(feature = "alloc")]
impl<R> MutabilityFamily for Vec<R> {
    type Kind = Exclusive;
}

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
