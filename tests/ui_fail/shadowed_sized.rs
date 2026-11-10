use rust_spec::RustSpec;

trait Sized {}

#[derive(RustSpec)]
struct Bare<T: ?::core::marker::Sized + Sized>(T);

mod core {
    pub mod marker {
        pub trait Sized {}
    }
}

#[derive(RustSpec)]
struct Relative<T: ?::core::marker::Sized>(T)
where T: core::marker::Sized;

fn main() {}
