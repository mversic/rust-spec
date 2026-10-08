#![allow(dead_code)]

use rust_spec::RustSpec;

#[derive(RustSpec)]
#[rust_spec(with_custom_niche, with_custom_niche)]
struct Duplicate(u8);

fn main() {}
