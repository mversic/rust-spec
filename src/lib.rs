//! **Compile-time classification of types** according to the Rust specification.
//!
//! # Classification Axes
//!
//! [`RustSpec`] describes a type across the following axes:
//! - [`layout`]: representation stability.
//! - [`size`]: statically sized, metadata-sized, or extern-type-like shape.
//! - `alignment`: whether alignment is one or greater than one.
//! - [`trap`](layout): whether the value representation has trap values.
//! - [`niche`]: whether a type has a stable, unstable, or no niche value.
//! - [`mutability`]: whether type can be mutated through shared access.
//! - [`drop`](mod@drop): whether dropping runs custom or compiler-generated code.
//! - [`Opaque`]: whether the value representation may be inspected.
//!
//! ## Layout
//!
//! Describes total reachable (through pointer indirection) representation stability:
//! - [`Stable`]: compiler-guaranteed representation.
//! - [`Unstable`]: representation is not guaranteed.
//!
//! ## Size
//!
//! Describes compile-time size and pointer metadata shape:
//! - [`Sized`](size::Sized)<[`Zero`]>: compile-time known zero-sized type.
//! - [`Sized`](size::Sized)<[`Gt`]<[`Zero`]>>: compile-time known non-zero-sized type.
//! - [`MetaSized`](size::MetaSized)<[`SliceLike`](size::SliceLike)>: dynamically sized slice-like type.
//! - [`MetaSized`](size::MetaSized)<[`DynTraitLike`](size::DynTraitLike)>: dynamically sized trait-object-like type.
//! - [`ExternTypeLike`](size::ExternTypeLike): dynamically sized extern-type-like type.
//! - [`NulTerminated`](size::NulTerminated): unsized nul-terminated data.
//!
//! ## Alignment
//!
//! - [`One`] means alignment is exactly one;
//! - [`Gt`]<[`One`]> means it is greater than one.
//!
//! ## Trap
//!
//! Describes total reachable (through pointer indirection) value-representation validity:
//! - [`Robust`](layout::Robust): every bit pattern (through indirection) is valid.
//! - [`NonRobust`](layout::NonRobust): some bit patterns are trap/invalid values.
//!
//! This classification does not establish provenance, alignment, lifetimes, or aliasing requirements
//! for pointers or references. For example, `Option<&u8>` is `Robust`, but a non-null representation
//! must still be a valid reference before constructing the value.
//!
//! ## Niche
//!
//! Describes whether and what kind of niche is available for the type:
//! - [`WithoutNiche`]: no niche is available.
//! - [`WithNiche`]<[`Stable`]>: compiler-guaranteed niche.
//! - [`WithNiche`]<[`Unstable`]>: niche exists but is not guaranteed.
//!
//! **Use `#[rust_spec(custom_niche)]` when deriving [`RustSpec`] to select `WithNiche<Unstable>`**.
//!
//! ## Drop
//!
//! Describes the type's behavior when dropped:
//! - [`NoDrop`]: dropping the type runs no drop glue.
//! - [`AutoDrop`](drop::AutoDrop): owned contents need automatic drop glue.
//! - [`CustomDrop`]<[`NoDrop`]>: the type implements [`Drop`] and its contents need no automatic drop glue.
//! - [`CustomDrop`]<[`AutoDrop`](drop::AutoDrop)>: the type implements [`Drop`] and its contents need automatic drop glue.
//!
//! **Use `#[rust_spec(custom_drop)]` when deriving [`RustSpec`] for a type that implements `Drop`**.
//!
//! ## Mutability
//!
//! Describes whether a type has parts that can be mutated through shared access:
//! - [`Exclusive`](mutability::Exclusive): mutation requires exclusive non-interleaving access.
//! - [`WithInterior`](mutability::WithInterior): a part can be mutated through shared access.
//!
//! ## Opacity
//!
//! - [`NonOpaque`]: the value representation is not opaque.
//! - [`Opaque`]: the value must appear only behind an indirection.
//!
//! **Use `#[rust_spec(opaque)]` when deriving [`RustSpec`] to classify type as `Opaque`**.
//!
//! ## How to Use
//!
//! Derive [`RustSpec`] for your types, then use its associated marker families as bounds when implementing other traits:
//!
//! ```rust
//! # #[cfg(feature = "derive")]
//! # {
//! use disjoint_impls::disjoint_impls;
//! use rust_spec::{RustSpec, Stable, Unstable, niche::{WithNiche, WithoutNiche}};
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
//! # }
//! ```
//!
//! Check out [CO3](https://github.com/mversic/co3) to see how this crate allows for building higher zero-cost abstractions.
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as rust_spec;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::ops::Add;

