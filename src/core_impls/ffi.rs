use core::ffi::{CStr, c_void};

#[cfg(feature = "alloc")]
use alloc::ffi::CString;

#[cfg(feature = "alloc")]
use crate::niche::{Unstable as UnstableNiche, WithNiche};
use crate::{
    RustSpec,
    layout::{NonRobust, Robust, Stable, Unstable},
    mutability::Exclusive,
    niche::WithoutNiche,
    size::{self, MetaSized, SliceLike},
};

unsafe impl RustSpec for c_void {
    type Layout = Stable<Robust>;
    type Size = size::Sized<size::NonZst>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

unsafe impl RustSpec for CStr {
    type Layout = Unstable<NonRobust>;
    type Size = MetaSized<SliceLike>;
    type Niche = WithoutNiche;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}

#[cfg(feature = "alloc")]
unsafe impl RustSpec for CString {
    type Layout = Unstable<NonRobust>;
    type Size = MetaSized<SliceLike>;
    type Niche = WithNiche<UnstableNiche>;
    type Mutability = Exclusive;
    type __IndirectLayout = Stable<Robust>;
}
