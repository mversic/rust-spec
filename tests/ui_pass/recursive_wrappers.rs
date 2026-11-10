#![allow(dead_code)]
use core::{cell::UnsafeCell, ptr::NonNull, pin::Pin};
use rust_spec::{RustSpec, Stable, Unstable, Gt, Zero,
    size::Sized as SpecSized, drop::{NoDrop, AutoDrop},
    layout::{Robust, NonRobust}, mutability::{Exclusive, WithoutInterior, WithInterior},
    niche::{WithNiche, WithoutNiche}};
use static_assertions::assert_impl_all;
use std::collections::VecDeque;

#[derive(RustSpec)]
#[repr(C)]
struct Boxed(Box<Self>);

#[derive(RustSpec)]
#[repr(C)]
struct OptionalBox(Option<Box<Self>>);

#[derive(RustSpec)]
#[repr(C)]
struct OptionalNonNull(Option<NonNull<Self>>);

#[derive(RustSpec)]
struct OptionalVec(Option<Vec<Self>>);

#[derive(RustSpec)]
#[repr(C)]
struct OptionalRef<'a>(Option<&'a Self>);

#[derive(RustSpec)]
#[repr(C)]
struct OptionalRaw(Option<*mut Self>);

#[derive(RustSpec)]
struct Tree<T> { value: T, children: Vec<Tree<T>> }

#[derive(RustSpec)]
enum List<T> { End, Link(T, Box<Self>) }

#[derive(RustSpec)]
#[repr(C)]
struct Nested((Option<Box<Self>>, [NonNull<Self>; 2]));

#[derive(RustSpec)]
struct Queue<T> { value: T, children: VecDeque<Self> }

#[derive(RustSpec)]
struct OptionalQueue(Option<VecDeque<Self>>);

#[derive(RustSpec)]
#[repr(C)]
struct Pinned(Pin<Box<Self>>);

#[derive(RustSpec)]
#[repr(C)]
struct OptionalPinned(Option<Pin<Box<Self>>>);

#[derive(RustSpec)]
#[repr(C)]
struct PinnedRef<'a>(Pin<&'a Self>);

mod alias {
    use rust_spec::RustSpec;
    type Box<T> = std::boxed::Box<T>;
    #[derive(RustSpec)]
    pub struct Node<T> { next: Option<Box<Node<T>>>, value: T }
}

mod ordinary_name {
    use rust_spec::RustSpec;
    #[derive(RustSpec)]
    pub struct Box<T>(T);
    #[derive(RustSpec)]
    pub struct Ordinary(Box<u8>);
}

fn main() {
    assert_impl_all!(Queue<UnsafeCell<bool>>: RustSpec<Layout = Unstable,
        Trap = NonRobust, Drop = AutoDrop, Mutability = WithInterior<NonRobust>,
        __IndirectTrap = NonRobust>);
    assert_impl_all!(OptionalQueue: RustSpec<Layout = Unstable, Trap = NonRobust, Drop = AutoDrop>);
    assert_impl_all!(Pinned: RustSpec<Layout = Stable, Trap = NonRobust,
        Drop = AutoDrop, Size = SpecSized<Gt<Zero>>>);
    assert_impl_all!(OptionalPinned: RustSpec<Layout = Stable, Trap = Robust,
        Drop = AutoDrop, Niche = WithoutNiche>);
    assert_impl_all!(PinnedRef<'static>: RustSpec<Layout = Stable, Trap = NonRobust, Drop = NoDrop>);
    assert_impl_all!(Boxed: RustSpec<Layout = Stable, Trap = NonRobust,
        Size = SpecSized<Gt<Zero>>, Drop = AutoDrop, Mutability = Exclusive<WithoutInterior>,
        Niche = WithNiche<Unstable>, __IndirectTrap = NonRobust>);
    assert_impl_all!(OptionalBox: RustSpec<Layout = Stable, Trap = Robust,
        Drop = AutoDrop, Niche = WithoutNiche, __IndirectTrap = Robust>);
    assert_impl_all!(OptionalNonNull: RustSpec<Layout = Stable, Trap = Robust,
        Drop = NoDrop, Niche = WithoutNiche, __IndirectTrap = Robust>);
    assert_impl_all!(OptionalVec: RustSpec<Layout = Unstable, Trap = NonRobust, Drop = AutoDrop>);
    assert_impl_all!(OptionalRef<'static>: RustSpec<Layout = Stable, Trap = Robust, Drop = NoDrop>);
    assert_impl_all!(OptionalRaw: RustSpec<Layout = Unstable, Trap = NonRobust, Drop = NoDrop>);
    assert_impl_all!(Tree<UnsafeCell<bool>>: RustSpec<Layout = Unstable, Trap = NonRobust,
        Drop = AutoDrop, Mutability = WithInterior<NonRobust>, __IndirectTrap = NonRobust>);
    assert_impl_all!(List<u8>: RustSpec<Layout = Unstable, Trap = NonRobust, Drop = AutoDrop>);
    assert_impl_all!(Nested: RustSpec<Layout = Unstable, Trap = NonRobust, Drop = AutoDrop>);
    assert_impl_all!(alias::Node<bool>: RustSpec<Trap = NonRobust, Drop = AutoDrop>);
    assert_impl_all!(ordinary_name::Ordinary: RustSpec);
}
