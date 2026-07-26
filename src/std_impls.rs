#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::{
    cell::{Cell, UnsafeCell},
    ffi::c_void,
    marker::PhantomData,
    mem::ManuallyDrop,
    num::NonZero,
    ptr::NonNull,
};

#[cfg(feature = "alloc")]
use crate::layout::Unstable;
use crate::{
    RustSpec,
    layout::{NonRobust, Robust, Stable},
    mutability::{Exclusive, Interior},
    niche::{self, WithNiche, WithoutNiche},
    size::{self, MetaSized, SliceLike},
};

macro_rules! non_zero_derive {
    ($($primitive:ty),+ $(,)?) => {$(
        unsafe impl RustSpec for NonZero<$primitive> {
            type Layout = Stable<NonRobust>;            type Size = size::Sized<size::NonZst>;
            type Niche = WithNiche<niche::Stable>;
            type Mutability = Exclusive;
            type __IndirectLayout = Stable<Robust>;

        })+
    }
}

non_zero_derive! {
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128,
}

unsafe impl RustSpec for c_void {
    type Layout = Stable<Robust>;
    // NOTE: size_of::<c_void> == 1
    type Size = size::Sized<size::NonZst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

unsafe impl RustSpec for () {
    type Layout = Stable<Robust>;
    type Size = size::Sized<size::Zst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

unsafe impl<T: ?Sized> RustSpec for PhantomData<T> {
    type Layout = Stable<Robust>;
    type Size = size::Sized<size::Zst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

unsafe impl<T: ?Sized> RustSpec for NonNull<T>
where
    T: RustSpec,
{
    type Layout = Stable<NonRobust>;
    type Size = size::Sized<size::NonZst>;
    type Niche = WithNiche<niche::Stable>;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

unsafe impl RustSpec for str {
    type Layout = Stable<NonRobust>;
    type Size = MetaSized<SliceLike>;
    type Niche = WithNiche<niche::Stable>;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

#[cfg(feature = "alloc")]
unsafe impl RustSpec for String {
    type Layout = Unstable<NonRobust>;
    type Size = size::Sized<size::NonZst>;
    type Niche = WithNiche<niche::Unstable>;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

#[cfg(feature = "alloc")]
unsafe impl<T> RustSpec for Vec<T> {
    type Layout = Unstable<NonRobust>;
    type Size = size::Sized<size::NonZst>;
    type Niche = WithNiche<niche::Unstable>;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

unsafe impl<T: ?Sized> RustSpec for UnsafeCell<T>
where
    T: RustSpec,
{
    type Layout = T::Layout;
    type Size = T::Size;
    type Niche = WithoutNiche;
    type Mutability = Interior;
    type __IndirectLayout = T::__IndirectLayout;
}

unsafe impl<T: ?Sized> RustSpec for Cell<T>
where
    T: RustSpec,
{
    type Layout = T::Layout;
    type Size = T::Size;
    type Niche = WithoutNiche;
    type Mutability = Interior;
    type __IndirectLayout = T::__IndirectLayout;
}

unsafe impl<T: ?Sized> RustSpec for ManuallyDrop<T>
where
    T: RustSpec,
{
    type Layout = T::Layout;
    type Size = T::Size;
    type Niche = T::Niche;
    type Mutability = T::Mutability;
    type __IndirectLayout = T::__IndirectLayout;
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::boxed::Box;
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    use super::*;
    use crate::layout::Unstable;
    use crate::{
        niche::WithNiche,
        size::{NonZst, Sized as Co3Sized},
    };

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
                __IndirectLayout = Stable<Robust>,
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
    fn str_is_supported() {
        assert_impl_all!(str:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = MetaSized<SliceLike>,
                Niche = WithNiche<niche::Stable>,
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
        #[cfg(feature = "alloc")]
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
    fn str_decode_rejects_invalid_utf8() {
        assert_impl_all!(str:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = MetaSized<SliceLike>,
                Niche = WithNiche<niche::Stable>,
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
    fn robust_unsafe_cell() {
        assert_impl_all!(UnsafeCell<u8>:
            RustSpec<
                Layout = Stable<Robust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
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
        assert_impl_all!(&[UnsafeCell<u8>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&mut [UnsafeCell<u8>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
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
        assert_impl_all!([UnsafeCell<u8>; 2]:
            RustSpec<
                Layout = Stable<Robust>,
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
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithoutNiche,
                Mutability = Interior,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!(&UnsafeCell<NonZero<u8>>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut UnsafeCell<NonZero<u8>>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<NonZero<u8>>>:
            RustSpec<
                Layout = Stable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Stable>,
                Mutability = Interior,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&[UnsafeCell<NonZero<u8>>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        assert_impl_all!(&mut [UnsafeCell<NonZero<u8>>]:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<NonZero<u8>>]>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<NonRobust>,
            >,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<NonZero<u8>>>:
            RustSpec<
                Layout = Unstable<NonRobust>,
                Size = Co3Sized<NonZst>,
                Niche = WithNiche<niche::Unstable>,
                Mutability = Exclusive,
                __IndirectLayout = Stable<Robust>,
            >,
        );
        assert_impl_all!([UnsafeCell<NonZero<u8>>; 2]:
            RustSpec<
                Layout = Stable<NonRobust>,
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
