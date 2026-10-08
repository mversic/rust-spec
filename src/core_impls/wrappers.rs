#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};
use core::{
    cmp::Reverse,
    convert::Infallible,
    marker::{PhantomData, PhantomPinned},
    mem::{ManuallyDrop, MaybeUninit},
    num::{NonZero, Saturating, Wrapping},
    ops::Add,
    ptr::NonNull,
};

#[cfg(feature = "alloc")]
use crate::drop::{CustomDrop, InnerDrop};
use crate::{
    RustSpec, Stable, Unstable,
    drop::NoDrop,
    layout::{NonRobust, Robust},
    mutability::Exclusive,
    niche::{WithNiche, WithoutNiche},
    size,
};

macro_rules! non_zero_derive {
    ($($primitive:ty),+ $(,)?) => {$(
        unsafe impl RustSpec for NonZero<$primitive> {
            type Layout = Stable;
            type Size = size::Sized<crate::Gt<crate::Zero>>;
            type Alignment = <$primitive as RustSpec>::Alignment;
            type Trap = NonRobust;
            type Niche = WithNiche<Stable>;
            type Drop = NoDrop;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    )+};
}

macro_rules! stable_robust_zst {
    (($($generics:tt)*) => $ty:ty) => {
        unsafe impl<$($generics)*> RustSpec for $ty {
            type Layout = Stable;
            type Size = size::Sized<size::Zero>;
            type Alignment = crate::One;
            type Trap = Robust;
            type Niche = WithoutNiche;
            type Drop = NoDrop;
            type Mutability = Exclusive;
            type __IndirectTrap = Robust;
        }
    };
}

macro_rules! transparent_wrapper {
    (($($generics:tt)*) => $ty:ty) => {
        unsafe impl<$($generics)*> RustSpec for $ty
        where
            NoDrop: Add<T::Drop>,
        {
            type Layout = T::Layout;
            type Size = T::Size;
            type Alignment = T::Alignment;
            type Trap = T::Trap;
            type Niche = T::Niche;
            type Drop = <NoDrop as Add<T::Drop>>::Output;
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

unsafe impl<T: RustSpec + ?Sized> RustSpec for ManuallyDrop<T> {
    type Layout = T::Layout;
    type Size = T::Size;
    type Alignment = T::Alignment;
    type Trap = T::Trap;
    type Niche = T::Niche;
    type Drop = NoDrop;
    type Mutability = T::Mutability;
    type __IndirectTrap = T::__IndirectTrap;
}

unsafe impl<T: RustSpec> RustSpec for MaybeUninit<T> {
    type Layout = T::Layout;
    type Size = T::Size;
    type Alignment = T::Alignment;
    type Trap = Robust;
    type Niche = WithoutNiche;
    type Drop = NoDrop;
    type Mutability = T::Mutability;
    type __IndirectTrap = Robust;
}

unsafe impl<T: ?Sized> RustSpec for NonNull<T> {
    type Layout = Stable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type Alignment = <usize as RustSpec>::Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<Stable>;
    type Drop = NoDrop;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

unsafe impl RustSpec for str {
    type Layout = Stable;
    type Size = size::MetaSized<size::SliceLike>;
    type Alignment = crate::One;
    type Trap = NonRobust;
    // TODO: This should not be set at all
    type Niche = WithNiche<Unstable>;
    type Drop = NoDrop;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}
#[cfg(feature = "alloc")]
unsafe impl RustSpec for String {
    type Layout = Unstable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type Alignment = <usize as RustSpec>::Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type Drop = InnerDrop;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

#[cfg(feature = "alloc")]
unsafe impl<T: RustSpec> RustSpec for Vec<T>
where
    NoDrop: Add<T::Drop>,
    CustomDrop: Add<T::Drop>,
{
    type Layout = Unstable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type Alignment = <usize as RustSpec>::Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type Drop = <CustomDrop as Add<T::Drop>>::Output;
    type Mutability = Exclusive;
    type __IndirectTrap = T::Trap;
}
