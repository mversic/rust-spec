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
    TypeSpec,
    mutability::{Exclusive, Interior},
    niche::{WithNiche, WithoutNiche},
    repr::{NonRobust, Robust, Stable, Unstable},
    size::{MetaSized, SliceLike},
};

macro_rules! non_zero_derive {
    ($($primitive:ty),+ $(,)?) => {$(
        unsafe impl TypeSpec for NonZero<$primitive> {
            type Repr = Stable<NonRobust>;
            type Size = crate::size::Sized<crate::size::NonZst>;
            type Niche = WithNiche<crate::niche::Stable>;
            type Mutability = Exclusive;
        })+
    }
}

non_zero_derive! {
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128,
}

unsafe impl TypeSpec for c_void {
    type Repr = Stable<Robust>;
    // TODO: Shouldn't it be ExternTypeLike? I know that `mem::size_of` returns 1.
    type Size = crate::size::Sized<crate::size::NonZst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
}

unsafe impl TypeSpec for () {
    type Repr = Stable<Robust>;
    type Size = crate::size::Sized<crate::size::Zst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
}

unsafe impl<T: ?Sized> TypeSpec for PhantomData<T> {
    type Repr = Stable<Robust>;
    type Size = crate::size::Sized<crate::size::Zst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
}

unsafe impl<T: TypeSpec<Repr: Add<Stable<NonRobust>>> + ?Sized> TypeSpec for NonNull<T> {
    type Repr = <<T as TypeSpec>::Repr as Add<Stable<NonRobust>>>::Output;
    type Size = crate::size::Sized<crate::size::NonZst>;
    type Niche = WithNiche<crate::niche::Stable>;
    type Mutability = Exclusive;
}

unsafe impl TypeSpec for str {
    type Repr = Stable<NonRobust>;
    type Size = MetaSized<SliceLike>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
}

#[cfg(feature = "alloc")]
unsafe impl TypeSpec for String {
    type Repr = Unstable<NonRobust>;
    type Size = crate::size::Sized<crate::size::NonZst>;
    type Niche = WithNiche<crate::niche::Custom>;
    type Mutability = Exclusive;
}

unsafe impl<T: TypeSpec + ?Sized> TypeSpec for UnsafeCell<T> {
    type Repr = T::Repr;
    type Size = T::Size;
    type Niche = WithoutNiche;
    type Mutability = Interior;
}

unsafe impl<T: TypeSpec + ?Sized> TypeSpec for Cell<T> {
    type Repr = T::Repr;
    type Size = T::Size;
    type Niche = WithoutNiche;
    type Mutability = Interior;
}

unsafe impl<T: TypeSpec + ?Sized> TypeSpec for ManuallyDrop<T> {
    type Repr = T::Repr;
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
    //        TypeSpec<Repr = Stable<Robust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithoutNiche>,
    //    );
    //    assert_impl_all!(&ManuallyDrop<u8>:
    //        TypeSpec<Repr = Stable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(&mut ManuallyDrop<u8>:
    //        TypeSpec<Repr = Stable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    #[cfg(feature = "alloc")]
    //    assert_impl_all!(Box<ManuallyDrop<u8>>:
    //        TypeSpec<Repr = Stable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(&[ManuallyDrop<u8>]:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(&mut [ManuallyDrop<u8>]:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    #[cfg(feature = "alloc")]
    //    assert_impl_all!(Box<[ManuallyDrop<u8>]>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    #[cfg(feature = "alloc")]
    //    assert_impl_all!(Vec<ManuallyDrop<u8>>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!([ManuallyDrop<u8>; 2]:
    //        TypeSpec<Repr = Stable<Robust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithoutNiche>,
    //    );
    //    assert_impl_all!(Option<ManuallyDrop<u8>>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );

    //    assert_not_impl_any!(ManuallyDrop<u8>: ReprC);
    //}

    //#[cfg(feature = "alloc")]
    //#[test]
    //fn manually_drop_inner_with_drop() {
    //    assert_impl_all!(ManuallyDrop<String>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(&ManuallyDrop<String>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(&mut ManuallyDrop<String>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(Box<ManuallyDrop<String>>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
    //    );
    //    assert_impl_all!(&[ManuallyDrop<String>]:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(&mut [ManuallyDrop<String>]:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(Box<[ManuallyDrop<String>]>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(Vec<ManuallyDrop<String>>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!([ManuallyDrop<String>; 2]:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
    //    );
    //    assert_impl_all!(Option<ManuallyDrop<String>>:
    //        TypeSpec<Repr = Unstable<NonRobust>>,
    //        TypeSpec<Size = Co3Sized<NonZst>>,
    //        // FIXME:
    //        //TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
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
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = MetaSized<SliceLike>>,
        );

        assert_impl_all!(&str:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(&mut str:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<str>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }

    #[test]
    fn str_decode_rejects_invalid_utf8() {
        assert_impl_all!(str: TypeSpec<Size = MetaSized<SliceLike>>);
        assert_impl_all!(&str: TypeSpec<Size = Co3Sized<NonZst>>);
        assert_impl_all!(&mut str: TypeSpec<Size = Co3Sized<NonZst>>);
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<str>: TypeSpec<Size = Co3Sized<NonZst>>);
    }

    #[test]
    fn robust_unsafe_cell() {
        assert_impl_all!(UnsafeCell<u8>:
            TypeSpec<Repr = Stable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(&UnsafeCell<u8>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut UnsafeCell<u8>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<u8>>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[UnsafeCell<u8>]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&mut [UnsafeCell<u8>]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<u8>]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<u8>>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([UnsafeCell<u8>; 2]:
            TypeSpec<Repr = Stable<Robust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(Option<UnsafeCell<u8>>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }

    #[test]
    fn non_robust_unsafe_cell() {
        assert_impl_all!(UnsafeCell<NonZero<u8>>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(&UnsafeCell<NonZero<u8>>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&mut UnsafeCell<NonZero<u8>>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<UnsafeCell<NonZero<u8>>>:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Stable>>,
        );
        assert_impl_all!(&[UnsafeCell<NonZero<u8>>]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!(&mut [UnsafeCell<NonZero<u8>>]:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Box<[UnsafeCell<NonZero<u8>>]>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        #[cfg(feature = "alloc")]
        assert_impl_all!(Vec<UnsafeCell<NonZero<u8>>>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
        assert_impl_all!([UnsafeCell<NonZero<u8>>; 2]:
            TypeSpec<Repr = Stable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithoutNiche>,
        );
        assert_impl_all!(Option<UnsafeCell<NonZero<u8>>>:
            TypeSpec<Repr = Unstable<NonRobust>>,
            TypeSpec<Size = Co3Sized<NonZst>>,
            TypeSpec<Niche = WithNiche<crate::niche::Custom>>,
        );
    }
}
