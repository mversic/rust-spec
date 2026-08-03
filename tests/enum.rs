use core::{cell::UnsafeCell, num::NonZeroU8};

use rust_spec::{
    RustSpec, Stable, Unstable,
    layout::{NonRobust, Robust},
    mutability::{Exclusive, Interior},
    niche::{WithNiche, WithoutNiche},
    size::{Gt, Sized as SpecSized, Zero},
};
use static_assertions::assert_impl_all;

pub trait ProjectionWithGat {
    type Borrowed<'a>;
}

pub trait ProjectionWithoutGat {
    type Borrowed;
}

impl ProjectionWithGat for u32 {
    type Borrowed<'a> = Self;
}

#[derive(RustSpec)]
pub enum RustEnum {
    A(u8),
    B,
}

#[derive(RustSpec)]
#[repr(u8)]
pub enum PrimitiveEnum {
    A,
    B,
}

#[derive(RustSpec)]
#[repr(u8)]
pub enum PrimitiveDataEnumWithUnstableField {
    A(String),
    B,
}

#[derive(RustSpec)]
#[repr(C)]
pub enum ReprCFieldlessEnum {
    A,
    B,
}

#[derive(RustSpec)]
#[repr(C)]
pub enum ReprCDataEnum {
    A(u8),
    B,
}

#[derive(RustSpec)]
#[repr(C)]
pub enum ReprCSingleInteriorEnum {
    Value(UnsafeCell<u8>),
}

#[derive(RustSpec)]
#[repr(transparent)]
pub enum TransparentInteriorEnum {
    Value(UnsafeCell<u8>),
}

#[derive(RustSpec)]
#[repr(C, u8)]
pub enum ReprCPrimitiveDataEnum {
    A(u8),
    B,
}

#[derive(RustSpec)]
#[repr(C)]
pub enum ReprCDataEnumWithUnstableField {
    A(String),
    B,
}

#[derive(RustSpec)]
enum EmptyEnum {}

#[derive(RustSpec)]
pub enum SingletonEnum {
    Only,
}

#[derive(RustSpec)]
pub enum SingletonWithoutNicheEnum {
    Value(u8),
}

#[derive(RustSpec)]
pub enum SingletonWithNicheEnum {
    Value(NonZeroU8),
}

#[derive(RustSpec)]
pub enum SingletonInteriorEnum {
    Value(UnsafeCell<u8>),
}

#[derive(RustSpec)]
pub enum TwoVariantEnum {
    First,
    Second,
}

#[derive(RustSpec)]
#[repr(u8)]
pub enum TaggedSingletonEnum {
    Only,
}

#[derive(RustSpec)]
#[repr(transparent)]
pub enum TransparentNoNicheEnum {
    Value(u8),
}

#[derive(RustSpec)]
#[repr(transparent)]
pub enum TransparentWithNicheEnum<'a> {
    Value(&'a u8),
}

#[derive(RustSpec)]
#[repr(transparent)]
pub enum TransparentWithMultipleFieldsEnum {
    Value((), NonZeroU8),
}

#[derive(RustSpec)]
#[repr(transparent)]
pub enum TransparentZstEnum {
    Value,
}

