//! `RustSpec` implementations for types defined by `core`.

mod cells;
mod ffi;
mod wrappers;

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::{boxed::Box, string::String, vec::Vec};
    use core::{cell::UnsafeCell, mem::ManuallyDrop, num::NonZero};

    use static_assertions::assert_impl_all;

    use crate::{
        RustSpec,
        layout::{NonRobust, Robust, Stable, Unstable},
        mutability::{Exclusive, Interior},
        niche::{self, WithNiche, WithoutNiche},
        size::{MetaSized, NonZst, Sized as Co3Sized, SliceLike},
    };

    #[test]
    fn str_support() {
        assert_impl_all!(str:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = MetaSized<SliceLike>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );

        assert_impl_all!(&str:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut str:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<str>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
    }

    #[test]
    fn manually_drop_inner_without_drop() {
        assert_impl_all!(ManuallyDrop<u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&ManuallyDrop<u8>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&mut ManuallyDrop<u8>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<ManuallyDrop<u8>>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&[ManuallyDrop<u8>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&mut [ManuallyDrop<u8>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[ManuallyDrop<u8>]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<ManuallyDrop<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!([ManuallyDrop<u8>; 2]:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<ManuallyDrop<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn manually_drop_inner_with_drop() {
        assert_impl_all!(ManuallyDrop<String>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&ManuallyDrop<String>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut ManuallyDrop<String>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(Box<ManuallyDrop<String>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(&[ManuallyDrop<String>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut [ManuallyDrop<String>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(Box<[ManuallyDrop<String>]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(Vec<ManuallyDrop<String>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!([ManuallyDrop<String>; 2]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<ManuallyDrop<String>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
    }

    #[test]
    fn robust_unsafe_cell() {
        assert_impl_all!(UnsafeCell<u8>:
            RustSpec<
                Layout = Unstable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&UnsafeCell<u8>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Unstable<Robust>,
            >,
        );
        assert_impl_all!(&mut UnsafeCell<u8>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Unstable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Unstable<Robust>,
            >,
        );
        assert_impl_all!(&[UnsafeCell<u8>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<Robust>,
            >,
        );
        assert_impl_all!(&mut [UnsafeCell<u8>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<Robust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<Robust>,
            >,
        );
        assert_impl_all!([UnsafeCell<u8>; 2]:
            RustSpec<
                Layout = Unstable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
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
    }

    #[test]
    fn non_robust_unsafe_cell() {
        assert_impl_all!(UnsafeCell<NonZero<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&UnsafeCell<NonZero<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut UnsafeCell<NonZero<u8>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<NonZero<u8>>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(&[UnsafeCell<NonZero<u8>>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut [UnsafeCell<NonZero<u8>>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<NonZero<u8>>]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<NonZero<u8>>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Unstable<NonRobust>,
            >,
        );
        assert_impl_all!([UnsafeCell<NonZero<u8>>; 2]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(Option<UnsafeCell<NonZero<u8>>>:
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
