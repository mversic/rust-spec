#![cfg(feature = "derive")]
#![allow(dead_code)]

use core::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    num::NonZeroU8,
    ptr::NonNull,
};
use rust_spec::{NonOpaque, Opaque, RustSpec};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
#[rust_spec(opaque)]
struct Hidden(*mut u8);

#[derive(RustSpec)]
#[rust_spec(opaque)]
enum ExplicitEnum {
    Value(u8),
}

#[derive(RustSpec)]
#[rust_spec(opaque)]
union ExplicitUnion {
    value: u8,
}

#[derive(RustSpec)]
struct Pair<A, B>(A, B);

#[derive(RustSpec)]
struct Mixed<T>(Hidden, T, u8);

#[derive(RustSpec)]
#[repr(transparent)]
struct Wrapper<T>(T);

#[derive(RustSpec)]
enum Choice<A, B> {
    A(A),
    B(B),
}

#[derive(RustSpec)]
#[repr(C)]
enum CChoice<A, B> {
    A(A),
    B(B),
}

#[derive(RustSpec)]
union Storage<T> {
    value: ManuallyDrop<T>,
    byte: u8,
}

#[derive(RustSpec)]
struct Empty;

#[test]
fn only_explicit_derives_are_opaque() {
    assert_impl_all!(Hidden: RustSpec<Opacity = Opaque>);
    assert_impl_all!(ExplicitEnum: RustSpec<Opacity = Opaque>);
    assert_impl_all!(ExplicitUnion: RustSpec<Opacity = Opaque>);
    assert_impl_all!(Pair<Hidden, u8>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Pair<u8, Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Pair<Hidden, Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Pair<u8, u16>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Mixed<u8>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Mixed<Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Wrapper<Pair<Hidden, u8>>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Choice<Hidden, u8>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Choice<u8, Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(CChoice<u8, Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Storage<Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Storage<u8>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Empty: RustSpec<Opacity = NonOpaque>);
}

#[test]
fn stored_values_do_not_propagate_opacity() {
    assert_impl_all!((Hidden, u8): RustSpec<Opacity = NonOpaque>);
    assert_impl_all!((u8, Hidden): RustSpec<Opacity = NonOpaque>);
    assert_impl_all!((Hidden, Hidden): RustSpec<Opacity = NonOpaque>);
    assert_impl_all!([Hidden; 2]: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!([Hidden]: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Option<Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Result<Hidden, u8>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Result<u8, Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Result<Hidden, ()>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Result<(), Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Result<NonZeroU8, ()>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(Cell<Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(UnsafeCell<Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(ManuallyDrop<Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(MaybeUninit<Hidden>: RustSpec<Opacity = NonOpaque>);
}

#[test]
fn indirection_and_absent_values_are_nonopaque() {
    assert_impl_all!(&Hidden: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(&mut Hidden: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(*const Hidden: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(*mut Hidden: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(NonNull<Hidden>: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!([Hidden; 0]: RustSpec<Opacity = NonOpaque>);
    assert_impl_all!(PhantomData<Hidden>: RustSpec<Opacity = NonOpaque>);
    #[cfg(feature = "alloc")]
    {
        assert_impl_all!(Box<Hidden>: RustSpec<Opacity = NonOpaque>);
        assert_impl_all!(Vec<Hidden>: RustSpec<Opacity = NonOpaque>);
    }
}
