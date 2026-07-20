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
//! Describes whether a type has a stable layout or trap values:
//! - [`layout::Stable<layout::Robust>`]: stable layout with no trap/invalid values.
//! - [`layout::Stable<layout::NonRobust>`]: stable layout, but with trap values.
//! - [`layout::Unstable<layout::Robust>`]: unstable layout with no trap values.
//! - [`layout::Unstable<layout::NonRobust>`]: unstable layout, but with trap values.
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

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use disjoint_impls::disjoint_impls;
#[cfg(feature = "derive")]
pub use rust_spec_derive::{RustSpec, Wide};

use crate::niche::{WithNiche, WithoutNiche};

pub mod layout;
pub mod mutability;
pub mod niche;
mod primitives;
pub mod size;
mod std_impls;
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
    }

    unsafe impl<R: RustSpec<Layout = layout::Unstable<K>, Size: size::Thin> + ?Sized, K> RustSpec
        for &R
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: RustSpec<Layout = layout::Unstable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        RustSpec for &R
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: RustSpec<Layout = layout::Stable<K>, Size: size::Thin> + ?Sized, K> RustSpec for &R {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: RustSpec<Layout = layout::Stable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        RustSpec for &R
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = R::Mutability;
    }

    unsafe impl<R: RustSpec<Layout = layout::Unstable<K>, Size: size::Thin> + ?Sized, K> RustSpec
        for &mut R
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: RustSpec<Layout = layout::Unstable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        RustSpec for &mut R
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: RustSpec<Layout = layout::Stable<K>, Size: size::Thin> + ?Sized, K> RustSpec
        for &mut R
    {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: RustSpec<Layout = layout::Stable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        RustSpec for &mut R
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = R::Mutability;
    }

    #[cfg(feature = "alloc")]
    unsafe impl<R: RustSpec<Layout = layout::Unstable<K>, Size: size::Thin> + ?Sized, K> RustSpec
        for Box<R>
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: RustSpec<Layout = layout::Unstable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        RustSpec for Box<R>
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: RustSpec<Layout = layout::Stable<K>, Size = size::Sized<S>>, S, K> RustSpec
        for Box<R>
    {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: RustSpec<Layout = layout::Stable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        RustSpec for Box<R>
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
    }

    unsafe impl<R: RustSpec<Niche = WithoutNiche>> RustSpec for Option<R> {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
    }
    unsafe impl<R: RustSpec<Niche = WithNiche<niche::Unstable>>> RustSpec for Option<R> {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: RustSpec<Layout = layout::Unstable<K>, Niche = WithNiche<niche::Stable>>, K> RustSpec
        for Option<R>
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = R::Mutability;
    }
    unsafe impl<
        R: RustSpec<Layout = layout::Stable<layout::NonRobust>, Niche = WithNiche<niche::Stable>>,
    > RustSpec for Option<R>
    {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: RustSpec<Size = size::Sized<K>>, E: RustSpec<Size = size::Sized<K>>, K> RustSpec
        for Result<R, E>
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = mutability::Exclusive;
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
    }
    unsafe impl<
        R: RustSpec<Size = size::Sized<size::NonZst>, Niche = WithNiche<niche::Unstable>>,
        E: RustSpec<Size = size::Sized<size::Zst>>,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<
        R: RustSpec<
                Layout = layout::Unstable<K>,
                Size = size::Sized<size::NonZst>,
                Niche = WithNiche<niche::Stable>,
            >,
        E: RustSpec<Size = size::Sized<size::Zst>>,
        K,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = R::Mutability;
    }
    unsafe impl<
        R: RustSpec<
                Layout = layout::Stable<layout::NonRobust>,
                Size = size::Sized<size::NonZst>,
                Niche = WithNiche<niche::Stable>,
            >,
        E: RustSpec<Size = size::Sized<size::Zst>>,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = R::Mutability;
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
    }
    unsafe impl<
        R: RustSpec<Size = size::Sized<size::Zst>>,
        E: RustSpec<Size = size::Sized<size::NonZst>, Niche = WithNiche<niche::Unstable>>,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Unstable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Unstable>;
        type Mutability = E::Mutability;
    }
    unsafe impl<
        R: RustSpec<Size = size::Sized<size::Zst>>,
        E: RustSpec<
                Layout = layout::Unstable<K>,
                Size = size::Sized<size::NonZst>,
                Niche = WithNiche<niche::Stable>,
            >,
        K,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = E::Mutability;
    }
    unsafe impl<
        R: RustSpec<Size = size::Sized<size::Zst>>,
        E: RustSpec<
                Layout = layout::Stable<layout::NonRobust>,
                Size = size::Sized<size::NonZst>,
                Niche = WithNiche<niche::Stable>,
            >,
    > RustSpec for Result<R, E>
    {
        type Layout = layout::Stable<layout::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = E::Mutability;
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    use static_assertions::assert_impl_all;

    use super::*;
    use crate::rust_spec::{
        layout::{NonRobust, Robust},
        layout::{Stable, Unstable},
        mutability::{Exclusive, Interior},
        niche::{WithNiche, WithoutNiche},
        size::{NonZst, Sized as Co3Sized},
    };

    #[test]
    fn type_spec_joins_family_axes() {
        assert_impl_all!(u8:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
            >,
        );

        assert_impl_all!(Option<&u8>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
            >,
        );
    }

    #[test]
    fn mutability_family_tracks_whole_value_mutability() {
        use core::{
            cell::{Cell, UnsafeCell},
            ptr::NonNull,
        };

        assert_impl_all!(u8: RustSpec<Mutability = Exclusive>);
        assert_impl_all!(UnsafeCell<u8>: RustSpec<Mutability = Interior>);
        assert_impl_all!(Cell<u8>: RustSpec<Mutability = Interior>);
        assert_impl_all!((UnsafeCell<u8>, UnsafeCell<u8>): RustSpec<Mutability = Interior>);
        assert_impl_all!((UnsafeCell<u8>, u8): RustSpec<Mutability = Exclusive>);
        assert_impl_all!([UnsafeCell<u8>; 2]: RustSpec<Mutability = Interior>);
        assert_impl_all!([u8; 2]: RustSpec<Mutability = Exclusive>);
        assert_impl_all!(&UnsafeCell<u8>: RustSpec<Mutability = Interior>);
        assert_impl_all!(&mut UnsafeCell<u8>: RustSpec<Mutability = Interior>);
        assert_impl_all!(*const UnsafeCell<u8>: RustSpec<Mutability = Exclusive>);
        assert_impl_all!(*mut UnsafeCell<u8>: RustSpec<Mutability = Exclusive>);
        assert_impl_all!(NonNull<UnsafeCell<u8>>: RustSpec<Mutability = Exclusive>);
        assert_impl_all!(Option<UnsafeCell<u8>>: RustSpec<Mutability = Exclusive>);

        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>: RustSpec<Mutability = Interior>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>: RustSpec<Mutability = Exclusive>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>: RustSpec<Mutability = Exclusive>);
    }

    #[cfg(feature = "derive")]
    #[test]
    fn type_spec_derive_tracks_whole_value_mutability() {
        use core::cell::UnsafeCell;

        #[allow(dead_code)]
        #[derive(RustSpec)]
        struct InteriorWrapper {
            left: UnsafeCell<u8>,
            right: UnsafeCell<u16>,
        }

        #[allow(dead_code)]
        #[derive(RustSpec)]
        struct MixedWrapper {
            cell: UnsafeCell<u8>,
            plain: u8,
        }

        #[allow(dead_code)]
        #[derive(RustSpec)]
        struct EmptyWrapper;

        assert_impl_all!(InteriorWrapper: RustSpec<Mutability = Interior>);
        assert_impl_all!(MixedWrapper: RustSpec<Mutability = Exclusive>);
        assert_impl_all!(EmptyWrapper: RustSpec<Mutability = Exclusive>);
    }

    #[cfg(feature = "derive")]
    #[test]
    fn type_spec_derive_tracks_unstable_robustness() {
        #[allow(dead_code)]
        #[derive(RustSpec)]
        struct RobustRustRepr {
            value: u8,
        }

        #[allow(dead_code)]
        #[derive(RustSpec)]
        struct NonRobustRustRepr {
            value: bool,
        }

        assert_impl_all!(RobustRustRepr: RustSpec<Layout = Unstable<Robust>>);
        assert_impl_all!(NonRobustRustRepr: RustSpec<Layout = Unstable<NonRobust>>);
    }

    #[test]
    fn transparent_type() {
        assert_impl_all!(bool:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!(&bool:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut bool:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<bool>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[bool]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [bool]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[bool]>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<bool>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!([bool; 2]:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        // FIXME:
        //assert_impl_all!(Option<bool>:
        //    RustSpec<Layout = Unstable<NonRobust>>,
        //    RustSpec<Size = Co3Sized<NonZst>>,
        //    RustSpec<Niche = WithNiche<niche::Unstable>>,
        //);
    }

    #[test]
    fn robust_ref() {
        assert_impl_all!(&&u8:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut &u8:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[&u8]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!(&mut [&u8]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&u8]>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&u8>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!([&u8; 2]:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!(Option<&u8>:
            // FIXME:
            //RustSpec<Layout = Stable<Robust>>,
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithoutNiche>,
        );
    }

    #[test]
    fn transparent_ref() {
        assert_impl_all!(&&bool:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut &bool:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[&bool]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [&bool]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&bool]>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&bool>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!([&bool; 2]:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!(Option<&bool>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithoutNiche>,
        );
    }

    #[test]
    fn robust_ref_mut() {
        assert_impl_all!(&&mut u8:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut &mut u8:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut u8>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[&mut u8]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!(&mut [&mut u8]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut u8]>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut u8>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!([&mut u8; 2]:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!(Option<&mut u8>:
            // FIXME:
            //RustSpec<Layout = Stable<Robust>>,
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithoutNiche>,
        );
    }

    #[test]
    fn transparent_ref_mut() {
        assert_impl_all!(&&mut bool:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut &mut bool:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut bool>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[&mut bool]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!(&mut [&mut bool]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut bool]>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut bool>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!([&mut bool; 2]:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<niche::Unstable>>,
        );
        assert_impl_all!(Option<&mut bool>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithoutNiche>,
        );
    }
}