use crate::{
    drop::{CustomDrop, DropKind, NoDrop},
    layout::Robust,
    mutability::Exclusive,
    niche::{WithNiche, WithoutNiche},
    size::{MetadataKind, SizedKind},
};
use disjoint_impls::disjoint_impls;
#[cfg(feature = "derive")]
pub use rust_spec_derive::RustSpec;

mod core_impls;
pub mod drop;
pub mod layout;
pub mod mutability;
pub mod niche;

mod primitives;
pub mod size;

mod tuple;

/// Marker for a compiler-guaranteed representation or niche.
pub enum Stable {}

/// Marker for a representation or niche that is not compiler-guaranteed.
pub enum Unstable {}

/// The value is opaque and must be placed behind an indirection.
pub enum Opaque {}

/// The value representation is not opaque.
pub enum NonOpaque {}

/// Marker for a zero-sized type.
pub enum Zero {}

/// Marker for a statically sized type greater than the given threshold.
pub struct Gt<T>(core::marker::PhantomData<T>);

/// Marker for alignment one.
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
impl WithoutOrUnstableNiche for WithNiche<Unstable> {}

#[doc(hidden)]
pub trait __HrtbAxes<const FIELD: usize> {
    type Layout<'a>;
    type Size<'a>;
    type Alignment<'a>;
    type Trap<'a>;
    type Niche<'a>;
    type Drop<'a>;
    type Mutability<'a>;
    type Opacity<'a>;
    type __IndirectTrap<'a>;
}

