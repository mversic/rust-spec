//! Mutability classification of Rust types.

use core::ops::Add;

/// Closed family of shared-access mutability classifications.
#[sealed::sealed]
pub trait MutabilitySpec {}

/// Marker for types whose whole value may be mutated through shared access.
pub enum Interior {}

/// Marker for types whose value requires exclusive access to mutate.
pub enum Exclusive {}

#[sealed::sealed]
impl MutabilitySpec for Interior {}

#[sealed::sealed]
impl MutabilitySpec for Exclusive {}

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

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, vec::Vec};

    use static_assertions::assert_impl_all;

    use crate::{
        RustSpec,
        layout::{NonRobust, Robust},
        layout::{Stable, Unstable},
        mutability::{Exclusive, Interior},
        niche::{self, WithNiche, WithoutNiche},
        size::{NonZst, Sized as Co3Sized},
    };

    #[test]
    fn mutability_family_tracks_whole_value_mutability() {
        use core::{
            cell::{Cell, UnsafeCell},
            ptr::NonNull,
        };

        assert_impl_all!(u8:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(UnsafeCell<u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Cell<u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!((UnsafeCell<u8>, UnsafeCell<u8>):
            RustSpec<
                Layout = Unstable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!((UnsafeCell<u8>, u8):
            RustSpec<
                Layout = Unstable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!([UnsafeCell<u8>; 2]:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!([u8; 2]:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&UnsafeCell<u8>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&mut UnsafeCell<u8>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(*const UnsafeCell<u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(*mut UnsafeCell<u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(NonNull<UnsafeCell<u8>>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<UnsafeCell<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );

        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
    }
}
