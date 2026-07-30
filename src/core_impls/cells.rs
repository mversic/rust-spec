use core::{
    cell::{Cell, UnsafeCell},
    ops::Add,
};

use crate::{RustSpec, Stable, layout::Robust, mutability::Interior, niche::WithoutNiche};

macro_rules! interior_cell {
    ($($ty:ident),+ $(,)?) => {$(
        unsafe impl<T: RustSpec + ?Sized> RustSpec for $ty<T>
        where
            Robust: Add<T::Trap>,
        {
            type Layout = Stable;
            type Trap = <Robust as Add<T::Trap>>::Output;
            type Size = T::Size;
            type Niche = WithoutNiche;
            type Mutability = Interior;
            type __IndirectTrap = T::__IndirectTrap;
        }
    )+};
}

interior_cell!(UnsafeCell, Cell);
