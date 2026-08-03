use core::ffi::{CStr, c_void};

#[cfg(feature = "alloc")]
use alloc::ffi::CString;

#[cfg(feature = "alloc")]
use crate::niche::WithNiche;
use crate::{
    RustSpec, Stable, Unstable,
    layout::{NonRobust, Robust},
    mutability::Exclusive,
    niche::WithoutNiche,
    size::{self, MetaSized, SliceLike},
};

unsafe impl RustSpec for c_void {
    type Layout = Stable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type Alignment = crate::One;
    type Trap = Robust;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

unsafe impl RustSpec for CStr {
    type Layout = Unstable;
    type Size = MetaSized<SliceLike>;
    type Alignment = crate::One;
    type Trap = NonRobust;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

#[cfg(feature = "alloc")]
unsafe impl RustSpec for CString {
    type Layout = Unstable;
    type Size = MetaSized<SliceLike>;
    type Alignment = <usize as RustSpec>::Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}
