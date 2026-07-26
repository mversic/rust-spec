use rust_spec::{
    RustSpec,
    layout::{NonRobust, Robust, Stable, Unstable},
    mutability::Exclusive,
    niche::{Unstable as UnstableNiche, WithNiche},
    size::{NonZst, Sized as SpecSized},
};
use static_assertions::assert_impl_all;

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

#[test]
fn enum_classification() {
    assert_impl_all!(RustEnum:
        RustSpec<
            Layout = Unstable<Robust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
        >,
    );
    assert_impl_all!(PrimitiveEnum:
        RustSpec<
            Layout = Stable<NonRobust>,
            __IndirectLayout = Stable<Robust>,
            Size = SpecSized<NonZst>,
            Niche = WithNiche<UnstableNiche>,
            Mutability = Exclusive,
        >,
    );
}
