#![cfg(feature = "derive")]
#![allow(dead_code)]
use core::{cell::Cell, ptr::NonNull};

use rust_spec::{
    RustSpec, Stable, Unstable,
    drop::NoDrop,
    mutability::{Exclusive, WithInterior},
    niche::{WithNiche, WithoutNiche},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
enum Generic<T> {
    Empty,
    Value(T),
}
#[derive(RustSpec)]
#[repr(C)]
enum GenericC<T> {
    Empty,
    Value(T),
}
#[derive(RustSpec)]
#[repr(u8)]
enum GenericTagged<T> {
    Empty,
    Value(T),
}
#[derive(RustSpec)]
#[repr(C, u8)]
enum GenericCTagged<T> {
    Empty,
    Value(T),
}
#[derive(RustSpec)]
#[repr(transparent)]
enum Transparent<T> {
    Value(T),
}
#[derive(RustSpec)]
enum Singleton<T> {
    Value(T),
}

#[test]
fn enum_mutability_follows_payloads() {
    macro_rules! check {
        ($($wrapper:ident),*) => {$(
            assert_impl_all!($wrapper<Cell<u8>>: RustSpec<Mutability = WithInterior>);
            assert_impl_all!($wrapper<u8>: RustSpec<Mutability = Exclusive>);
        )*};
    }
    check!(
        Generic,
        GenericC,
        GenericTagged,
        GenericCTagged,
        Transparent,
        Singleton
    );
    assert_impl_all!(Option<Cell<u8>>: RustSpec<Mutability = WithInterior>);
    assert_impl_all!(Result<Cell<u8>, ()>: RustSpec<Mutability = WithInterior>);
    assert_impl_all!(Result<(), Cell<u8>>: RustSpec<Mutability = WithInterior>);
    assert_impl_all!(Result<u8, Cell<u8>>: RustSpec<Mutability = WithInterior>);
    assert_impl_all!(Result<Cell<u8>, u8>: RustSpec<Mutability = WithInterior>);
    // A nested payload still permits mutation through a shared reference.
    let value = GenericC::Value(Some(Cell::new(0)));
    assert_impl_all!(GenericC<Option<Cell<u8>>>: RustSpec<Mutability = WithInterior>);
    if let GenericC::Value(Some(cell)) = &value {
        cell.set(1);
        assert_eq!(cell.get(), 1);
    }
}

#[derive(RustSpec)]
#[repr(C)]
struct Tail<T: ?Sized> {
    prefix: u8,
    tail: T,
}
#[derive(RustSpec)]
#[repr(transparent)]
struct Pointer<T: ?Sized>(NonNull<T>);

trait Object {}
impl Object for u32 {}
unsafe impl RustSpec for dyn Object + '_ {
    type Layout = Unstable;
    type Size = rust_spec::size::MetaSized<rust_spec::size::DynTraitLike>;
    type Alignment = rust_spec::Gt<rust_spec::One>;
    type Trap = rust_spec::layout::NonRobust;
    type Niche = WithoutNiche;
    type Drop = NoDrop;
    type Mutability = Exclusive;
    type Opacity = rust_spec::NonOpaque;
    type __IndirectTrap = rust_spec::layout::Robust;
}

#[test]
fn nonnull_layout_follows_pointer_shape() {
    assert_impl_all!(NonNull<u8>: RustSpec<Layout = Stable, Niche = WithNiche<Stable>>);
    assert_impl_all!(Option<NonNull<u8>>: RustSpec<Layout = Stable>);
    macro_rules! check {
        ($($ty:ty),*) => {$(
            assert_impl_all!(NonNull<$ty>: RustSpec<Layout = Unstable, Niche = WithNiche<Stable>>);
            assert_impl_all!(Option<NonNull<$ty>>: RustSpec<Layout = Unstable>);
            assert_impl_all!(Pointer<$ty>: RustSpec<Layout = Unstable>);
        )*};
    }
    check!([u8], str, dyn Object, Tail<[u8]>, Tail<dyn Object>);
}

#[test]
fn result_supports_all_payload_niche_combinations() {
    use core::num::NonZeroU8;

    macro_rules! check {
        ($ok:ty, $err:ty) => {
            assert_impl_all!(Result<$ok, $err>: RustSpec<
                Layout = Unstable,
                Size = rust_spec::size::Sized<rust_spec::Gt<rust_spec::Zero>>,
                Trap = rust_spec::layout::NonRobust,
                Niche = WithNiche<Unstable>,
            >);
        };
    }
    check!(u8, u8);
    check!(bool, u8);
    check!(u8, bool);
    check!(bool, bool);
    check!(NonZeroU8, u8);
    check!(u8, NonZeroU8);
    check!(NonZeroU8, bool);
    check!(bool, NonZeroU8);
    check!(NonZeroU8, NonZeroU8);
}
