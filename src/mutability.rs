//! Mutability classification of Rust types.

use core::ops::Add;

/// Marker for types whose value requires exclusive access to mutate.
pub enum Exclusive {}

/// Marker for types with at least one part mutable through shared access.
///
/// Any type that carries `Cell<T>` or `UnsafeCell<T>`.
pub enum WithInterior {}

macro_rules! combine_mutability {
    ($left:ty, $right:ty => $output:ty) => {
        impl Add<$right> for $left {
            type Output = $output;

            fn add(self, _: $right) -> Self::Output {
                unreachable!()
            }
        }
    };
}

combine_mutability!(Exclusive, Exclusive => Exclusive);
combine_mutability!(Exclusive, WithInterior => WithInterior);
combine_mutability!(WithInterior, Exclusive => WithInterior);
combine_mutability!(WithInterior, WithInterior => WithInterior);

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, vec::Vec};

    use static_assertions::assert_impl_all;

    use crate::{
        RustSpec, Stable, Unstable,
        layout::{NonRobust, Robust},
        mutability::{Exclusive, WithInterior},
        niche::{WithNiche, WithoutNiche},
        size::Sized as RustSpecSized,
    };

    #[test]
    fn mutability_family_tracks_shared_mutability() {
        use core::{
            cell::{Cell, UnsafeCell},
            ptr::NonNull,
        };

        assert_impl_all!(u8:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
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
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Cell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!((UnsafeCell<u8>, UnsafeCell<u8>):
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!((UnsafeCell<u8>, u8):
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([UnsafeCell<u8>; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = Robust,
                Niche = WithoutNiche,
                Mutability = WithInterior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!([u8; 2]:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
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
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&mut UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(*const UnsafeCell<u8>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
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
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
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
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
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
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = WithInterior,
                __IndirectTrap = Robust,
            >,
        );

        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>:
            RustSpec<
                Layout = Stable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>:
            RustSpec<
                Layout = Unstable,
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
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
                Size = RustSpecSized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );
    }
}
