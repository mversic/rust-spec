use core::{
    cell::{Cell, UnsafeCell},
    ops::Add,
};

use crate::{
    RustSpec,
    layout::{Robust, Unstable},
    mutability::Interior,
    niche::WithoutNiche,
};

macro_rules! interior_cell {
    ($($ty:ident),+ $(,)?) => {$(
        unsafe impl<T: RustSpec + ?Sized> RustSpec for $ty<T>
        where
            Unstable<Robust>: Add<T::Layout>,
        {
            // NOTE: https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html#memory-layout
            // The documentation says it is never valid to transmute between UnsafeCell<T> and T
            type Layout = <Unstable<Robust> as Add<T::Layout>>::Output;
            type Size = T::Size;
            type Niche = WithoutNiche;
            type Mutability = Interior;
            type __IndirectLayout = T::__IndirectLayout;
        }
    )+};
}

interior_cell!(UnsafeCell, Cell);
