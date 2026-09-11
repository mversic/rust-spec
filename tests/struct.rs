use core::{cell::UnsafeCell, num::NonZero as StdNonZero, num::NonZeroU8, ops::Add};
use std::ffi::c_void;

use rust_spec::{
    RustSpec, Stable, Unstable,
    layout::{NonRobust, Robust},
    mutability::{Exclusive, Interior},
    niche::{WithNiche, WithoutNiche},
    size::{self, Gt, MetaSized, Sized as SpecSized, SliceLike, Zero},
};
use static_assertions::assert_impl_all;

pub trait Projection {
    type Kita<'a>
    where
        Self: 'a;
}

impl Projection for c_void {
    type Kita<'a>
        = &'a u8
    where
        Self: 'a;
}

pub trait CTypeProjection {
    type CType;
}

impl CTypeProjection for u8 {
    type CType = u8;
}

impl<T: CTypeProjection> CTypeProjection for UnsafeCell<T> {
    type CType = T::CType;
}

#[derive(RustSpec)]
pub struct ParamReprCZst<T: ?Sized> {
    pub a: (),
    pub b: T,
}

#[derive(RustSpec)]
pub struct WithGat1<'b>(pub <c_void as Projection>::Kita<'b>)
where
    for<'d> c_void: Projection,
    Self: 'b;

#[derive(RustSpec)]
#[repr(C)]
pub struct WithGat2<'b>(pub <u8 as Projection>::Kita<'b>)
where
    for<'d> u8: Projection,
    Self: 'b;

#[derive(RustSpec)]
#[repr(transparent)]
pub struct Custom2View<'_dšč, 'a>(pub <&'a i8 as Projection>::Kita<'_dšč>)
where
    Self: '_dšč,
    for<'_dummy> &'a i8: Projection;

#[derive(RustSpec)]
#[repr(transparent)]
pub struct CounterData(<usize as CTypeProjection>::CType)
where
    for<'__dummy> usize: CTypeProjection;

#[derive(RustSpec)]
#[repr(C)]
pub struct OpaqueData(<usize as CTypeProjection>::CType)
where
    for<'__dummy> usize: CTypeProjection;

#[derive(RustSpec)]
struct Empty;

#[derive(RustSpec)]
pub struct RustStruct {
    pub value: u8,
}

#[derive(RustSpec)]
pub struct NonRobustRustRepr {
    pub value: bool,
}

#[derive(RustSpec)]
#[repr(C)]
struct ReprCStruct {
    value: u8,
}

#[derive(RustSpec)]
#[repr(C)]
struct ReprCProjectionCTypeFields {
    first: <UnsafeCell<u8> as CTypeProjection>::CType,
    second: <u8 as CTypeProjection>::CType,
}

#[derive(RustSpec)]
pub struct InteriorWrapper {
    _left: UnsafeCell<u8>,
    pub right: UnsafeCell<u16>,
}

#[derive(RustSpec)]
pub struct MixedMutability {
    pub cell: UnsafeCell<u8>,
    pub value: u8,
}

#[derive(RustSpec)]
pub struct Wrapper<T> {
    pub value: T,
}

#[derive(RustSpec)]
#[repr(C)]
struct Pair<T, U> {
    first: T,
    second: U,
}

#[derive(RustSpec)]
pub struct SingletonWithNiche(pub NonZeroU8);

#[derive(RustSpec)]
#[repr(transparent)]
struct TransparentWithNiche(NonZeroU8);

#[derive(RustSpec)]
#[repr(transparent)]
struct TransparentWithMultipleFields((), NonZeroU8);

#[derive(RustSpec)]
#[repr(transparent)]
struct TransparentZstStruct;

struct NotRustSpec;

#[derive(RustSpec)]
#[repr(C)]
struct RawPointer<T> {
    pointer: *mut T,
}

#[derive(RustSpec)]
#[repr(transparent)]
struct Bytes([u8]);

#[derive(RustSpec)]
pub struct Packet {
    pub tag: NonZeroU8,
    pub payload: [u8],
}

#[derive(RustSpec)]
pub struct TuplePacket(pub NonZeroU8, pub [u8]);

#[derive(RustSpec)]
#[repr(transparent)]
pub struct TransparentSlice<T>([T]);

fn assert_rust_spec<T: RustSpec + ?Sized>() {}

fn a_slice_of_any_rust_spec_type_is_rust_spec<T: RustSpec>() {
    assert_rust_spec::<[T]>();
}

fn a_transparent_slice_wrapper_of_any_rust_spec_type_is_rust_spec<T: RustSpec>()
where
    WithoutNiche: Add<T::Niche>,
{
    assert_rust_spec::<TransparentSlice<T>>();
}

#[test]
fn generic_slice_classification() {
    a_slice_of_any_rust_spec_type_is_rust_spec::<u8>();
    a_transparent_slice_wrapper_of_any_rust_spec_type_is_rust_spec::<u8>();
}

#[test]
fn struct_classification() {
    assert_impl_all!(ParamReprCZst<()>:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Zero>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ParamReprCZst<UnsafeCell<NonZeroU8>>:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithoutNiche,
            // FIXME: Should this be Interior?
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ParamReprCZst<u8>:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(SingletonWithNiche:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(TransparentWithNiche:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Stable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TransparentWithMultipleFields:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TransparentZstStruct:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Zero>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(Empty:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Zero>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(RustStruct:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(NonRobustRustRepr:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCStruct:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCProjectionCTypeFields:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(InteriorWrapper:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::Gt<rust_spec::One>,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(MixedMutability:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Wrapper<u8>:
        RustSpec<
            Layout = Unstable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Wrapper<UnsafeCell<u8>>:
        RustSpec<
            Layout = Unstable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Interior,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Pair<u8, StdNonZero<u8>>:
        RustSpec<
            Layout = Stable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Pair<bool, u8>:
        RustSpec<
            Layout = Stable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(RawPointer<NotRustSpec>:
        RustSpec<
            Layout = Stable,
            Size = size::Sized<Gt<Zero>>,
            Alignment = <usize as RustSpec>::Alignment,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
}

#[test]
fn struct_wide_classification() {
    assert_impl_all!(Bytes:
        RustSpec<
            Layout = Stable,
            Size = MetaSized<SliceLike>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Packet:
        RustSpec<
            Layout = Unstable,
            Size = MetaSized<SliceLike>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TuplePacket:
        RustSpec<
            Layout = Unstable,
            Size = MetaSized<SliceLike>,
            Alignment = rust_spec::One,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
}
