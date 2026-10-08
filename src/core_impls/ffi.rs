use core::ffi::{CStr, c_void};

#[cfg(feature = "alloc")]
use alloc::ffi::CString;

use crate::{
    RustSpec, Stable, Unstable,
    drop::NoDrop,
    layout::{NonRobust, Robust},
    mutability::Exclusive,
    niche::WithoutNiche,
    size::{self, NulTerminated},
};
#[cfg(feature = "alloc")]
use crate::{
    drop::{Inner, WithDrop},
    niche::WithNiche,
};

unsafe impl RustSpec for c_void {
    type Layout = Stable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type Alignment = crate::One;
    type Trap = Robust;
    type Niche = WithoutNiche;
    type Drop = NoDrop;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

unsafe impl RustSpec for CStr {
    type Layout = Unstable;
    type Size = NulTerminated;
    type Alignment = crate::One;
    type Trap = NonRobust;
    type Niche = WithoutNiche;
    type Drop = NoDrop;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}

#[cfg(feature = "alloc")]
unsafe impl RustSpec for CString {
    type Layout = Unstable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type Alignment = <usize as RustSpec>::Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type Drop = WithDrop<Inner>;
    type Mutability = Exclusive;
    type __IndirectTrap = Robust;
}