disjoint_impls! {
    /// Joined Rust-spec classification of a type.
    ///
    /// # Safety hazards when deriving `RustSpec`
    ///
    /// Check [`rust_spec_derive::RustSpec`]
    ///
    /// # Safety
    ///
    /// Implementors must classify `Self` truthfully according to each axis's documentation.
    /// Consumers must check [`RustSpec::Opacity`] before using the other axes to inspect
    /// or construct a value representation; see [`Opaque`].
    pub unsafe trait RustSpec {
        /// Representation-stability classification; see [`layout`].
        type Layout;

        /// Statically known, metadata-sized, or extern-type-like size classification; see [`size`].
        type Size;

        /// Alignment classification; see the [crate root](crate) for [`One`] and [`Gt`].
        type Alignment;

        /// Trap-representation classification, assuming pointer and reference validity
        /// is established separately; see [`layout`].
        type Trap;

        /// Niche availability classification; see [`niche`].
        type Niche;

        /// Whether `Self` declares `Drop`, has inner drop glue, or has no drop glue; see [`drop`](mod@drop).
        type Drop;

        /// Shared-access mutability classification; see [`mutability`].
        type Mutability;

        /// Whether the value representation is opaque; see [`Opaque`].
        type Opacity;

        /// Trap classification reachable through one or more supported pointer indirections; see [`layout`].
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
        type Drop = drop::NoDrop;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
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
        type Drop = drop::NoDrop;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::Trap;
    }
    unsafe impl<R: ?Sized> RustSpec for &R
    where
        R: RustSpec<Size = size::NulTerminated>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = drop::NoDrop;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
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
        type Drop = drop::NoDrop;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
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
        type Drop = drop::NoDrop;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::Trap;
    }
    unsafe impl<R: ?Sized> RustSpec for &mut R
    where
        R: RustSpec<Size = size::NulTerminated>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = drop::NoDrop;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::Trap;
    }

    #[cfg(feature = "alloc")]
    unsafe impl<R, S: SizedKind> RustSpec for Box<R>
    where
        R: RustSpec<Size = size::Sized<S>, Drop: Sized>,
        NoDrop: Add<<R as RustSpec>::Drop, Output: DropKind + Sized>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Stable>;
        type Drop = CustomDrop<<NoDrop as Add<R::Drop>>::Output>;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::Trap;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: ?Sized, U: MetadataKind> RustSpec for Box<R>
    where
        R: RustSpec<Size = size::MetaSized<U>, Drop: Sized>,
        NoDrop: Add<<R as RustSpec>::Drop, Output: DropKind + Sized>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = CustomDrop<<NoDrop as Add<R::Drop>>::Output>;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::Trap;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: ?Sized> RustSpec for Box<R>
    where
        R: RustSpec<Size = size::NulTerminated, Drop: Sized>,
        NoDrop: Add<<R as RustSpec>::Drop, Output: DropKind + Sized>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = CustomDrop<<NoDrop as Add<R::Drop>>::Output>;
        type Mutability = mutability::Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::Trap;
    }

    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithoutNiche, Drop: Sized>,
        NoDrop: Add<<R as RustSpec>::Drop>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = R::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <NoDrop as Add<R::Drop>>::Output;
        type Mutability = R::Mutability;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithNiche<Unstable>, Drop: Sized>,
        NoDrop: Add<<R as RustSpec>::Drop>,
    {
        type Layout = Unstable;
        // FIXME: This can be Robust but we have to track
        // the number of available custom/unstable niches.
        // The same thing happens for all other enums
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = R::Alignment;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <NoDrop as Add<R::Drop>>::Output;
        type Mutability = R::Mutability;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R> RustSpec for Option<R>
    where
        R: RustSpec<Niche = WithNiche<Stable>, Drop: Sized>,
        NoDrop: Add<<R as RustSpec>::Drop>,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = R::Alignment;
        type Trap = R::__IndirectTrap;
        type Niche = WithoutNiche;
        type Drop = <NoDrop as Add<R::Drop>>::Output;
        type Mutability = R::Mutability;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::__IndirectTrap;
    }

    unsafe impl<R, E, K: SizedKind> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<K>,
            Mutability: Add<<E as RustSpec>::Mutability>,
            Drop: Add<<E as RustSpec>::Drop>,
            __IndirectTrap: Add<<E as RustSpec>::__IndirectTrap>,
        >,
        E: RustSpec< Size = size::Sized<K>, Mutability: Sized, Drop: Sized>,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = <R::__IndirectTrap as Add<E::__IndirectTrap>>::Output;
    }

    // TODO: The next 3 impls are the same
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<Gt<Zero>>,
            Niche = WithoutNiche,
            Drop: Add<<E as RustSpec>::Drop>,
            Mutability: Add<<E as RustSpec>::Mutability>,
        >,
        E: RustSpec< Size = size::Sized<Zero>, Mutability: Sized, Drop: Sized>,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
        <E as RustSpec>::Alignment: Sized,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<Gt<Zero>>,
            Niche = WithNiche<Unstable>,
            Drop: Add<<E as RustSpec>::Drop>,
            Mutability: Add<<E as RustSpec>::Mutability>,
        >,
        E: RustSpec< Size = size::Sized<Zero>, Mutability: Sized, Drop: Sized>,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
        <E as RustSpec>::Alignment: Sized,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<Gt<Zero>>,
            Niche = WithNiche<Stable>,
            Drop: Add<<E as RustSpec>::Drop>,
            Mutability: Add<<E as RustSpec>::Mutability>,
        >,
        E: RustSpec< Size = size::Sized<Zero>, Alignment = Gt<One>, Mutability: Sized, Drop: Sized>,
        <E as RustSpec>::Alignment: Sized,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<Gt<Zero>>,
            Niche = WithNiche<Stable>,
            Drop: Add<<E as RustSpec>::Drop>,
            Mutability: Add<<E as RustSpec>::Mutability>,
        >,
        E: RustSpec< Size = size::Sized<Zero>, Alignment = One, Mutability: Sized, Drop: Sized>,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
        <E as RustSpec>::Alignment: Sized,
    {
        type Layout = R::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = R::__IndirectTrap;
        type Niche = WithoutNiche;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = R::__IndirectTrap;
    }

    // TODO: The next 3 impls are the same
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<Zero>,
            Drop: Add<<E as RustSpec>::Drop>,
            Mutability: Add<<E as RustSpec>::Mutability>,
        >,
        E: RustSpec< Size = size::Sized<Gt<Zero>>, Niche = WithoutNiche, Mutability: Sized, Drop: Sized>,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
        <E as RustSpec>::Alignment: Sized,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = E::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<Zero>,
            Niche: WithoutOrUnstableNiche,
            Drop: Add<<E as RustSpec>::Drop>,
            Mutability: Add<<E as RustSpec>::Mutability>,
        >,
        E: RustSpec<
            Size = size::Sized<Gt<Zero>>,
            Niche = WithNiche<Unstable>,
            Mutability: Sized,
            Drop: Sized,
        >,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
        <E as RustSpec>::Alignment: Sized,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = E::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<Zero>,
            Alignment = Gt<One>,
            Drop: Add<<E as RustSpec>::Drop>,
            Mutability: Add<<E as RustSpec>::Mutability>,
        >,
        E: RustSpec<
            Size = size::Sized<Gt<Zero>>,
            Niche = WithNiche<Stable>,
            Mutability: Sized,
            Drop: Sized,
        >,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
        <E as RustSpec>::Alignment: Sized,
    {
        type Layout = Unstable;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = layout::NonRobust;
        type Niche = WithNiche<Unstable>;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = E::__IndirectTrap;
    }
    unsafe impl<R, E> RustSpec for Result<R, E>
    where
        R: RustSpec<
            Size = size::Sized<Zero>,
            Alignment = One,
            Drop: Add<<E as RustSpec>::Drop>,
            Mutability: Add<<E as RustSpec>::Mutability>,
        >,
        E: RustSpec<
            Size = size::Sized<Gt<Zero>>,
            Niche = WithNiche<Stable>,
            Mutability: Sized,
            Drop: Sized,
        >,
        <R as RustSpec>::Alignment: Max<<E as RustSpec>::Alignment>,
        <E as RustSpec>::Alignment: Sized,
    {
        type Layout = E::Layout;
        type Size = size::Sized<Gt<Zero>>;
        type Alignment = <R::Alignment as Max<E::Alignment>>::Output;
        type Trap = E::__IndirectTrap;
        type Niche = WithoutNiche;
        type Drop = <R::Drop as Add<E::Drop>>::Output;
        type Mutability = <R::Mutability as Add<E::Mutability>>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = E::__IndirectTrap;
    }

    unsafe impl<R: ?Sized> RustSpec for *const R
    where
        R: RustSpec<Size: size::Thin>,
    {
        type Layout = Stable;
        type Size = size::Sized<crate::Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = Robust;
        type Niche = WithoutNiche;
        type Drop = NoDrop;
        type Mutability = Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = Robust;
    }
    unsafe impl<R: ?Sized, K: MetadataKind> RustSpec for *const R
    where
        R: RustSpec<Size = size::MetaSized<K>>,
    {
        type Layout = Unstable;
        type Size = size::Sized<crate::Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = Robust;
        type Niche = WithoutNiche;
        type Drop = NoDrop;
        type Mutability = Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = Robust;
    }
    unsafe impl<R: ?Sized> RustSpec for *const R
    where
        R: RustSpec<Size = size::NulTerminated>,
    {
        type Layout = Unstable;
        type Size = size::Sized<crate::Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = Robust;
        type Niche = WithoutNiche;
        type Drop = NoDrop;
        type Mutability = Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = Robust;
    }

    unsafe impl<R: ?Sized> RustSpec for *mut R
    where
        R: RustSpec<Size: size::Thin>,
    {
        type Layout = Stable;
        type Size = size::Sized<crate::Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = Robust;
        type Niche = WithoutNiche;
        type Drop = NoDrop;
        type Mutability = Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = Robust;
    }
    unsafe impl<R: ?Sized, K: MetadataKind> RustSpec for *mut R
    where
        R: RustSpec<Size = size::MetaSized<K>>,
    {
        type Layout = Unstable;
        type Size = size::Sized<crate::Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = Robust;
        type Niche = WithoutNiche;
        type Drop = NoDrop;
        type Mutability = Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = Robust;
    }
    unsafe impl<R: ?Sized> RustSpec for *mut R
    where
        R: RustSpec<Size = size::NulTerminated>,
    {
        type Layout = Unstable;
        type Size = size::Sized<crate::Gt<Zero>>;
        type Alignment = <usize as RustSpec>::Alignment;
        type Trap = Robust;
        type Niche = WithoutNiche;
        type Drop = NoDrop;
        type Mutability = Exclusive;
        type Opacity = NonOpaque;
        type __IndirectTrap = Robust;
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
        size::{Gt, Sized as RustSpecSized},
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
        //        Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
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
                Size = RustSpecSized<Gt<Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
    }
}
