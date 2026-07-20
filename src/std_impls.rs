#[cfg(feature = "alloc")]
use alloc::string::String;
use core::{
    cell::{Cell, UnsafeCell},
    ffi::c_void,
    marker::PhantomData,
    mem::ManuallyDrop,
    num::NonZero,
    ops::Add,
    ptr::NonNull,
};

use crate::{
    RustSpec,
    layout::{NonRobust, Robust, Stable, Unstable},
    mutability::{Exclusive, Interior},
    niche::{WithNiche, WithoutNiche},
    size::{MetaSized, SliceLike},
};

macro_rules! non_zero_derive {
    ($($primitive:ty),+ $(,)?) => {$(
        unsafe impl RustSpec for NonZero<$primitive> {
            type Layout = Stable<NonRobust>;
            type Size = crate::size::Sized<crate::size::NonZst>;
            type Niche = WithNiche<crate::niche::Stable>;
            type Mutability = Exclusive;
        })+
    }
}

non_zero_derive! {
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128,
}

unsafe impl RustSpec for c_void {
    type Layout = Stable<Robust>;
    // TODO: Shouldn't it be ExternTypeLike? I know that `mem::size_of` returns 1.
    type Size = crate::size::Sized<crate::size::NonZst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
}

unsafe impl RustSpec for () {
    type Layout = Stable<Robust>;
    type Size = crate::size::Sized<crate::size::Zst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
}

unsafe impl<T: ?Sized> RustSpec for PhantomData<T> {
    type Layout = Stable<Robust>;
    type Size = crate::size::Sized<crate::size::Zst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
}

unsafe impl<T: RustSpec<Layout: Add<Stable<NonRobust>>> + ?Sized> RustSpec for NonNull<T> {
    type Layout = <<T as RustSpec>::Layout as Add<Stable<NonRobust>>>::Output;
    type Size = crate::size::Sized<crate::size::NonZst>;
    type Niche = WithNiche<crate::niche::Stable>;
    type Mutability = Exclusive;
}

unsafe impl RustSpec for str {
    type Layout = Stable<NonRobust>;
    type Size = MetaSized<SliceLike>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
}

#[cfg(feature = "alloc")]
unsafe impl RustSpec for String {
    type Layout = Unstable<NonRobust>;
    type Size = crate::size::Sized<crate::size::NonZst>;
    type Niche = WithNiche<crate::niche::Custom>;
    type Mutability = Exclusive;
}

unsafe impl<T: RustSpec + ?Sized> RustSpec for UnsafeCell<T> {
    type Layout = T::Layout;
    type Size = T::Size;
    type Niche = WithoutNiche;
    type Mutability = Interior;
}

unsafe impl<T: RustSpec + ?Sized> RustSpec for Cell<T> {
    type Layout = T::Layout;
    type Size = T::Size;
    type Niche = WithoutNiche;
    type Mutability = Interior;
}

unsafe impl<T: RustSpec + ?Sized> RustSpec for ManuallyDrop<T> {
    type Layout = T::Layout;
    type Size = T::Size;
    type Niche = T::Niche;
    type Mutability = T::Mutability;
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::boxed::Box;

    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    use super::*;
    #[cfg(feature = "alloc")]
    use crate::{
        niche::WithNiche,
        size::{NonZst, Sized as Co3Sized},
    };

    // TODO: Enable
    //#[test]
    //fn manually_drop_inner_without_drop() {
    //    assert_impl_all!(ManuallyDrop<u8>:
    //        RustSpec<Layout = Stable<Robust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithoutNiche>,
    //    );
    //    assert_impl_all!(&ManuallyDrop<u8>:
    //        RustSpec<Layout = Stable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(&mut ManuallyDrop<u8>:
    //        RustSpec<Layout = Stable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    #[cfg(feature = "alloc")]
    //    assert_impl_all!(Box<ManuallyDrop<u8>>:
    //        RustSpec<Layout = Stable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(&[ManuallyDrop<u8>]:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(&mut [ManuallyDrop<u8>]:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    #[cfg(feature = "alloc")]
    //    assert_impl_all!(Box<[ManuallyDrop<u8>]>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    #[cfg(feature = "alloc")]
    //    assert_impl_all!(Vec<ManuallyDrop<u8>>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!([ManuallyDrop<u8>; 2]:
    //        RustSpec<Layout = Stable<Robust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithoutNiche>,
    //    );
    //    assert_impl_all!(Option<ManuallyDrop<u8>>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );

    //    assert_not_impl_any!(ManuallyDrop<u8>: ReprC);
    //}

    //#[cfg(feature = "alloc")]
    //#[test]
    //fn manually_drop_inner_with_drop() {
    //    assert_impl_all!(ManuallyDrop<String>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(&ManuallyDrop<String>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(&mut ManuallyDrop<String>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(Box<ManuallyDrop<String>>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(&[ManuallyDrop<String>]:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(&mut [ManuallyDrop<String>]:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(Box<[ManuallyDrop<String>]>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(Vec<ManuallyDrop<String>>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!([ManuallyDrop<String>; 2]:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(Option<ManuallyDrop<String>>:
    //        RustSpec<Layout = Unstable<NonRobust>>,
    //        RustSpec<Size = Co3Sized<NonZst>>,
    //        // FIXME:
    //        //RustSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_not_impl_any!(ManuallyDrop<String>: ReprC);

    //    #[cfg(feature = "alloc")]
    //    assert_not_impl_any!(Box<[ManuallyDrop<String>]>: Encode, Decode<'static>);
    //    #[cfg(feature = "alloc")]
    //    assert_not_impl_any!(Vec<ManuallyDrop<String>>: Encode, Decode<'static>);
    //}

    #[test]
    fn str_is_supported() {
        assert_impl_all!(str:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = MetaSized<SliceLike>>,
        );

        assert_impl_all!(&str:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut str:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<str>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }

    #[test]
    fn str_decode_rejects_invalid_utf8() {
        assert_impl_all!(str: RustSpec<Size = MetaSized<SliceLike>>);
        assert_impl_all!(&str: RustSpec<Size = Co3Sized<NonZst>>);
        assert_impl_all!(&mut str: RustSpec<Size = Co3Sized<NonZst>>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<str>: RustSpec<Size = Co3Sized<NonZst>>);
    }

    #[test]
    fn robust_unsafe_cell() {
        assert_impl_all!(UnsafeCell<u8>:
            RustSpec<Layout = Stable<Robust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(&UnsafeCell<u8>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut UnsafeCell<u8>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[UnsafeCell<u8>]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&mut [UnsafeCell<u8>]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([UnsafeCell<u8>; 2]:
            RustSpec<Layout = Stable<Robust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(Option<UnsafeCell<u8>>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }

    #[test]
    fn non_robust_unsafe_cell() {
        assert_impl_all!(UnsafeCell<NonZero<u8>>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(&UnsafeCell<NonZero<u8>>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut UnsafeCell<NonZero<u8>>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<NonZero<u8>>>:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[UnsafeCell<NonZero<u8>>]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&mut [UnsafeCell<NonZero<u8>>]:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<NonZero<u8>>]>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<NonZero<u8>>>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([UnsafeCell<NonZero<u8>>; 2]:
            RustSpec<Layout = Stable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(Option<UnsafeCell<NonZero<u8>>>:
            RustSpec<Layout = Unstable<NonRobust>>,
            RustSpec<Size = Co3Sized<NonZst>>,
            RustSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }
}
