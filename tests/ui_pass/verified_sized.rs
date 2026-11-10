use rust_spec::RustSpec;

#[derive(RustSpec)]
struct Builtin<T: ?Sized>(T) where T: Sized;

#[derive(RustSpec)]
struct Qualified<T: ?Sized>(T) where T: core::marker::Sized;

mod stronger {
    use rust_spec::RustSpec;
    pub trait Sized: ::core::marker::Sized {}
    impl Sized for u8 {}

    #[derive(RustSpec)]
    pub struct Wrapper<T: ?::core::marker::Sized + Sized>(pub T);
}

#[derive(RustSpec)]
struct HigherRanked<T>(T) where for<'a> &'a T: Sized;

#[derive(RustSpec)]
struct WithDefault<T = u8>(T) where T: Sized;

fn require<T: RustSpec>() {}
fn main() {
    require::<Builtin<u8>>();
    require::<HigherRanked<u8>>();
    require::<WithDefault>();
    require::<Qualified<u8>>();
    require::<stronger::Wrapper<u8>>();
}
