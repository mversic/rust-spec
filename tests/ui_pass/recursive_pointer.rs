#![allow(dead_code)]

use core::cell::UnsafeCell;
use rust_spec::{
    RustSpec, Specified, Unspecified, Gt, Zero,
    drop::NoDrop,
    layout::{Robust, NonRobust},
    mutability::{Exclusive, WithoutInterior, WithInterior},
    niche::{WithoutNiche, WithNiche},
    size::{Sized as SpecSized, MetaSized, SliceLike},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
#[repr(C)]
struct Node {
    next: *mut Node,
}

#[derive(RustSpec)]
#[repr(C)]
struct ConstNode(*const Self);

#[derive(RustSpec)]
struct RustNode(*mut Self);

#[derive(RustSpec)]
#[repr(C)]
struct Shared<'a>(&'a Shared<'a>);

#[derive(RustSpec)]
#[repr(C)]
struct Mutable<'a>(&'a mut Self);

#[derive(RustSpec)]
#[repr(C)]
struct Nested<'a>(([&'a Self; 2], u32));

#[derive(RustSpec)]
#[repr(C)]
struct Payload<T> {
    next: *mut Payload<T>,
    payload: T,
}

#[derive(RustSpec)]
#[repr(C)]
#[rust_spec(non_robust)]
struct Invariant(*mut Self);

#[derive(RustSpec)]
#[repr(transparent)]
struct Transparent(*mut Self);

#[derive(RustSpec)]
#[repr(C)]
struct SliceTail {
    next: *const Self,
    tail: [u8],
}

#[derive(RustSpec)]
#[repr(C)]
struct GenericTail<T: ?Sized> {
    next: *const Self,
    tail: T,
}

fn main() {
    assert_impl_all!(Node: RustSpec<Layout = Specified, Size = SpecSized<Gt<Zero>>,
        __Alignment = <usize as RustSpec>::__Alignment, Trap = Robust,
        Niche = WithoutNiche, Drop = NoDrop,
        Mutability = Exclusive<WithoutInterior>, __IndirectTrap = Robust>);
    assert_impl_all!(ConstNode: RustSpec<Layout = Specified, Trap = Robust,
        Mutability = Exclusive<WithoutInterior>, __IndirectTrap = Robust>);
    assert_impl_all!(RustNode: RustSpec<Layout = Unspecified, Trap = Robust>);
    assert_impl_all!(Shared<'static>: RustSpec<Layout = Specified, Trap = NonRobust,
        Niche = WithNiche<Unspecified>, Drop = NoDrop,
        Mutability = Exclusive<WithoutInterior>, __IndirectTrap = NonRobust>);
    assert_impl_all!(Mutable<'static>: RustSpec<Layout = Specified, Trap = NonRobust,
        Mutability = Exclusive<WithoutInterior>, __IndirectTrap = NonRobust>);
    assert_impl_all!(Nested<'static>: RustSpec<Layout = Unspecified, Trap = NonRobust,
        Mutability = Exclusive<WithoutInterior>, __IndirectTrap = NonRobust>);
    assert_impl_all!(Payload<u8>: RustSpec<Layout = Specified, Trap = Robust>);
    assert_impl_all!(Payload<bool>: RustSpec<Trap = NonRobust, __IndirectTrap = NonRobust>);
    assert_impl_all!(Payload<UnsafeCell<u8>>: RustSpec<Mutability = WithInterior<Robust>>);
    assert_impl_all!(*mut Payload<UnsafeCell<bool>>:
        RustSpec<Mutability = Exclusive<WithInterior<NonRobust>>>);
    assert_impl_all!(Invariant: RustSpec<Trap = NonRobust, __IndirectTrap = NonRobust>);
    assert_impl_all!(Transparent: RustSpec<Layout = Specified, Trap = Robust>);
    assert_impl_all!(SliceTail: RustSpec<Layout = Unspecified, Size = MetaSized<SliceLike>,
        Trap = Robust, Mutability = Exclusive<WithoutInterior>>);
    assert_impl_all!(GenericTail<u8>: RustSpec<Layout = Specified, Size = SpecSized<Gt<Zero>>>);
    assert_impl_all!(GenericTail<[u8]>: RustSpec<Layout = Unspecified, Size = MetaSized<SliceLike>>);
}
