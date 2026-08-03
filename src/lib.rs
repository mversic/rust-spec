//! Compile-time classification of Rust types according to Rust-specified type properties.
//!
//! # Classification Axes
//!
//! `RustSpec` describes a type across the following axes:
//! - [`layout`]: representation stability.
//! - [`size`]: statically sized, metadata-sized, or extern-type-like shape.
//! - `Alignment`: whether ABI alignment is one or greater than one.
//! - [`trap`](layout): whether the value representation has trap values.
//! - [`niche`]: whether a type has a stable, unstable, or no niche value.
//!
//! ## Layout
//!
//! Describes total reachable (through pointer indirection) representation stability:
//! - `Stable`: compiler-guaranteed representation.
//! - `Unstable`: representation is not guaranteed.
//!
//! ## Size
//!
//! Describes compile-time size and pointer metadata shape:
//! - [`size::Sized<Zero>`]: compile-time known zero-sized type.
//! - [`size::Sized<Gt<Zero>>`]: compile-time known non-zero-sized type.
//! - [`size::MetaSized<size::SliceLike>`]: dynamically sized slice-like type.
//! - [`size::MetaSized<size::DynTraitLike>`]: dynamically sized trait-object-like type.
//! - [`size::ExternTypeLike`]: dynamically sized extern-type-like type.
//!
//! ## Alignment
//!
//! Whether ABI alignment is exactly one or greater than one.
//!
//! ## Trap
//!
//! Describes total reachable (through pointer indirection) value-representation validity:
//! - [`layout::Robust`]: every bit pattern is valid.
//! - [`layout::NonRobust`]: some bit patterns are trap/invalid values.
//!
//! ## Niche
//!
//! Describes whether and what kind of niche is available for the type:
//! - [`niche::WithoutNiche`]: no niche is available.
//! - [`niche::WithNiche<Stable>`]: compiler-guaranteed niche.
//! - [`niche::WithNiche<Unstable>`]: niche exists but is not guaranteed.
//!
//! ## How to Use
//!
//! Derive `RustSpec` for your types, then use its associated marker families as bounds when implementing other traits:
//!
//! ```rust
//! use disjoint_impls::disjoint_impls;
//! use rust_spec::{
//!     RustSpec, Stable, Unstable,
//!     niche::{WithNiche, WithoutNiche}
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
//!     impl<T: RustSpec<Niche = WithNiche<Stable>>> NullableEncoding for T {
//!         const NEEDS_TAG: bool = false;
//!     }
//!
//!     impl<T: RustSpec<Niche = WithNiche<Unstable>>> NullableEncoding for T {
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
use core::ops::Add;

use crate::{
    niche::{WithNiche, WithoutNiche},
    size::{MetadataKind, SizedKind},
};
use disjoint_impls::disjoint_impls;
#[cfg(feature = "derive")]
pub use rust_spec_derive::RustSpec;

mod core_impls;

/// Marker for a compiler-guaranteed representation or niche.
pub enum Stable {}

/// Marker for a representation or niche that is not compiler-guaranteed.
pub enum Unstable {}

/// Marker for a zero-sized type.
pub enum Zero {}

/// Marker for a statically sized type greater than the given threshold.
pub struct Gt<T>(core::marker::PhantomData<T>);

/// Marker for ABI alignment one.
pub enum One {}

/// Type-level maximum used to combine alignment categories.
pub trait Max<Rhs> {
    type Output;
}

impl<Rhs> Max<Rhs> for One {
    type Output = Rhs;
}
impl<Rhs> Max<Rhs> for Gt<One> {
    type Output = Gt<One>;
}

trait WithoutOrUnstableNiche {}

impl WithoutOrUnstableNiche for WithoutNiche {}
impl WithoutOrUnstableNiche for WithNiche<Stable> {}

pub mod layout;
#[doc(hidden)]
pub mod mutability;
pub mod niche;
mod primitives;
pub mod size;
mod tuple;

