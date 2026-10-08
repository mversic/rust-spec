#![allow(dead_code)]

use rust_spec::RustSpec;

#[derive(RustSpec)]
#[rust_spec(with_custom_niche)]
enum FieldlessEnum {
    A,
    B,
}

#[derive(RustSpec)]
#[rust_spec(with_custom_niche)]
enum DataEnum {
    A(u8),
    B,
}

#[derive(RustSpec)]
#[repr(transparent)]
#[rust_spec(with_custom_niche)]
enum TransparentEnum {
    A(u8),
}

#[derive(RustSpec)]
#[repr(C)]
#[rust_spec(with_custom_niche)]
enum ReprCEnum {
    A(u8),
}

#[derive(RustSpec)]
#[rust_spec(with_custom_niche)]
union PlainUnion {
    value: u8,
}

fn main() {}
