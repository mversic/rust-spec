#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};
use core::{
    cmp::Reverse,
    convert::Infallible,
    marker::{PhantomData, PhantomPinned},
    mem::ManuallyDrop,
    num::{NonZero, Saturating, Wrapping},
    ptr::NonNull,
};

use crate::{
    RustSpec, Stable, Unstable,
    layout::{NonRobust, Robust},
    mutability::Exclusive,
    niche::{WithNiche, WithoutNiche},
    size,
};

macro_rules! non_zero_derive {
    ($($primitive:ty),+ $(,)?) => {$(
        unsafe impl RustSpec for NonZero<$primitive> {
            type Layout = Stable;
            type Trap = NonRobust;
            type Size = size::Sized<size::NonZst>;
            type Niche = WithNiche<Stable>;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    )+};
}

macro_rules! stable_robust_zst {
    (($($generics:tt)*) => $ty:ty) => {
        unsafe impl<$($generics)*> RustSpec for $ty {
            type Layout = Stable;
            type Trap = Robust;
            type Size = size::Sized<size::Zst>;
            type Niche = WithoutNiche;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    };
}

macro_rules! transparent_wrapper {
    (($($generics:tt)*) => $ty:ty) => {
        unsafe impl<$($generics)*> RustSpec for $ty {
            type Layout = T::Layout;
            type Trap = T::Trap;
            type Size = T::Size;
            type Niche = T::Niche;
            type Mutability = T::Mutability;
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
transparent_wrapper!((T: RustSpec + ?Sized) => ManuallyDrop<T>);

unsafe impl<T: ?Sized> RustSpec for NonNull<T> {
    type Layout = Stable;
    type Trap = NonRobust;
    type Size = size::Sized<size::NonZst>;
    type Niche = WithNiche<Stable>;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

unsafe impl RustSpec for str {
    type Layout = Stable;
    type Trap = NonRobust;
    type Size = size::MetaSized<size::SliceLike>;
    // TODO: This should not be set at all
    type Niche = WithNiche<Unstable>;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}
#[cfg(feature = "alloc")]
unsafe impl RustSpec for String {
    type Layout = Unstable;
    type Trap = NonRobust;
    type Size = size::Sized<size::NonZst>;
    type Niche = WithNiche<Unstable>;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

#[cfg(feature = "alloc")]
unsafe impl<T: RustSpec> RustSpec for Vec<T> {
    type Layout = Unstable;
    type Trap = NonRobust;
    type Size = size::Sized<size::NonZst>;
    type Niche = WithNiche<Unstable>;
    type Mutability = Exclusive;
    type __IndirectTrap = T::Trap;
}