/// Opaque, lifetime-indexed field axes emitted by `RustSpec` derives for
/// projections whose prerequisites are supplied by higher-ranked bounds.
#[doc(hidden)]
pub trait __HrtbAxes<const FIELD: usize> {
    type Layout<'a>;
    type Size<'a>;
    type Alignment<'a>;
    type Trap<'a>;
    type Niche<'a>;
    type Mutability<'a>;
    type __IndirectTrap<'a>;
}

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
        /// Representation-stability classification.
        type Layout;

        /// Statically known, metadata-sized, or extern-type-like size classification.
        type Size;

        /// ABI alignment
        type Alignment;

        /// Trap-representation classification.
        type Trap;

        /// Niche availability classification.
        type Niche;

        /// Shared-access mutability classification.
        #[doc(hidden)]
        type Mutability;

        /// Trap classification reachable through one or more supported pointer indirections.
        #[doc(hidden)]
        type __IndirectTrap;
    }

    unsafe impl<R: ?Sized> RustSpec for &R
    where
        R: RustSpec<Size: size::Thin>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Stable>;
        type Mutability = R::Mutability;
        type __IndirectTrap = R::Trap;
    }
    unsafe impl<R: ?Sized, U: MetadataKind> RustSpec for &R
    where
        R: RustSpec<Size = size::MetaSized<U>>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = R::Mutability;
        type __IndirectTrap = R::Trap;
    }

    unsafe impl<R: ?Sized> RustSpec for &mut R
    where
        R: RustSpec<Size: size::Thin>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Stable>;
        type Mutability = R::Mutability;
        type __IndirectTrap = R::Trap;
    }
    unsafe impl<R: ?Sized, U: MetadataKind> RustSpec for &mut R
    where
        R: RustSpec<Size = size::MetaSized<U>>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = R::Mutability;
        type __IndirectTrap = R::Trap;
    }

    #[cfg(feature = "alloc")]
    unsafe impl<R, S: SizedKind> RustSpec for Box<R>
    where
        R: RustSpec<Size = size::Sized<S>>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Stable>;
        type Mutability = R::Mutability;
        type __IndirectTrap = R::Trap;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: ?Sized, U: MetadataKind> RustSpec for Box<R>
    where
        R: RustSpec<Size = size::MetaSized<U>>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = R::Trap;
    }

    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithoutNiche>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = R::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithNiche<Unstable>>,
    {
        type Layout = Unstable;
        // FIXME: This can be Robust but we have to track
        // the number of available custom/unstable niches.
        // The same thing happens for all other enums
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = R::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithNiche<Stable>>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = R::Alignment;
        type Trap = R::__IndirectTrap;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = R::__IndirectTrap;
    }

    unsafe impl<R, E, K: SizedKind> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<K>>,
        E: RustSpec<Size = size::Sized<K>>,
        <R as RustSpec>::Niche: WithoutOrUnstableNiche,
        <E as RustSpec>::Niche: WithoutOrUnstableNiche,
        <R as RustSpec>::__IndirectTrap: Add<<E as RustSpec>::__IndirectTrap>,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = <R::__IndirectTrap as Add<E::__IndirectTrap>>::Output;
    }

    // TODO: The next 3 impls are the same
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<Gt<Zero>>, Niche = WithoutNiche>,
        E: RustSpec<Size = size::Sized<Zero>>,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<Gt<Zero>>, Niche = WithNiche<Unstable>>,
        E: RustSpec<Size = size::Sized<Zero>>,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<Gt<Zero>>, Niche = WithNiche<Stable>>,
        E: RustSpec<Size = size::Sized<Zero>, Alignment = Gt<One>>,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<Gt<Zero>>, Niche = WithNiche<Stable>>,
        E: RustSpec<Size = size::Sized<Zero>, Alignment = One>,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = R::__IndirectTrap;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = R::__IndirectTrap;
    }

    // TODO: The next 3 impls are the same
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<Zero>>,
        E: RustSpec<Size = size::Sized<Gt<Zero>>, Niche = WithoutNiche>,
        <R as RustSpec>::Alignment: Sized,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = E::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<Zero>, Niche: WithoutOrUnstableNiche>,
        E: RustSpec<Size = size::Sized<Gt<Zero>>, Niche = WithNiche<Unstable>>,
        <R as RustSpec>::Alignment: Sized,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = E::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<Zero>, Alignment = Gt<One>>,
        E: RustSpec<Size = size::Sized<Gt<Zero>>, Niche = WithNiche<Stable>>,
        <R as RustSpec>::Alignment: Sized,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = E::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<Size = size::Sized<Zero>, Alignment = One>,
        E: RustSpec<Size = size::Sized<Gt<Zero>>, Niche = WithNiche<Stable>>,
        <R as RustSpec>::Alignment: Sized,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = E::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = E::__IndirectTrap;
        type Niche = WithoutNiche;
        type Mutability = mutability::Exclusive;
        type __IndirectTrap = E::__IndirectTrap;
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    use static_assertions::assert_impl_all;

    use super::*;
    use {
        layout::{NonRobust, Robust},
        mutability::Exclusive,
        niche::{WithNiche, WithoutNiche},
        size::{Gt, Sized as Co3Sized},
    };

    #[test]
    fn indirect_trap_follows_supported_pointers_only() {
        assert_impl_all!(&u8: RustSpec<__IndirectTrap = Robust>);
        assert_impl_all!(&bool: RustSpec<__IndirectTrap = NonRobust>);
        assert_impl_all!(&&u8: RustSpec<__IndirectTrap = NonRobust>);
        assert_impl_all!(Option<&u8>: RustSpec<__IndirectTrap = Robust>);
        assert_impl_all!(*const bool: RustSpec<__IndirectTrap = Robust>);
    }

    #[test]
    fn transparent_type() {
        assert_impl_all!(bool:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&bool:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut bool:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<bool>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[bool]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [bool]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[bool]>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<bool>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([bool; 2]:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        // FIXME:
        //assert_impl_all!(Option<bool>:
        //    RustSpec<
        //        Layout = Unstable,
        //        Size = Co3Sized<Gt<Zero>>,
        //        Trap = NonRobust,
        //        Niche = WithNiche<Unstable>,
        //        Mutability = Exclusive,
        //    >,
        //);
    }

    #[test]
    fn option_stable_niche_layout() {
        use core::num::NonZeroU8;

        assert_impl_all!(Option<NonZeroU8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&NonZeroU8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
    }

    #[test]
    fn result_stable_niche_layout() {
        use core::num::NonZeroU8;

        assert_impl_all!(Result<NonZeroU8, ()>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Result<&u8, ()>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Result<&NonZeroU8, ()>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Result<(), NonZeroU8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Result<&mut u8, ()>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }

    #[test]
    fn robust_ref() {
        assert_impl_all!(&&u8:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut &u8:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[&u8]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut [&u8]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&u8]>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&u8>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([&u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }

    #[test]
    fn transparent_ref() {
        assert_impl_all!(&&bool:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut &bool:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[&bool]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [&bool]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&bool]>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&bool>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([&bool; 2]:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Option<&bool>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
    }

    #[test]
    fn robust_ref_mut() {
        assert_impl_all!(&&mut u8:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut &mut u8:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[&mut u8]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut [&mut u8]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut u8]>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut u8>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([&mut u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&mut u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }

    #[test]
    fn transparent_ref_mut() {
        assert_impl_all!(&&mut bool:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut &mut bool:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut bool>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[&mut bool]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut [&mut bool]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut bool]>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut bool>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([&mut bool; 2]:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Option<&mut bool>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
    }
}
