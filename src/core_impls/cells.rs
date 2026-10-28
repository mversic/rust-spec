use core::{
    cell::{Cell, UnsafeCell},
    ops::Add,
};

use crate::{
    NonOpaque, RustSpec, drop::NoDrop, layout::Robust, mutability::WithInterior,
    niche::WithoutNiche,
};

macro_rules! interior_cell {
    ($($ty:ident),+ $(,)?) => {$(
        unsafe impl<T: RustSpec + ?Sized> RustSpec for $ty<T>
        where
            Robust: Add<T::Trap>,
            NoDrop: Add<T::Drop>,
        {
            type Layout = T::Layout;
            type Size = T::Size;
            type Alignment = T::Alignment;
            type Trap = <Robust as Add<T::Trap>>::Output;
            type Niche = WithoutNiche;
            type Drop = <NoDrop as Add<T::Drop>>::Output;
            type Mutability = WithInterior<T::Trap>;
            type Opacity = NonOpaque;
            type __IndirectTrap = T::__IndirectTrap;
        }
    )+};
}

interior_cell!(UnsafeCell, Cell);
