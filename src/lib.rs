#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as rust_spec;

#[cfg(feature = "derive")]
pub use rust_spec_derive::TypeSpec;

use crate::{mutability::MutabilityFamily, niche::NicheFamily, repr::ReprFamily, size::SizeFamily};

pub mod mutability;
pub mod niche;
mod primitives;
pub mod repr;
pub mod size;
mod std_impls;
mod tuple;

/// Joined Rust-spec classification of a type.
///
/// This is the canonical surface for Rust-specified properties used by
/// conversion. The older `*Family` traits are retained as compatibility
/// projections for code that still needs to name one axis independently.
///
/// # Safety
///
/// Implementors must classify `Self` truthfully. In particular, [`Self::Size`]
/// carries the same safety requirements as [`size::SizeFamily::Kind`].
pub unsafe trait TypeSpec {
    /// Representation stability and robustness classification.
    type Repr;

    /// Statically known, metadata-sized, or extern-type-like size classification.
    type Size;

    /// Niche availability classification.
    type Niche;

    /// Shared-access mutability classification.
    type Mutability;
}

unsafe impl<T: ReprFamily + SizeFamily + NicheFamily + MutabilityFamily + ?Sized> TypeSpec for T {
    type Repr = <T as ReprFamily>::Kind;
    type Size = <T as SizeFamily>::Kind;
    type Niche = <T as NicheFamily>::Kind;

    type Mutability = <T as MutabilityFamily>::Kind;
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    use static_assertions::assert_impl_all;

    use super::*;
    use crate::rust_spec::{
        mutability::{Exclusive, Interior},
        niche::{WithNiche, WithoutNiche},
        repr::{NonRobust, Robust, Stable, Unstable},
        size::{NonZst, Sized as Co3Sized},
    };

    #[test]
    fn type_spec_joins_family_axes() {
        assert_impl_all!(u8:
            TypeSpec<
                Repr = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
            >,
        );

        assert_impl_all!(Option<&u8>:
            TypeSpec<
                Repr = Stable<NonRobust>,
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

        assert_impl_all!(u8: MutabilityFamily<Kind = Exclusive>);
        assert_impl_all!(UnsafeCell<u8>: MutabilityFamily<Kind = Interior>);
        assert_impl_all!(Cell<u8>: MutabilityFamily<Kind = Interior>);
        assert_impl_all!((UnsafeCell<u8>, UnsafeCell<u8>): MutabilityFamily<Kind = Interior>);
        assert_impl_all!((UnsafeCell<u8>, u8): MutabilityFamily<Kind = Exclusive>);
        assert_impl_all!([UnsafeCell<u8>; 2]: MutabilityFamily<Kind = Interior>);
        assert_impl_all!([u8; 2]: MutabilityFamily<Kind = Exclusive>);
        assert_impl_all!(&UnsafeCell<u8>: MutabilityFamily<Kind = Interior>);
        assert_impl_all!(&mut UnsafeCell<u8>: MutabilityFamily<Kind = Interior>);
        assert_impl_all!(*const UnsafeCell<u8>: MutabilityFamily<Kind = Exclusive>);
        assert_impl_all!(*mut UnsafeCell<u8>: MutabilityFamily<Kind = Exclusive>);
        assert_impl_all!(NonNull<UnsafeCell<u8>>: MutabilityFamily<Kind = Exclusive>);
        assert_impl_all!(Option<UnsafeCell<u8>>: MutabilityFamily<Kind = Exclusive>);

        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>: MutabilityFamily<Kind = Interior>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>: MutabilityFamily<Kind = Exclusive>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>: MutabilityFamily<Kind = Exclusive>);
    }

    #[cfg(feature = "derive")]
    #[test]
    fn type_spec_derive_tracks_whole_value_mutability() {
        use core::cell::UnsafeCell;

        #[allow(dead_code)]
        #[derive(TypeSpec)]
        struct InteriorWrapper {
            left: UnsafeCell<u8>,
            right: UnsafeCell<u16>,
        }

        #[allow(dead_code)]
        #[derive(TypeSpec)]
        struct MixedWrapper {
            cell: UnsafeCell<u8>,
            plain: u8,
        }

        #[allow(dead_code)]
        #[derive(TypeSpec)]
        struct EmptyWrapper;

        assert_impl_all!(InteriorWrapper: MutabilityFamily<Kind = Interior>);
        assert_impl_all!(MixedWrapper: MutabilityFamily<Kind = Exclusive>);
        assert_impl_all!(EmptyWrapper: MutabilityFamily<Kind = Exclusive>);
    }

    #[cfg(feature = "derive")]
    #[test]
    fn type_spec_derive_tracks_unstable_robustness() {
        #[allow(dead_code)]
        #[derive(TypeSpec)]
        struct RobustRustRepr {
            value: u8,
        }

        #[allow(dead_code)]
        #[derive(TypeSpec)]
        struct NonRobustRustRepr {
            value: bool,
        }

        assert_impl_all!(RobustRustRepr: ReprFamily<Kind = Unstable<Robust>>);
        assert_impl_all!(NonRobustRustRepr: ReprFamily<Kind = Unstable<NonRobust>>);
    }

    #[test]
    fn transparent_type() {
        assert_impl_all!(bool:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&bool:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut bool:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<bool>:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[bool]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [bool]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[bool]>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<bool>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([bool; 2]:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        // FIXME:
        //assert_impl_all!(Option<bool>:
        //    ReprFamily<Kind = Unstable>,
        //    SizeFamily<Kind = Co3Sized<NonZst>>,
        //    NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        //);
    }

    #[test]
    fn robust_ref() {
        assert_impl_all!(&&u8:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut &u8:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[&u8]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&mut [&u8]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&u8]>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&u8>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([&u8; 2]:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(Option<&u8>:
            // FIXME:
            //ReprFamily<Kind = Stable<Robust>>,
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithoutNiche>,
        );
    }

    #[test]
    fn transparent_ref() {
        assert_impl_all!(&&bool:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut &bool:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[&bool]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [&bool]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&bool]>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&bool>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([&bool; 2]:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(Option<&bool>:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithoutNiche>,
        );
    }

    #[test]
    fn robust_ref_mut() {
        assert_impl_all!(&&mut u8:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut &mut u8:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut u8>:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[&mut u8]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&mut [&mut u8]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut u8]>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut u8>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([&mut u8; 2]:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(Option<&mut u8>:
            // FIXME:
            //ReprFamily<Kind = Stable<Robust>>,
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithoutNiche>,
        );
    }

    #[test]
    fn transparent_ref_mut() {
        assert_impl_all!(&&mut bool:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut &mut bool:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut bool>:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[&mut bool]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&mut [&mut bool]:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut bool]>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut bool>:
            ReprFamily<Kind = Unstable>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([&mut bool; 2]:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(Option<&mut bool>:
            ReprFamily<Kind = Stable<NonRobust>>,
            SizeFamily<Kind = Co3Sized<NonZst>>,
            NicheFamily<Kind = WithoutNiche>,
        );
    }
}
