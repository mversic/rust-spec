//! Compile-time classification of Rust types according to Rust-specified type properties.
//!
//! # Classification Axes
//!
//! `RustSpec` describes a type across the following axes:
//! - [`layout`]: layout stability and robustness.
//! - [`size`]: statically sized, metadata-sized, or extern-type-like shape.
//! - [`niche`]: whether a type has a stable, unstable, or no niche value.
//! - [`mutability`]: whether the whole value can be mutated through shared access.
//!
//! ## Layout
//!
//! Describes whether a type has a stable layout or trap/invalid values:
//! - [`layout::Stable<Robust>`]: stable layout with no trap values.
//! - [`layout::Stable<NonRobust>`]: stable layout, but with trap values.
//! - [`layout::Unstable<Robust>`]: unstable layout with no trap values.
//! - [`layout::Unstable<NonRobust>`]: unstable layout, but with trap values.
//!
//! A supported pointer's layout stability follows its pointee, because a Rust
//! reference or `Box` requires a valid pointee. Its robustness still describes
//! the pointer value itself. Raw pointers are not followed.
//!
//! ## Size
//!
//! Describes compile-time size and pointer metadata shape:
//! - [`size::Sized<size::Zst>`]: compile-time known zero-sized type.
//! - [`size::Sized<size::NonZst>`]: compile-time known non-zero-sized type.
//! - [`size::MetaSized<size::SliceLike>`]: dynamically sized slice-like type.
//! - [`size::MetaSized<size::DynTraitLike>`]: dynamically sized trait-object-like type.
//! - [`size::ExternTypeLike`]: dynamically sized extern-type-like type.
//!
//! ## Niche
//!
//! Describes whether and what kind of niche is available for the type:
//! - [`niche::WithoutNiche`]: no niche is available.
//! - [`niche::WithNiche<niche::Stable>`]: compiler-guaranteed niche.
//! - [`niche::WithNiche<niche::Unstable>`]: niche exists but is not guaranteed.
//!
//! ## Mutability
//!
//! Describes whether shared access (`&R`) can mutate the whole value:
//! - [`mutability::Interior`]: the whole value may be mutated through shared access.
//! - [`mutability::Exclusive`]: mutation of the value requires exclusive access.
//!
//! ## How to Use
//!
//! Derive `RustSpec` for your types, then use its associated marker families as bounds when implementing other traits:
//!
//! ```rust
//! use disjoint_impls::disjoint_impls;
//! use rust_spec::{
//!     RustSpec,
//!     niche::{self, WithNiche, WithoutNiche}
//! };
//!
//! #[derive(RustSpec)]
//! struct Header {
//!     id: u32,
//!     flags: u16,
//! }
//!
//! disjoint_impls! {
//!     trait NullableEncoding {
//!         const NEEDS_TAG: bool;
//!     }
//!
//!     impl<T: RustSpec<Niche = WithoutNiche>> NullableEncoding for T {
//!         const NEEDS_TAG: bool = true;
//!     }
//!
//!     impl<T: RustSpec<Niche = WithNiche<niche::Stable>>> NullableEncoding for T {
//!         const NEEDS_TAG: bool = false;
//!     }
//!
//!     impl<T: RustSpec<Niche = WithNiche<niche::Unstable>>> NullableEncoding for T {
//!         const NEEDS_TAG: bool = false;
//!     }
//! }
//!
//! const HEADER_OPTION_NEEDS_TAG: bool = Header::NEEDS_TAG;
//! ```
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as rust_spec;

use core::ops::Add;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use disjoint_impls::disjoint_impls;
#[cfg(feature = "derive")]
pub use rust_spec_derive::RustSpec;

use crate::{
    layout::{NonRobust, Robust},
    niche::{WithNiche, WithoutNiche},
    size::{MetadataKind, SizedKind},
};

mod core_impls;
pub mod layout;
pub mod mutability;
pub mod niche;
mod primitives;
pub mod size;
mod tuple;

