#![allow(dead_code)]

use core::num::NonZeroU8;
use rust_spec::{RustSpec, Unstable, niche::WithNiche};
use static_assertions::assert_impl_all;

#[derive(RustSpec)]
#[rust_spec(with_custom_niche)]
struct Struct(NonZeroU8);

#[derive(RustSpec)]
#[repr(transparent)]
#[rust_spec(with_custom_niche)]
struct TransparentStruct(NonZeroU8);

fn main() {
    assert_impl_all!(Struct: RustSpec<Niche = WithNiche<Unstable>>);
    assert_impl_all!(TransparentStruct: RustSpec<Niche = WithNiche<Unstable>>);
}
