#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as rust_spec;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use disjoint_impls::disjoint_impls;
#[cfg(feature = "derive")]
pub use rust_spec_derive::TypeSpec;

use crate::niche::{WithNiche, WithoutNiche};

pub mod mutability;
pub mod niche;
mod primitives;
pub mod repr;
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

    unsafe impl<R: TypeSpec<Repr = repr::Unstable<K>, Size: size::Thin> + ?Sized, K> TypeSpec for &R {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Repr = repr::Unstable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        TypeSpec for &R
    {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Repr = repr::Stable<K>, Size: size::Thin> + ?Sized, K> TypeSpec for &R {
        type Repr = repr::Stable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Repr = repr::Stable<K>, Size = size::MetaSized<U>> + ?Sized, U, K> TypeSpec
        for &R
    {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = R::Mutability;
    }

    unsafe impl<R: TypeSpec<Repr = repr::Unstable<K>, Size: size::Thin> + ?Sized, K> TypeSpec
        for &mut R
    {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Repr = repr::Unstable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        TypeSpec for &mut R
    {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Repr = repr::Stable<K>, Size: size::Thin> + ?Sized, K> TypeSpec for &mut R {
        type Repr = repr::Stable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Repr = repr::Stable<K>, Size = size::MetaSized<U>> + ?Sized, U, K> TypeSpec
        for &mut R
    {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = R::Mutability;
    }

    #[cfg(feature = "alloc")]
    unsafe impl<R: TypeSpec<Repr = repr::Unstable<K>, Size: size::Thin> + ?Sized, K> TypeSpec
        for Box<R>
    {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: TypeSpec<Repr = repr::Unstable<K>, Size = size::MetaSized<U>> + ?Sized, U, K>
        TypeSpec for Box<R>
    {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = mutability::Exclusive;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: TypeSpec<Repr = repr::Stable<K>, Size = size::Sized<S>>, S, K> TypeSpec for Box<R> {
        type Repr = repr::Stable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Stable>;
        type Mutability = R::Mutability;
    }
    #[cfg(feature = "alloc")]
    unsafe impl<R: TypeSpec<Repr = repr::Stable<K>, Size = size::MetaSized<U>> + ?Sized, U, K> TypeSpec
        for Box<R>
    {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = mutability::Exclusive;
    }

    unsafe impl<R: TypeSpec<Niche = WithoutNiche>> TypeSpec for Option<R> {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = mutability::Exclusive;
    }
    unsafe impl<R: TypeSpec<Niche = WithNiche<niche::Custom>>> TypeSpec for Option<R> {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Repr = repr::Unstable<K>, Niche = WithNiche<niche::Stable>>, K> TypeSpec
        for Option<R>
    {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Repr = repr::Stable<repr::NonRobust>, Niche = WithNiche<niche::Stable>>>
        TypeSpec for Option<R>
    {
        type Repr = repr::Stable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = R::Mutability;
    }
    unsafe impl<R: TypeSpec<Size = size::Sized<K>>, E: TypeSpec<Size = size::Sized<K>>, K> TypeSpec
        for Result<R, E>
    {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = mutability::Exclusive;
    }
    unsafe impl<
        R: TypeSpec<Size = size::Sized<size::NonZst>, Niche = WithoutNiche>,
        E: TypeSpec<Size = size::Sized<size::Zst>>,
    > TypeSpec for Result<R, E>
    {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = mutability::Exclusive;
    }
    unsafe impl<
        R: TypeSpec<Size = size::Sized<size::NonZst>, Niche = WithNiche<niche::Custom>>,
        E: TypeSpec<Size = size::Sized<size::Zst>>,
    > TypeSpec for Result<R, E>
    {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = R::Mutability;
    }
    unsafe impl<
        R: TypeSpec<
                Repr = repr::Unstable<K>,
                Size = size::Sized<size::NonZst>,
                Niche = WithNiche<niche::Stable>,
            >,
        E: TypeSpec<Size = size::Sized<size::Zst>>,
        K,
    > TypeSpec for Result<R, E>
    {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = R::Mutability;
    }
    unsafe impl<
        R: TypeSpec<
                Repr = repr::Stable<repr::NonRobust>,
                Size = size::Sized<size::NonZst>,
                Niche = WithNiche<niche::Stable>,
            >,
        E: TypeSpec<Size = size::Sized<size::Zst>>,
    > TypeSpec for Result<R, E>
    {
        type Repr = repr::Stable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = R::Mutability;
    }

    unsafe impl<
        R: TypeSpec<Size = size::Sized<size::Zst>>,
        E: TypeSpec<Size = size::Sized<size::NonZst>, Niche = WithoutNiche>,
    > TypeSpec for Result<R, E>
    {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = mutability::Exclusive;
    }
    unsafe impl<
        R: TypeSpec<Size = size::Sized<size::Zst>>,
        E: TypeSpec<Size = size::Sized<size::NonZst>, Niche = WithNiche<niche::Custom>>,
    > TypeSpec for Result<R, E>
    {
        type Repr = repr::Unstable<repr::NonRobust>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithNiche<niche::Custom>;
        type Mutability = E::Mutability;
    }
    unsafe impl<
        R: TypeSpec<Size = size::Sized<size::Zst>>,
        E: TypeSpec<
                Repr = repr::Unstable<K>,
                Size = size::Sized<size::NonZst>,
                Niche = WithNiche<niche::Stable>,
            >,
        K,
    > TypeSpec for Result<R, E>
    {
        type Repr = repr::Unstable<K>;
        type Size = size::Sized<size::NonZst>;
        type Niche = WithoutNiche;
        type Mutability = E::Mutability;
    }
    unsafe impl<
        R: TypeSpec<Size = size::Sized<size::Zst>>,
        E: TypeSpec<
                Repr = repr::Stable<repr::NonRobust>,
                Size = size::Sized<size::NonZst>,
                Niche = WithNiche<niche::Stable>,
            >,
    > TypeSpec for Result<R, E>
    {
        type Repr = repr::Stable<repr::NonRobust>;
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

        assert_impl_all!(u8: TypeSpec<Mutability = Exclusive>);
        assert_impl_all!(UnsafeCell<u8>: TypeSpec<Mutability = Interior>);
        assert_impl_all!(Cell<u8>: TypeSpec<Mutability = Interior>);
        assert_impl_all!((UnsafeCell<u8>, UnsafeCell<u8>): TypeSpec<Mutability = Interior>);
        assert_impl_all!((UnsafeCell<u8>, u8): TypeSpec<Mutability = Exclusive>);
        assert_impl_all!([UnsafeCell<u8>; 2]: TypeSpec<Mutability = Interior>);
        assert_impl_all!([u8; 2]: TypeSpec<Mutability = Exclusive>);
        assert_impl_all!(&UnsafeCell<u8>: TypeSpec<Mutability = Interior>);
        assert_impl_all!(&mut UnsafeCell<u8>: TypeSpec<Mutability = Interior>);
        assert_impl_all!(*const UnsafeCell<u8>: TypeSpec<Mutability = Exclusive>);
        assert_impl_all!(*mut UnsafeCell<u8>: TypeSpec<Mutability = Exclusive>);
        assert_impl_all!(NonNull<UnsafeCell<u8>>: TypeSpec<Mutability = Exclusive>);
        assert_impl_all!(Option<UnsafeCell<u8>>: TypeSpec<Mutability = Exclusive>);

        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>: TypeSpec<Mutability = Interior>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>: TypeSpec<Mutability = Exclusive>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>: TypeSpec<Mutability = Exclusive>);
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

        assert_impl_all!(InteriorWrapper: TypeSpec<Mutability = Interior>);
        assert_impl_all!(MixedWrapper: TypeSpec<Mutability = Exclusive>);
        assert_impl_all!(EmptyWrapper: TypeSpec<Mutability = Exclusive>);
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

        assert_impl_all!(RobustRustRepr: TypeSpec<Repr = Unstable<Robust>>);
        assert_impl_all!(NonRobustRustRepr: TypeSpec<Repr = Unstable<NonRobust>>);
    }

    #[test]
    fn transparent_type() {
        assert_impl_all!(bool:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!(&bool:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut bool:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<bool>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[bool]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [bool]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[bool]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<bool>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!([bool; 2]:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        // FIXME:
        //assert_impl_all!(Option<bool>:
        //    TypeSpec<Repr = Unstable<NonRobust>>,
        //    TypeSpec<Size = Co3Sized<NonZst>>,
        //    TypeSpec<Niche = WithNiche<niche::Custom>>,
        //);
    }

    #[test]
    fn robust_ref() {
        assert_impl_all!(&&u8:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut &u8:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[&u8]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!(&mut [&u8]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&u8]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&u8>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!([&u8; 2]:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!(Option<&u8>:
            // FIXME:
            //TypeSpec<Repr = Stable<Robust>>,
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
    }

    #[test]
    fn transparent_ref() {
        assert_impl_all!(&&bool:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut &bool:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&bool>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[&bool]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut [&bool]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&bool]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&bool>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!([&bool; 2]:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!(Option<&bool>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
    }

    #[test]
    fn robust_ref_mut() {
        assert_impl_all!(&&mut u8:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut &mut u8:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut u8>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[&mut u8]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!(&mut [&mut u8]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut u8]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut u8>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!([&mut u8; 2]:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!(Option<&mut u8>:
            // FIXME:
            //TypeSpec<Repr = Stable<Robust>>,
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
    }

    #[test]
    fn transparent_ref_mut() {
        assert_impl_all!(&&mut bool:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&mut &mut bool:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<&mut bool>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Stable>>,
        );
        assert_impl_all!(&[&mut bool]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!(&mut [&mut bool]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[&mut bool]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<&mut bool>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!([&mut bool; 2]:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<niche::Custom>>,
        );
        assert_impl_all!(Option<&mut bool>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
    }
}
