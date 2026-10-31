//! **Compile-time classification of types** according to the Rust specification.
//!
//! # Classification Axes
//!
//! [`RustSpec`] describes a type across the following axes:
//! - [`layout`]: representation stability.
//! - [`size`]: statically sized, metadata-sized, or extern-type-like shape.
//! - [`trap`](layout): whether the value has trap representations.
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
//! - [`drop::CustomDrop`]<[`NoDrop`]>: the type implements [`Drop`] and its contents need no automatic drop glue.
//! - [`drop::CustomDrop`]<[`AutoDrop`](drop::AutoDrop)>: the type implements [`Drop`] and its contents need automatic drop glue.
//!
//! **Use `#[rust_spec(custom_drop)]` when deriving [`RustSpec`] for a type that implements `Drop`**.
//!
//! ## Mutability
//!
//! Describes whether a type has parts that can be mutated through shared access:
//! - [`Exclusive<WithoutInterior>`](mutability::Exclusive): mutation requires exclusive non-interleaving access.
//! - [`WithInterior<Robust>`](mutability::WithInterior): only robust parts can be mutated through shared access.
//! - [`WithInterior<NonRobust>`](mutability::WithInterior): non-robust parts can be mutated through shared access.
//!
//! ## Opacity
//!
//! - [`NonOpaque`]: the value representation is not opaque.
//! - [`Opaque`]: the value must appear only behind an indirection.
//!
//! **Use `#[rust_spec(opaque)]` when deriving [`RustSpec`] to classify type as `Opaque`**.
//!
//! # How to Use
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
use core::{convert::Infallible, ops::Add, ptr::NonNull};

#[cfg(feature = "alloc")]
use crate::drop::{CustomDrop, DropKind};
use crate::{
    drop::{NeedsDrop, NoDrop},
    layout::NonRobust,
    mutability::PointerMutability,
    niche::{NullableRepresentation, WithNiche, WithoutNiche},
    size::PointerLayout,
};
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

type Sum<A, B> = <A as Add<B>>::Output;

/// The output of the type-level [`Max`] operation.
#[doc(hidden)]
pub type Maximum<A, B> = <A as Max<B>>::Output;

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
pub struct Gt<T>(core::marker::PhantomData<T>, Infallible);

/// Marker for alignment one.
#[doc(hidden)]
pub enum One {}

/// Type-level maximum used to combine alignment categories.
#[doc(hidden)]
pub trait Max<Rhs> {
    type Output;
}

impl<Rhs> Max<Rhs> for One {
    type Output = Rhs;
}
impl<Rhs> Max<Rhs> for Gt<One> {
    type Output = Gt<One>;
}

/// Joined Rust-spec classification of a type.
///
/// # Safety hazards when deriving `RustSpec`
///
/// Check [`rust_spec_derive::RustSpec`].
///
/// # Safety
///
/// Implementors must classify `Self` truthfully according to each axis's documentation.
/// [`Opaque`] types must be used only through indirection and consumers should not read them.
pub unsafe trait RustSpec {
    /// Representation-stability classification; see [`layout`].
    type Layout;

    /// Statically known, metadata-sized, or extern-type-like size classification; see [`size`].
    type Size;

