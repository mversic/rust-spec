#![cfg(feature = "derive")]

use core::{cell::UnsafeCell, ptr::NonNull};
use rust_spec::{
    NonOpaque, RustSpec, Stable, Unstable,
    drop::{CustomDrop, NoDrop},
    layout::{NonRobust, Robust},
    mutability::{Exclusive, WithInterior, WithoutInterior},
    niche::WithoutNiche,
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
#[repr(transparent)]
struct Pointer<T: ?Sized>(*mut T);

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
#[repr(C)]
struct Owner(*mut bool);
impl Drop for Owner {
    fn drop(&mut self) {}
}

#[test]
fn raw_pointer_axes_follow_the_pointee_without_owning_it() {
    assert_impl_all!(*const bool: RustSpec<Layout = Stable, Trap = NonRobust,
        __IndirectTrap = NonRobust, Drop = NoDrop, Niche = WithoutNiche, Opacity = NonOpaque>);
    assert_impl_all!(*mut (u8,): RustSpec<Layout = Unstable, Trap = Robust, Drop = NoDrop>);
    assert_impl_all!(*const UnsafeCell<bool>: RustSpec<Mutability = Exclusive<WithInterior<NonRobust>>, Trap = NonRobust>);
    assert_impl_all!(*mut [UnsafeCell<bool>]: RustSpec<Layout = Unstable, Mutability = Exclusive<WithInterior<NonRobust>>,
        Trap = NonRobust, __IndirectTrap = NonRobust>);
    assert_impl_all!(*const *mut [bool]: RustSpec<Layout = Unstable, Trap = NonRobust>);
    assert_impl_all!(*mut Owner: RustSpec<Drop = NoDrop>);
}

#[test]
fn derived_pointer_wrappers_propagate_axes() {
    assert_impl_all!(Pointer<bool>: RustSpec<Layout = Stable, Trap = NonRobust,
        __IndirectTrap = NonRobust, Drop = NoDrop, Niche = WithoutNiche>);
    assert_impl_all!(Pointer<(u8,)>: RustSpec<Layout = Unstable>);
    assert_impl_all!(Pointer<UnsafeCell<u8>>: RustSpec<Mutability = Exclusive<WithInterior<Robust>>>);
    assert_impl_all!(Pointer<[bool]>: RustSpec<Layout = Unstable, Trap = NonRobust>);
    assert_impl_all!(Owner: RustSpec<Drop = CustomDrop<NoDrop>, Mutability = Exclusive<WithoutInterior>>);
}

#[test]
fn pointer_storage_and_pointee_mutability_are_distinct() {
    assert_impl_all!(NonNull<UnsafeCell<bool>>: RustSpec<Mutability = Exclusive<WithInterior<NonRobust>>,
        Trap = NonRobust, __IndirectTrap = NonRobust>);
    assert_impl_all!(NonNull<[UnsafeCell<bool>]>: RustSpec<Mutability = Exclusive<WithInterior<NonRobust>>>);
    assert_impl_all!(Option<NonNull<UnsafeCell<bool>>>: RustSpec<Mutability = Exclusive<WithInterior<NonRobust>>>);
    assert_impl_all!(UnsafeCell<NonNull<u8>>: RustSpec<Mutability = WithInterior<NonRobust>>);
}
