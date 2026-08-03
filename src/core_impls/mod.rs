//! `RustSpec` implementations for types defined by `core`.

mod cells;
mod ffi;
mod wrappers;

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, string::String, vec::Vec};
    use core::{cell::UnsafeCell, mem::ManuallyDrop, num::NonZero as StdNonZero};

    use static_assertions::assert_impl_all;

    use crate::{
        RustSpec, Stable, Unstable,
        layout::{NonRobust, Robust},
        mutability::{Exclusive, Interior},
        niche::{WithNiche, WithoutNiche},
        size::{MetaSized, Sized as Co3Sized, SliceLike},
    };

    #[test]
    fn str_support() {
        assert_impl_all!(str:
            RustSpec<
                Layout = Stable,
                Size = MetaSized<SliceLike>,
                Alignment = crate::One,
                Trap = NonRobust,
                Mutability = Exclusive,
                __IndirectTrap = Robust,
            >,
        );

        assert_impl_all!(&str:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut str:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<str>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
    }

    #[test]
    fn manually_drop_inner_without_drop() {
        assert_impl_all!(ManuallyDrop<u8>:
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
        assert_impl_all!(&ManuallyDrop<u8>:
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
        assert_impl_all!(&mut ManuallyDrop<u8>:
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
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<ManuallyDrop<u8>>:
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
        assert_impl_all!(&[ManuallyDrop<u8>]:
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
        assert_impl_all!(&mut [ManuallyDrop<u8>]:
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
        assert_impl_all!(Box<[ManuallyDrop<u8>]>:
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
        assert_impl_all!(Vec<ManuallyDrop<u8>>:
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
        assert_impl_all!([ManuallyDrop<u8>; 2]:
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
        assert_impl_all!(Option<ManuallyDrop<u8>>:
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
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn manually_drop_inner_with_drop() {
        assert_impl_all!(ManuallyDrop<String>:
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
        assert_impl_all!(&ManuallyDrop<String>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut ManuallyDrop<String>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Box<ManuallyDrop<String>>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[ManuallyDrop<String>]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut [ManuallyDrop<String>]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Box<[ManuallyDrop<String>]>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(Vec<ManuallyDrop<String>>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([ManuallyDrop<String>; 2]:
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
        assert_impl_all!(Option<ManuallyDrop<String>>:
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

    #[test]
    fn robust_unsafe_cell() {
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
        assert_impl_all!(&[UnsafeCell<u8>]:
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
        assert_impl_all!(&mut [UnsafeCell<u8>]:
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
    }

    #[test]
    fn non_robust_unsafe_cell() {
        assert_impl_all!(UnsafeCell<StdNonZero<u8>>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(&UnsafeCell<StdNonZero<u8>>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Interior,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut UnsafeCell<StdNonZero<u8>>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Interior,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<StdNonZero<u8>>>:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Stable>,
                Mutability = Interior,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&[UnsafeCell<StdNonZero<u8>>]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!(&mut [UnsafeCell<StdNonZero<u8>>]:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<StdNonZero<u8>>]>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<StdNonZero<u8>>>:
            RustSpec<
                Layout = Unstable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = <usize as RustSpec>::Alignment,
                Trap = NonRobust,
                Niche = WithNiche<Unstable>,
                Mutability = Exclusive,
                __IndirectTrap = NonRobust,
            >,
        );
        assert_impl_all!([UnsafeCell<StdNonZero<u8>>; 2]:
            RustSpec<
                Layout = Stable,
                Size = Co3Sized<crate::Gt<crate::Zero>>,
                Alignment = crate::One,
                Trap = NonRobust,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectTrap = Robust,
            >,
        );
        assert_impl_all!(Option<UnsafeCell<StdNonZero<u8>>>:
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
    }
}
