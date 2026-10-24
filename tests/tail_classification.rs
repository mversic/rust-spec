#![cfg(feature = "derive")]
#![allow(dead_code)]

use rust_spec::{
    Gt, RustSpec, Stable, Zero,
    layout::Robust,
    size::{MetaSized, Sized as SpecSized, SliceLike},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
#[repr(C)]
struct SizedTail<T> {
    prefix: (),
    tail: T,
}

#[derive(RustSpec)]
#[repr(C)]
struct SliceTail<T> {
    prefix: u8,
    tail: [T],
}

#[derive(RustSpec)]
struct TupleTail<T: ?Sized> {
    tail: (u8, T),
}

#[derive(RustSpec)]
struct ZeroTail {
    tail: (),
}

#[derive(RustSpec)]
#[repr(C)]
struct ZeroFields {
    first: [u8; 0],
    tail: (),
}

#[derive(RustSpec)]
#[repr(C)]
struct NonzeroPrefix {
    first: u8,
    tail: (),
}

#[test]
fn tail_size_classification() {
    assert_impl_all!(SizedTail<u8>: RustSpec<Size = SpecSized<Gt<Zero>>>);
    assert_impl_all!(SizedTail<()>: RustSpec<Size = SpecSized<Zero>>);
    assert_impl_all!(ZeroTail: RustSpec<Size = SpecSized<Zero>>);
    assert_impl_all!(ZeroFields: RustSpec<Size = SpecSized<Zero>>);
    assert_impl_all!(NonzeroPrefix: RustSpec<Size = SpecSized<Gt<Zero>>>);
    assert_impl_all!(SliceTail<u8>: RustSpec<Size = MetaSized<SliceLike>>);
    assert_impl_all!(TupleTail<[u8]>: RustSpec<Size = MetaSized<SliceLike>>);
}

#[test]
fn primitive_spelling_does_not_override_field_size() {
    #[allow(non_camel_case_types)]
    type u8 = ();

    #[derive(RustSpec)]
    struct Shadowed {
        tail: u8,
    }

    assert_impl_all!(Shadowed: RustSpec<Size = SpecSized<Zero>>);
}

// A named wrapper owns the classification of its type arguments.
struct Unclassified;
struct Hidden<T>(T);
unsafe impl RustSpec for Hidden<*mut Unclassified> {
    type Layout = Stable;
    type Size = SpecSized<Gt<Zero>>;
    type Alignment = <usize as RustSpec>::Alignment;
    type Trap = Robust;
    type Niche = rust_spec::niche::WithoutNiche;
    type Drop = rust_spec::drop::NoDrop;
    type Mutability = rust_spec::mutability::Exclusive;
    type Opacity = rust_spec::NonOpaque;
    type __IndirectTrap = Robust;
}

#[derive(RustSpec)]
#[repr(C)]
struct ContainsHidden {
    field: Hidden<*mut Unclassified>,
}

#[test]
fn named_wrapper_arguments_are_not_inspected() {
    assert_impl_all!(ContainsHidden: RustSpec<Layout = Stable, Trap = Robust>);
}
