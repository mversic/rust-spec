//! Mutability classification of Rust types.

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
use core::ops::Add;

use disjoint_impls::disjoint_impls;

use crate::{
    TypeSpec,
    niche::{WithNiche, WithoutNiche},
    size::{MetaSized, NonZst, Thin, Zst},
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
    impl<R: TypeSpec<Size: Thin>> MutabilityFamily for Box<R> {
        type Kind = <R as TypeSpec>::Mutability;
    }

    #[cfg(feature = "alloc")]
    impl<R: TypeSpec<Size = MetaSized<S>> + ?Sized, S> MutabilityFamily for Box<R> {
        type Kind = Exclusive;
    }

    impl<R: TypeSpec<Niche = WithNiche<N>>, N> MutabilityFamily for Option<R>
    {
        type Kind = <R as TypeSpec>::Mutability;
    }
    impl<R: TypeSpec<Niche = WithoutNiche>> MutabilityFamily for Option<R>
    {
        type Kind = Exclusive;
    }

    impl<
        R: TypeSpec<Size = crate::size::Sized<K>>,
        E: TypeSpec<Size = crate::size::Sized<K>>,
        K
    > MutabilityFamily for Result<R, E>
    {
        type Kind = Exclusive;
    }
    impl<
        R: TypeSpec<Size = crate::size::Sized<NonZst>, Niche = WithNiche<N>>,
        E: TypeSpec<Size = crate::size::Sized<Zst>>,
        N,
    > MutabilityFamily for Result<R, E>
    {
        type Kind = <R as TypeSpec>::Mutability;
    }
    impl<
        R: TypeSpec<Size = crate::size::Sized<NonZst>, Niche = WithoutNiche>,
        E: TypeSpec<Size = crate::size::Sized<Zst>>,
    > MutabilityFamily for Result<R, E>
    {
        type Kind = Exclusive;
    }
    impl<
        R: TypeSpec<Size = crate::size::Sized<Zst>>,
        E: TypeSpec<Size = crate::size::Sized<NonZst>, Niche = WithNiche<N>>,
        N,
    > MutabilityFamily for Result<R, E>
    {
        type Kind = <E as TypeSpec>::Mutability;
    }
    impl<
        R: TypeSpec<Size = crate::size::Sized<Zst>>,
        E: TypeSpec<Size = crate::size::Sized<NonZst>, Niche = WithoutNiche>,
    > MutabilityFamily for Result<R, E>
    {
        type Kind = Exclusive;
    }
}

impl<R: TypeSpec + ?Sized> MutabilityFamily for &R {
    type Kind = R::Mutability;
}

impl<R: TypeSpec + ?Sized> MutabilityFamily for &mut R {
    type Kind = R::Mutability;
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
