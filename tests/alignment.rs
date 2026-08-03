use std::num::NonZeroU8;

use rust_spec::{
    One, RustSpec, Unstable, Zero,
    layout::{NonRobust, Robust},
    mutability::Exclusive,
    niche::WithNiche,
    size::{Gt, Sized as Co3Sized},
};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
pub struct Aggregate {
    pub byte: u8,
    pub wide: u64,
}

#[derive(RustSpec)]
#[repr(C, align(2048))]
struct ExplicitlyAligned1(u8);

#[derive(RustSpec)]
#[repr(C, align(1))]
struct ExplicitlyAligned2(u16);

#[derive(RustSpec)]
#[repr(u16)]
pub enum Tagged {
    Empty,
    Data(u64),
}

#[derive(RustSpec)]
#[repr(align(2))]
struct Align2Zst;

#[test]
fn primitive_alignment_is_tracked() {
    assert_impl_all!(u8: RustSpec<Alignment = rust_spec::One>);
    assert_impl_all!(u64: RustSpec<Alignment = rust_spec::Gt<rust_spec::One>>);
}

#[test]
fn derives_compute_aggregate_and_tag_alignment() {
    assert_impl_all!(Aggregate: RustSpec<Alignment = rust_spec::Gt<rust_spec::One>>);
    assert_impl_all!(Tagged: RustSpec<Alignment = rust_spec::Gt<rust_spec::One>>);
}

#[test]
fn derives_apply_repr_alignment_modifiers() {
    assert_impl_all!(ExplicitlyAligned1:
        RustSpec<Alignment = rust_spec::Gt<rust_spec::One>>
    );

    assert_impl_all!(ExplicitlyAligned2:
        RustSpec<Alignment = rust_spec::Gt<rust_spec::One>>
    );
}

#[test]
fn result_niches() {
    assert_impl_all!(Result<NonZeroU8, Align2Zst>:
        RustSpec<
            Layout = Unstable,
            Size = Co3Sized<Gt<Zero>>,
            Alignment = Gt<One>,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Result<Align2Zst, NonZeroU8>:
        RustSpec<
            Layout = Unstable,
            Size = Co3Sized<Gt<Zero>>,
            Alignment = Gt<One>,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Result<bool, Align2Zst>:
        RustSpec<
            Layout = Unstable,
            Size = Co3Sized<Gt<Zero>>,
            Alignment = Gt<One>,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
    assert_impl_all!(Result<Align2Zst, bool>:
        RustSpec<
            Layout = Unstable,
            Size = Co3Sized<Gt<Zero>>,
            Alignment = Gt<One>,
            Trap = NonRobust,
            Niche = WithNiche<Unstable>,
            Mutability = Exclusive,
            __IndirectTrap = Robust,
        >,
    );
}
