#[cfg(feature = "alloc")]
use alloc::ffi::CString;
use core::ffi::{CStr, c_void};

#[cfg(feature = "alloc")]
use crate::drop::{AutoDrop, CustomDrop};
use crate::{
    NonOpaque, RustSpec, Stable, Unstable,
    drop::NoDrop,
    layout::{NonRobust, Robust},
    mutability::{Exclusive, WithoutInterior},
    niche::{WithNiche, WithoutNiche},
    size::{self, NulTerminated},
};

unsafe impl RustSpec for c_void {
    type Layout = Stable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type Alignment = crate::One;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type Drop = NoDrop;
    type Mutability = Exclusive<WithoutInterior>;
    type Opacity = NonOpaque;
    type __IndirectTrap = Robust;
}

unsafe impl RustSpec for CStr {
    type Layout = Unstable;
    type Size = NulTerminated;
    type Alignment = crate::One;
    type Trap = NonRobust;
    type Niche = WithoutNiche;
    type Drop = NoDrop;
    type Mutability = Exclusive<WithoutInterior>;
    type Opacity = NonOpaque;
    type __IndirectTrap = Robust;
}

#[cfg(feature = "alloc")]
unsafe impl RustSpec for CString {
    type Layout = Unstable;
    type Size = size::Sized<crate::Gt<crate::Zero>>;
    type Alignment = <usize as RustSpec>::Alignment;
    type Trap = NonRobust;
    type Niche = WithNiche<Unstable>;
    type Drop = CustomDrop<AutoDrop>;
    type Mutability = Exclusive<WithoutInterior>;
    type Opacity = NonOpaque;
    type __IndirectTrap = Robust;
}
