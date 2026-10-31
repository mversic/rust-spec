use core::{
    cell::{Cell, UnsafeCell},
    ops::Add,
};

use crate::{
    NonOpaque, RustSpec, Sum, drop::NeedsDrop, layout::Robust, mutability::WithInterior,
    niche::WithoutNiche,
};

macro_rules! interior_cell {
    ($($ty:ident),+ $(,)?) => {$(
        unsafe impl<T: RustSpec + ?Sized> RustSpec for $ty<T>
        where
            Robust: Add<T::Trap>,
            T::Drop: crate::drop::DropKind,
        {
            type Layout = T::Layout;
            type Size = T::Size;
            type __Alignment = T::__Alignment;
            type Trap = Sum<Robust, T::Trap>;
            type Niche = WithoutNiche;
            type Drop = NeedsDrop<T::Drop>;
            type Mutability = WithInterior<T::Trap>;
            type Opacity = NonOpaque;
            type __IndirectTrap = T::__IndirectTrap;
        }
    )+};
}

interior_cell!(UnsafeCell, Cell);