disjoint_impls! {
    /// Joined Rust-spec classification of a type.
    ///
    /// This is the canonical surface for Rust-specified properties used by
    /// conversion.
    ///
    /// # Safety
    ///
    /// Implementors must classify `Self` truthfully. In particular, [`Self::Size`]
    /// carries the same safety requirements documented by the size marker types.
    pub unsafe trait RustSpec {
        /// Representation stability and robustness classification.
        type Layout;

        /// Statically known, metadata-sized, or extern-type-like size classification.
        type Size;

        /// Niche availability classification.
        type Niche;

        /// Shared-access mutability classification.
        type Mutability;

        /// Accumulator for layout reachable through one or more supported pointer indirections.
        ///
        /// A supported pointer's indirect layout is its pointee's complete
        /// layout. Structural types join the indirect layouts of their stored
        /// fields. Raw pointers are not followed.
        #[doc(hidden)]
        type __IndirectLayout;
    }

    unsafe impl<R: ?Sized, K: layout::TrapKind> RustSpec for &R
    where
        R: RustSpec<Layout = layout::Unstable<K>, Size: size::Thin>,
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
        type __IndirectLayout = R::Layout;
    }
    unsafe impl<R: ?Sized, K: layout::TrapKind> RustSpec for &R
    where
        R: RustSpec<Layout = layout::Stable<K>, Size: size::Thin>,
    {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
        type __IndirectLayout = R::Layout;
    }
    unsafe impl<R: ?Sized, U: MetadataKind> RustSpec for &R
    where
        R: RustSpec<Size = size::MetaSized<U>>,
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = R::Mutability;
        type __IndirectLayout = R::Layout;
    }

    unsafe impl<R: ?Sized, K: layout::TrapKind> RustSpec for &mut R
    where
        R: RustSpec<Layout = layout::Unstable<K>, Size: size::Thin>,
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
        type __IndirectLayout = R::Layout;
    }
    unsafe impl<R: ?Sized, K: layout::TrapKind> RustSpec for &mut R
    where
        R: RustSpec<Layout = layout::Stable<K>, Size: size::Thin>,
    {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
        type __IndirectLayout = R::Layout;
    }
    unsafe impl<R: ?Sized, U: MetadataKind> RustSpec for &mut R
    where
        R: RustSpec<Size = size::MetaSized<U>>,
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = R::Mutability;
        type __IndirectLayout = R::Layout;
    }

    #[cfg(feature = "alloc")]
    unsafe impl<R: ?Sized, K: layout::TrapKind> RustSpec for Box<R>
    where
        R: RustSpec<Layout = layout::Unstable<K>, Size: size::Thin>,
    {
        type Layout = layout::Unstable<NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
        type __IndirectLayout = R::Layout;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R, S: SizedKind, K: layout::TrapKind> RustSpec for Box<R>
    where
        R: RustSpec<Layout = layout::Stable<K>, Size = size::Sized<S>>,
    {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
        type __IndirectLayout = R::Layout;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: ?Sized, U: MetadataKind> RustSpec for Box<R>
    where
        R: RustSpec<Size = size::MetaSized<U>>,
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::Layout;
    }

    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithoutNiche>,
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }
    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithNiche<niche::Unstable>, __IndirectLayout = layout::Stable<Robust>>,
    {
        // FIXME: It can also become Stable<Robust> WithoutNiche if R has exactly one trap
        // The same can happen even if R has a stable niche. The whole Option can be Robust
        // We would have to support niche counting for this to work
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }
    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithNiche<niche::Unstable>, __IndirectLayout = layout::Stable<NonRobust>>,
    {
        // FIXME: It can also become Stable<Robust> WithoutNiche if R has exactly one trap
        // The same can happen even if R has a stable niche. The whole Option can be Robust
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }
    unsafe impl<R, K> RustSpec for Option<R>
    where
        R: RustSpec<
            Niche = WithNiche<niche::Stable>,
            __IndirectLayout = layout::Unstable<K>
        >,
        K: layout::TrapKind,
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }
    unsafe impl<R, K> RustSpec for Option<R>
    where
        R: RustSpec<
            Niche = WithNiche<niche::Stable>,
            __IndirectLayout = layout::Stable<K>,
        >,
        K: layout::TrapKind,
    {
        type Layout = layout::Stable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }

    unsafe impl<R, E, K: SizedKind> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<K>>,
        E: RustSpec<Size = size::Sized<K>>,
        <R as RustSpec>::__IndirectLayout: Add<<E as RustSpec>::__IndirectLayout>,
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = <R::__IndirectLayout as Add<E::__IndirectLayout>>::Output;

    }
    unsafe impl<
        R: RustSpec<Size = size::Sized<size::NonZst>, Niche = WithoutNiche>,
        E: RustSpec<Size = size::Sized<size::Zst>>,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }
    unsafe impl<
        R: RustSpec<Size = size::Sized<size::NonZst>, Niche = WithNiche<niche::Unstable>>,
        E: RustSpec<Size = size::Sized<size::Zst>>,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }

    unsafe impl<R, E, K> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<size::NonZst>,
            Niche = WithNiche<niche::Stable>,
            __IndirectLayout = layout::Unstable<K>,
            >,
        E: RustSpec<Size = size::Sized<size::Zst>>,
        K: layout::TrapKind,
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }
    unsafe impl<R, E, K> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<size::NonZst>,
            Niche = WithNiche<niche::Stable>,
            __IndirectLayout = layout::Stable<K>,
            >,
        E: RustSpec<Size = size::Sized<size::Zst>>,
        K: layout::TrapKind,
    {
        type Layout = layout::Stable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = R::__IndirectLayout;
    }

    unsafe impl<
        R: RustSpec<Size = size::Sized<size::Zst>>,
        E: RustSpec<Size = size::Sized<size::NonZst>, Niche = WithoutNiche>,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = E::__IndirectLayout;
    }
    unsafe impl<
        R: RustSpec<Size = size::Sized<size::Zst>>,
        E: RustSpec<Size = size::Sized<size::NonZst>, Niche = WithNiche<niche::Unstable>>,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = E::__IndirectLayout;
    }
    unsafe impl<R, E, K> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<size::Zst>>,
        E: RustSpec<
            Size = size::Sized<size::NonZst>,
            Niche = WithNiche<niche::Stable>,
            __IndirectLayout = layout::Unstable<K>,
            >,
        K: layout::TrapKind,
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = E::__IndirectLayout;
    }
    unsafe impl<R, E, K> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<size::Zst>>,
        E: RustSpec<
            Size = size::Sized<size::NonZst>,
            Niche = WithNiche<niche::Stable>,
            __IndirectLayout = layout::Stable<K>,
            >,
        K: layout::TrapKind,
    {
        type Layout = layout::Stable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectLayout = E::__IndirectLayout;
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    use static_assertions::assert_impl_all;

    use super::*;
    use {
        layout::{Stable, Unstable},
        mutability::Exclusive,
        niche::{WithNiche, WithoutNiche},
        size::{NonZst, Sized as Co3Sized},
    };

    #[test]
    fn indirect_layout_follows_supported_pointers_only() {
        assert_impl_all!(&u8: RustSpec<__IndirectLayout = Stable<Robust>>);
        assert_impl_all!(&bool: RustSpec<__IndirectLayout = Stable<NonRobust>>);
        assert_impl_all!(&&u8: RustSpec<__IndirectLayout = Stable<NonRobust>>);
        assert_impl_all!(Option<&u8>: RustSpec<__IndirectLayout = Stable<Robust>>);
        assert_impl_all!(*const bool: RustSpec<__IndirectLayout = Stable<Robust>>);
    }

    #[test]
    fn transparent_type() {
        assert_impl_all!(bool:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&bool:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut bool:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<bool>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&[bool]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [bool]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[bool]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<bool>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!([bool; 2]:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        // FIXME:
        //assert_impl_all!(Option<bool>:
        //    RustSpec<
        //        Layout = Unstable<NonRobust>,
        //        Size = Co3Sized<NonZst>,
        //        Niche = WithNiche<niche::Unstable>,
        //        Mutability = Exclusive,
        //    >,
        //);
    }

    #[test]
    fn option_stable_niche_layout() {
        use core::num::NonZeroU8;

        assert_impl_all!(Option<NonZeroU8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<&u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<&(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<Robust>,
            >,
        );
        assert_impl_all!(Option<&NonZeroU8>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
    }

    #[test]
    fn result_stable_niche_layout() {
        use core::num::NonZeroU8;

        assert_impl_all!(Result<NonZeroU8, ()>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Result<&u8, ()>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Result<&NonZeroU8, ()>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(Result<(), NonZeroU8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Result<&mut u8, ()>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
    }

    #[test]
    fn robust_ref() {
        assert_impl_all!(&&u8:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut &u8:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&[&u8]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut [&u8]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&u8]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&u8>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!([&u8; 2]:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<&u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
    }

    #[test]
    fn transparent_ref() {
        assert_impl_all!(&&bool:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut &bool:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&[&bool]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [&bool]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&bool]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&bool>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!([&bool; 2]:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(Option<&bool>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
    }

    #[test]
    fn robust_ref_mut() {
        assert_impl_all!(&&mut u8:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut &mut u8:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut u8>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&[&mut u8]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut [&mut u8]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut u8]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut u8>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!([&mut u8; 2]:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<&mut u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
    }

    #[test]
    fn transparent_ref_mut() {
        assert_impl_all!(&&mut bool:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut &mut bool:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut bool>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&[&mut bool]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut [&mut bool]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut bool]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut bool>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!([&mut bool; 2]:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(Option<&mut bool>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
    }
}
