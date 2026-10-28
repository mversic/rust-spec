#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};
use core::{
    cmp::Reverse,
    convert::Infallible,
    marker::{PhantomData, PhantomPinned},
    mem::{ManuallyDrop, MaybeUninit},
    num::{NonZero, Saturating, Wrapping},
};

#[cfg(feature = "alloc")]
use crate::drop::{AutoDrop, CustomDrop};
use crate::{
    NonOpaque, RustSpec, Stable, Unstable,
    drop::NoDrop,
    layout::{NonRobust, Robust},
    mutability::{Exclusive, WithInterior, WithoutInterior},
    niche::{WithNiche, WithoutNiche},
    size,
};

macro_rules! non_zero_derive {
    ($($primitive:ty),+ $(,)?) => {$(
        unsafe impl RustSpec for NonZero<$primitive> {
            type Layout = Stable;
            type Size = size::Sized<crate::Gt<crate::Zero>>;
            type __Alignment = <$primitive as RustSpec>::__Alignment;
            type Trap = NonRobust;
            type Niche = WithNiche<Stable>;
            type Drop = NoDrop;
            type Mutability = Exclusive<WithoutInterior>;
            type Opacity = NonOpaque;
            type __IndirectTrap = Robust;
        }
    )+};
}

macro_rules! stable_robust_zst {
    (($($generics:tt)*) => $ty:ty) => {
        unsafe impl<$($generics)*> RustSpec for $ty {
            type Layout = Stable;
            type Size = size::Sized<size::Zero>;
            type __Alignment = crate::One;
            type Trap = Robust;
            type Niche = WithoutNiche;
            type Drop = NoDrop;
            type Mutability = Exclusive<WithoutInterior>;
            type Opacity = NonOpaque;
            type __IndirectTrap = Robust;
        }
    };
}

macro_rules! transparent_wrapper {
    (($($generics:tt)*) => $ty:ty) => {
        unsafe impl<$($generics)*> RustSpec for $ty
        where
            T::Drop: crate::drop::DropKind,
        {
            type Layout = T::Layout;
            type Size = T::Size;
            type __Alignment = T::__Alignment;
            type Trap = T::Trap;
            type Niche = T::Niche;
            type Drop = <T::Drop as crate::drop::DropKind>::NeedsDrop;
            type Mutability = T::Mutability;
            type Opacity = NonOpaque;
            type __IndirectTrap = T::__IndirectTrap;
        }
    };
}

non_zero_derive!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
);

stable_robust_zst!(() => ());
stable_robust_zst!(() => Infallible);
stable_robust_zst!(() => PhantomPinned);
stable_robust_zst!((T: ?Sized) => PhantomData<T>);

transparent_wrapper!((T: RustSpec) => Reverse<T>);
transparent_wrapper!((T: RustSpec) => Wrapping<T>);
transparent_wrapper!((T: RustSpec) => Saturating<T>);

unsafe impl<T: RustSpec + ?Sized> RustSpec for ManuallyDrop<T> {
    type Layout = T::Layout;
    type Size = T::Size;
    type __Alignment = T::__Alignment;
    type Trap = T::Trap;
    type Niche = T::Niche;
    type Drop = NoDrop;
    type Mutability = T::Mutability;
    type Opacity = NonOpaque;
    type __IndirectTrap = T::__IndirectTrap;
}

// Keep the projection helper inaccessible outside this implementation block.
const _: () = {
    pub trait UninitMutability {
        type Output;
    }

    impl UninitMutability for Exclusive<WithoutInterior> {
        type Output = Self;
    }
    impl<R> UninitMutability for WithInterior<R> {
        type Output = WithInterior<Robust>;
    }
    impl<R> UninitMutability for Exclusive<WithInterior<R>> {
        type Output = Exclusive<WithInterior<Robust>>;
    }

    unsafe impl<T: RustSpec<Mutability: UninitMutability>> RustSpec for MaybeUninit<T> {
        type Layout = T::Layout;
        type Size = T::Size;
        type __Alignment = T::__Alignment;
        type Trap = Robust;
        type Niche = WithoutNiche;
        type Drop = NoDrop;
        // Preserve shared mutability, but impose no validity requirements on
        // uninitialized contents or anything reachable through those contents.
        type Mutability = <T::Mutability as UninitMutability>::Output;
        type Opacity = NonOpaque;
        type __IndirectTrap = Robust;
    }
};

unsafe impl RustSpec for str {
    type Layout = Stable;
    type Size = size::MetaSized<size::SliceLike>;
    type __Alignment = crate::One;
    type Trap = NonRobust;
    // TODO: This should not be set at all
    type Niche = WithNiche<Unstable>;
    type Drop = NoDrop;
    type Mutability = Exclusive<WithoutInterior>;
    type Opacity = NonOpaque;
    type __IndirectTrap = Robust;
}
#[cfg(feature = "alloc")]
unsafe impl RustSpec for String {
    type Layout = Unstable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type __Alignment = <usize as RustSpec>::__Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type Drop = AutoDrop;
    type Mutability = Exclusive<WithoutInterior>;
    type Opacity = NonOpaque;
    type __IndirectTrap = NonRobust;
}

#[cfg(feature = "alloc")]
unsafe impl<T: RustSpec<Mutability: crate::mutability::PointerMutability>> RustSpec for Vec<T> {
    type Layout = Unstable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type __Alignment = <usize as RustSpec>::__Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type Drop = CustomDrop<AutoDrop>;
    type Mutability = <T::Mutability as crate::mutability::PointerMutability>::Output;
    type Opacity = NonOpaque;
    type __IndirectTrap = T::Trap;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::{Cell, UnsafeCell};
    use static_assertions::assert_impl_all;

    #[test]
    fn maybe_uninit_preserves_mutability_without_inner_traps() {
        assert_impl_all!(MaybeUninit<bool>:
            RustSpec<Mutability = Exclusive<WithoutInterior>, Trap = Robust>);
        assert_impl_all!(MaybeUninit<UnsafeCell<bool>>:
            RustSpec<Mutability = WithInterior<Robust>, Trap = Robust>);
        assert_impl_all!(MaybeUninit<Cell<bool>>:
            RustSpec<Mutability = WithInterior<Robust>, Trap = Robust>);
        assert_impl_all!(MaybeUninit<&'static UnsafeCell<bool>>:
            RustSpec<Mutability = Exclusive<WithInterior<Robust>>, Trap = Robust>);
        assert_impl_all!([MaybeUninit<UnsafeCell<bool>>; 2]:
            RustSpec<Mutability = WithInterior<Robust>>);
        assert_impl_all!(UnsafeCell<MaybeUninit<bool>>:
            RustSpec<Mutability = WithInterior<Robust>>);
        assert_impl_all!(MaybeUninit<MaybeUninit<UnsafeCell<bool>>>:
            RustSpec<Mutability = WithInterior<Robust>>);
        assert_impl_all!(ManuallyDrop<UnsafeCell<bool>>:
            RustSpec<Mutability = WithInterior<NonRobust>, Trap = NonRobust>);

        #[cfg(feature = "alloc")]
        {
            use alloc::boxed::Box;
            assert_impl_all!(Box<MaybeUninit<UnsafeCell<bool>>>:
                RustSpec<Mutability = Exclusive<WithInterior<Robust>>>);
            assert_impl_all!(MaybeUninit<Box<UnsafeCell<bool>>>:
                RustSpec<Mutability = Exclusive<WithInterior<Robust>>, Trap = Robust>);
        }
    }
}
