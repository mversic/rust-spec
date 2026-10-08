#![allow(dead_code)]

use rust_spec::RustSpec;

#[derive(RustSpec)]
#[rust_spec(custom_niche, custom_niche)]
struct Duplicate(u8);

fn main() {}
