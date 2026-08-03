//! Mutability classification of Rust types.

use core::ops::Add;

/// Marker for types whose whole value may be mutated through shared access.
pub enum Interior {}

/// Marker for types whose value requires exclusive access to mutate.
pub enum Exclusive {}

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
        RustSpec, Stable, Unstable,
        layout::{NonRobust, Robust},
        mutability::{Exclusive, Interior},
        niche::{WithNiche, WithoutNiche},
        size::Sized as Co3Sized,
    };

    #[test]
    fn mutability_family_tracks_whole_value_mutability() {
        use core::{
            cell::{Cell, UnsafeCell},
            ptr::NonNull,
        };

        assert_impl_all!(u8:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Cell<u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!((UnsafeCell<u8>, UnsafeCell<u8>):
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!((UnsafeCell<u8>, u8):
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([UnsafeCell<u8>; 2]:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(*const UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(*mut UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(NonNull<UnsafeCell<u8>>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<UnsafeCell<u8>>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );

        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }
}
