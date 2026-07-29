use core::{cell::UnsafeCell, num::NonZero, num::NonZeroU8};
use std::ffi::c_void;

use rust_spec::{
    RustSpec, Stable, Unstable,
    layout::{NonRobust, Robust},
    mutability::{Exclusive, Interior},
    niche::{WithNiche, WithoutNiche},
    size::{self, MetaSized, NonZst, Sized as SpecSized, SliceLike, Zst},
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

impl Projection for &u8 {
    type Kita<'a>
        = Self
    where
        Self: 'a;
}

#[derive(RustSpec)]
pub struct ParamReprCZst<T: ?Sized> {
    pub a: (),
    pub b: T,
}

#[derive(RustSpec)]
pub struct WithGat<'a, 'b>(pub <&'a u8 as Projection>::Kita<'b>)
where
    for<'d> &'a u8: Projection,
    Self: 'b;

#[derive(RustSpec)]
pub struct WithGat2<'b>(pub <c_void as Projection>::Kita<'b>)
where
    for<'d> c_void: Projection,
    Self: 'b;

#[derive(RustSpec)]
#[repr(transparent)]
pub struct Custom2View<'_dšč, 'a>(pub <&'a i8 as Projection>::Kita<'_dšč>)
where
    Self: '_dšč,
    for<'_dummy> &'a i8: Projection;

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

#[test]
fn struct_classification() {
    assert_impl_all!(ParamReprCZst<()>:
        RustSpec<
            Layout = Unstable,
            Trap = Robust,
            Size = SpecSized<Zst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ParamReprCZst<UnsafeCell<NonZeroU8>>:
        RustSpec<
            Layout = Unstable,
            Trap = NonRobust,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            // FIXME: Should this be Interior?
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ParamReprCZst<u8>:
        RustSpec<
            Layout = Unstable,
            Trap = Robust,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(SingletonWithNiche:
        RustSpec<
            Layout = Unstable,
            Trap = NonRobust,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(TransparentWithNiche:
        RustSpec<
            Layout = Stable,
            Trap = NonRobust,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<Stable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TransparentWithMultipleFields:
        RustSpec<
            Layout = Stable,
            Trap = NonRobust,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TransparentZstStruct:
        RustSpec<
            Layout = Stable,
            Trap = Robust,
            Size = SpecSized<Zst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(WithGat<'static, 'static>:
        RustSpec<
            Layout = Unstable,
            Trap = NonRobust,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );

    assert_impl_all!(Empty:
        RustSpec<
            Layout = Unstable,
            Trap = Robust,
            Size = SpecSized<Zst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(RustStruct:
        RustSpec<
            Layout = Unstable,
            Trap = Robust,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(NonRobustRustRepr:
        RustSpec<
            Layout = Unstable,
            Trap = NonRobust,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCStruct:
        RustSpec<
            Layout = Stable,
            Trap = Robust,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(InteriorWrapper:
        RustSpec<
            Layout = Unstable,
            Trap = Robust,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Interior,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(MixedMutability:
        RustSpec<
            Layout = Unstable,
            Trap = Robust,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Wrapper<u8>:
        RustSpec<
            Layout = Unstable,
            Trap = Robust,
            Size = size::Sized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Wrapper<UnsafeCell<u8>>:
        RustSpec<
            Layout = Unstable,
            Trap = Robust,
            Size = size::Sized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Interior,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Pair<u8, NonZero<u8>>:
        RustSpec<
            Layout = Stable,
            Trap = NonRobust,
            Size = size::Sized<NonZst>,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Pair<bool, u8>:
        RustSpec<
            Layout = Stable,
            Trap = NonRobust,
            Size = size::Sized<NonZst>,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(RawPointer<NotRustSpec>:
        RustSpec<
            Layout = Stable,
            Trap = Robust,
            Size = size::Sized<NonZst>,
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
            Trap = Robust,
            Size = MetaSized<SliceLike>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Packet:
        RustSpec<
            Layout = Unstable,
            Trap = NonRobust,
            Size = MetaSized<SliceLike>,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TuplePacket:
        RustSpec<
            Layout = Unstable,
            Trap = NonRobust,
            Size = MetaSized<SliceLike>,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
}
