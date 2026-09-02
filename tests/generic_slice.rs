use core::ops::Add;

use rust_spec::{RustSpec, niche::WithoutNiche};

#[repr(transparent)]
#[derive(RustSpec)]
pub struct Transparent<T>([T]);

pub fn assert_rust_spec<T: RustSpec + ?Sized>() {}

pub fn a_slice_of_any_rust_spec_type_is_rust_spec<T: RustSpec>() {
    assert_rust_spec::<[T]>();
}

pub fn a_transparent_slice_wrapper_of_any_rust_spec_type_is_rust_spec<T: RustSpec>()
where
    WithoutNiche: Add<T::Niche>,
{
    assert_rust_spec::<Transparent<T>>();
}

fn main() {}
