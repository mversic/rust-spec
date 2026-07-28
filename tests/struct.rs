use core::{cell::UnsafeCell, num::NonZero, num::NonZeroU8};

use rust_spec::{
    RustSpec,
    layout::{NonRobust, Robust, Stable, Unstable},
    mutability::{Exclusive, Interior},
    niche::{self, Stable as StableNiche, Unstable as UnstableNiche, WithNiche, WithoutNiche},
    size::{self, MetaSized, NonZst, Sized as SpecSized, SliceLike, Zst},
};
use static_assertions::assert_impl_all;

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

#[repr(C)]
#[derive(RustSpec)]
struct Pair<T, U> {
    first: T,
    second: U,
}

struct NotRustSpec;

#[repr(C)]
#[derive(RustSpec)]
struct RawPointer<T> {
    pointer: *mut T,
}

#[repr(transparent)]
#[derive(RustSpec)]
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
    assert_impl_all!(Empty:
        RustSpec<
            Layout = Unstable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<Zst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(RustStruct:
        RustSpec<
            Layout = Unstable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(NonRobustRustRepr:
        RustSpec<
            Layout = Unstable<NonRobust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(ReprCStruct:
        RustSpec<
            Layout = Stable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(InteriorWrapper:
        RustSpec<
            Layout = Unstable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Interior,
        >,
    );
    assert_impl_all!(MixedMutability:
        RustSpec<
            Layout = Unstable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(Wrapper<u8>:
        RustSpec<
            Layout = Unstable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = size::Sized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(Wrapper<UnsafeCell<u8>>:
        RustSpec<
            Layout = Unstable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = size::Sized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Interior,
        >,
    );
    assert_impl_all!(Pair<u8, NonZero<u8>>:
        RustSpec<
            Layout = Stable<NonRobust>,
            __IndirectLayout = Stable<Robust>,
            Size = size::Sized<NonZst>,
            Niche = WithNiche<StableNiche>,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(Pair<bool, u8>:
        RustSpec<
            Layout = Stable<NonRobust>,
            __IndirectLayout = Stable<Robust>,
            Size = size::Sized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(RawPointer<NotRustSpec>:
        RustSpec<
            Layout = Stable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = size::Sized<NonZst>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
}

#[test]
fn struct_wide_classification() {
    assert_impl_all!(Bytes:
        RustSpec<
            Layout = Stable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = MetaSized<SliceLike>,
            Niche = WithoutNiche,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(Packet:
        RustSpec<
            Layout = Unstable<NonRobust>,
            __IndirectLayout = Stable<Robust>,
            Size = MetaSized<SliceLike>,
            Niche = WithNiche<niche::Unstable>,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(TuplePacket:
        RustSpec<
            Layout = Unstable<NonRobust>,
            __IndirectLayout = Stable<Robust>,
            Size = MetaSized<SliceLike>,
            Niche = WithNiche<niche::Unstable>,
            Mutability = Exclusive,
        >,
    );
}