#[derive(RustSpec)]
#[repr(C, u8)]
pub enum ReprCDataEnumView<'_dšč, 'a, T>
where
    Self: '_dšč,
    for<'_dummy> &'a [u32; 2]: ProjectionWithGat,
    for<'_dummy> u32: ProjectionWithGat,
    T: ProjectionWithGat,
{
    A(<&'a [u32; 2] as ProjectionWithGat>::Borrowed<'_dšč>),
    B(<u32 as ProjectionWithGat>::Borrowed<'_dšč>),
    C(<T as ProjectionWithGat>::Borrowed<'_dšč>),
    D,
}

#[derive(RustSpec)]
#[repr(C, u8)]
pub enum ReprCDataEnumViewWithoutGat<'a, T>
where
    for<'_dummy> &'a [u32; 2]: ProjectionWithoutGat,
    for<'_dummy> u32: ProjectionWithoutGat,
    T: ProjectionWithoutGat,
{
    A(<&'a [u32; 2] as ProjectionWithoutGat>::Borrowed),
    B(<u32 as ProjectionWithoutGat>::Borrowed),
    C(<T as ProjectionWithoutGat>::Borrowed),
    D,
}

#[derive(RustSpec)]
#[repr(C, u8)]
pub enum ReprCDataEnumView2<'_dšč, 'a>
where
    Self: '_dšč,
    for<'_dummy> &'a [u32; 2]: ProjectionWithGat,
    for<'_dummy> u32: ProjectionWithGat,
{
    A(<&'a [u32; 2] as ProjectionWithGat>::Borrowed<'_dšč>),
    B(<u32 as ProjectionWithGat>::Borrowed<'_dšč>),
    D,
}

#[derive(RustSpec)]
#[repr(C, u8)]
pub enum ReprCDataEnumView2WithoutGat<'a>
where
    for<'_dummy> &'a [u32; 2]: ProjectionWithoutGat,
    for<'_dummy> u32: ProjectionWithoutGat,
{
    A(<&'a [u32; 2] as ProjectionWithoutGat>::Borrowed),
    B(<u32 as ProjectionWithoutGat>::Borrowed),
    D,
}

#[derive(RustSpec)]
#[repr(C, u8)]
pub enum ReprCDataEnumView3<'_dšč, 'a>
where
    Self: '_dšč,
    for<'_dummy> &'a [u32; 2]: ProjectionWithGat,
    for<'_dummy> i32: ProjectionWithGat,
{
    A(<&'a [u32; 2] as ProjectionWithGat>::Borrowed<'_dšč>),
    B(<i32 as ProjectionWithGat>::Borrowed<'_dšč>),
    D,
}

#[derive(RustSpec)]
#[repr(C, u8)]
pub enum ReprCDataEnumView3WithoutGat<'a>
where
    for<'_dummy> &'a [u32; 2]: ProjectionWithoutGat,
    for<'_dummy> i32: ProjectionWithoutGat,
{
    A(<&'a [u32; 2] as ProjectionWithoutGat>::Borrowed),
    B(<i32 as ProjectionWithoutGat>::Borrowed),
    D,
}

#[derive(RustSpec)]
pub enum ReprCDataEnum2<'a, T> {
    A(&'a [u32; 2]),
    B(&'a u32),
    C(T),
    D,
}

#[test]
fn enum_classification() {
    assert_impl_all!(EmptyEnum:
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
    assert_impl_all!(SingletonEnum:
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
    assert_impl_all!(SingletonWithoutNicheEnum:
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
    assert_impl_all!(SingletonWithNicheEnum:
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
    assert_impl_all!(SingletonInteriorEnum:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::One,
            Trap = Robust,
            Niche = WithoutNiche,
            Mutability = Interior,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TwoVariantEnum:
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
    assert_impl_all!(TaggedSingletonEnum:
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
    assert_impl_all!(TransparentNoNicheEnum:
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
    assert_impl_all!(TransparentWithNicheEnum:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = <usize as RustSpec>::Alignment,
            Trap = NonRobust,
            Niche = WithNiche<Stable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(TransparentWithMultipleFieldsEnum:
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
    assert_impl_all!(TransparentZstEnum:
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

    assert_impl_all!(RustEnum:
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
    assert_impl_all!(PrimitiveEnum:
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
    assert_impl_all!(PrimitiveDataEnumWithUnstableField:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = <usize as RustSpec>::Alignment,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCFieldlessEnum:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::Gt<rust_spec::One>,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCDataEnum:
        RustSpec<
            Layout = Stable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = rust_spec::Gt<rust_spec::One>,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(ReprCSingleInteriorEnum:
        RustSpec<Mutability = Exclusive>,
    );
    assert_impl_all!(TransparentInteriorEnum:
        RustSpec<Mutability = Interior>,
    );
    assert_impl_all!(ReprCPrimitiveDataEnum:
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
    assert_impl_all!(ReprCDataEnumWithUnstableField:
        RustSpec<
            Layout = Unstable,
            Size = SpecSized<Gt<Zero>>,
            Alignment = <usize as RustSpec>::Alignment,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
}