    /// __Alignment classification; see the [crate root](crate) for [`One`] and [`Gt`].
    #[doc(hidden)]
    type __Alignment;

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

unsafe impl<R: ?Sized> RustSpec for NonNull<R>
where
    R: RustSpec<Size: PointerLayout, Mutability: PointerMutability>,
    <R::Size as PointerLayout>::Output: Add<R::Layout>,
{
    type Layout = Sum<<R::Size as PointerLayout>::Output, R::Layout>;
    type Size = size::Sized<Gt<Zero>>;
    type __Alignment = <usize as RustSpec>::__Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<<R::Size as PointerLayout>::Output>;
    type Drop = NoDrop;
    type Mutability = <R::Mutability as PointerMutability>::Output;
    type Opacity = NonOpaque;
    type __IndirectTrap = R::Trap;
}

unsafe impl<R> RustSpec for Option<R>
where
    R: RustSpec<Niche: NullableRepresentation, Drop: drop::DropKind>,
    <R::Niche as NullableRepresentation>::Layout: Add<R::Layout>,
    <R::Niche as NullableRepresentation>::Trap: Add<R::__IndirectTrap>,
{
    type Layout = Sum<<R::Niche as NullableRepresentation>::Layout, R::Layout>;
    type Size = size::Sized<Gt<Zero>>;
    type __Alignment = R::__Alignment;
    type Trap = Sum<<R::Niche as NullableRepresentation>::Trap, R::__IndirectTrap>;
    type Niche = <R::Niche as NullableRepresentation>::Niche;
    type Drop = NeedsDrop<R::Drop>;
    type Mutability = R::Mutability;
    type Opacity = NonOpaque;
    type __IndirectTrap = R::__IndirectTrap;
}

unsafe impl<R: ?Sized> RustSpec for &R
where
    R: RustSpec<Size: PointerLayout, Mutability: PointerMutability>,
    <R::Size as PointerLayout>::Output: Add<R::Layout>,
{
    type Layout = Sum<<R::Size as PointerLayout>::Output, R::Layout>;
    type Size = size::Sized<Gt<Zero>>;
    type __Alignment = <usize as RustSpec>::__Alignment;
    type Trap = layout::NonRobust;
    type Niche = WithNiche<<R::Size as PointerLayout>::Output>;
    type Drop = drop::NoDrop;
    type Mutability = <R::Mutability as PointerMutability>::Output;
    type Opacity = NonOpaque;
    type __IndirectTrap = R::Trap;
}

unsafe impl<R: ?Sized> RustSpec for &mut R
where
    R: RustSpec<Size: PointerLayout, Mutability: PointerMutability>,
    <R::Size as PointerLayout>::Output: Add<R::Layout>,
{
    type Layout = Sum<<R::Size as PointerLayout>::Output, R::Layout>;
    type Size = size::Sized<Gt<Zero>>;
    type __Alignment = <usize as RustSpec>::__Alignment;
    type Trap = layout::NonRobust;
    type Niche = WithNiche<<R::Size as PointerLayout>::Output>;
    type Drop = drop::NoDrop;
    type Mutability = <R::Mutability as PointerMutability>::Output;
    type Opacity = NonOpaque;
    type __IndirectTrap = R::Trap;
}

#[cfg(feature = "alloc")]
unsafe impl<R: ?Sized> RustSpec for Box<R>
where
    R: RustSpec<Size: PointerLayout, Drop: DropKind, Mutability: PointerMutability>,
    <R::Size as PointerLayout>::Output: Add<R::Layout>,
{
    type Layout = Sum<<R::Size as PointerLayout>::Output, R::Layout>;
    type Size = size::Sized<Gt<Zero>>;
    type __Alignment = <usize as RustSpec>::__Alignment;
    type Trap = layout::NonRobust;
    type Niche = WithNiche<<R::Size as PointerLayout>::Output>;
    type Drop = CustomDrop<NeedsDrop<R::Drop>>;
    type Mutability = <R::Mutability as PointerMutability>::Output;
    type Opacity = NonOpaque;
    type __IndirectTrap = R::Trap;
}

unsafe impl<R: ?Sized> RustSpec for *const R
where
    R: RustSpec<Size: PointerLayout, Mutability: PointerMutability>,
    <R::Size as PointerLayout>::Output: Add<R::Layout>,
{
    type Layout = Sum<<R::Size as PointerLayout>::Output, R::Layout>;
    type Size = size::Sized<crate::Gt<Zero>>;
    type __Alignment = <usize as RustSpec>::__Alignment;
    type Trap = R::Trap;
    type Niche = WithoutNiche;
    type Drop = NoDrop;
    type Mutability = <R::Mutability as PointerMutability>::Output;
    type Opacity = NonOpaque;
    type __IndirectTrap = R::Trap;
}

unsafe impl<R: ?Sized> RustSpec for *mut R
where
    R: RustSpec<Size: PointerLayout, Mutability: PointerMutability>,
    <R::Size as PointerLayout>::Output: Add<R::Layout>,
{
    type Layout = Sum<<R::Size as PointerLayout>::Output, R::Layout>;
    type Size = size::Sized<crate::Gt<Zero>>;
    type __Alignment = <usize as RustSpec>::__Alignment;
    type Trap = R::Trap;
    type Niche = WithoutNiche;
    type Drop = NoDrop;
    type Mutability = <R::Mutability as PointerMutability>::Output;
    type Opacity = NonOpaque;
    type __IndirectTrap = R::Trap;
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    use static_assertions::assert_impl_all;

    use super::*;
    use {
        layout::{NonRobust, Robust},
        mutability::{Exclusive, WithoutInterior},
        niche::{WithNiche, WithoutNiche},
        size::{Gt, Sized as RustSpecSized},
    };

    #[test]
    fn indirect_trap_follows_supported_pointers_only() {
        assert_impl_all!(&u8: RustSpec<__IndirectTrap = Robust>);
        assert_impl_all!(&bool: RustSpec<__IndirectTrap = NonRobust>);
        assert_impl_all!(&&u8: RustSpec<__IndirectTrap = NonRobust>);
        assert_impl_all!(Option<&u8>: RustSpec<__IndirectTrap = Robust>);
        assert_impl_all!(*const bool: RustSpec<__IndirectTrap = NonRobust>);
    }

    #[test]
    fn transparent_type() {
        assert_impl_all!(bool:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&bool:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut bool:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<bool>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[bool]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [bool]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[bool]>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<bool>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([bool; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
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
        //        Mutability = Exclusive<WithoutInterior>,
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
                __Alignment = One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&(u8, u8, u8)>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&NonZeroU8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
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
                __Alignment = One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Result<&u8, ()>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Result<&NonZeroU8, ()>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Result<(), NonZeroU8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Result<&mut u8, ()>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
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
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut &u8:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[&u8]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut [&u8]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&u8]>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&u8>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([&u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
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
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut &bool:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[&bool]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [&bool]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&bool]>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&bool>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([&bool; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Option<&bool>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
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
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut &mut u8:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[&mut u8]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut [&mut u8]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut u8]>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut u8>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([&mut u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<&mut u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
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
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut &mut bool:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut bool>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[&mut bool]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut [&mut bool]:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut bool]>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut bool>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([&mut bool; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Option<&mut bool>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<Gt<Zero>>,
                __Alignment = <usize as RustSpec>::__Alignment,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Exclusive<WithoutInterior>,
                __IndirectTrap = NonRobust,
            >,
        );
    }
}
