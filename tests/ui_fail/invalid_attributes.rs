#![allow(dead_code)]

use rust_spec::RustSpec;

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
struct NoDrop(u8);

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
struct FieldNeedsDrop(String);

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
struct Generic<T: RustSpec>(T);

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
enum FieldlessEnum {
    Value,
}

#[derive(RustSpec)]
#[rust_spec(custom_drop)]
union ValueUnion {
    value: u8,
}

#[derive(RustSpec)]
#[rust_spec(opaque, custom_drop)]
struct Opaque;

#[derive(RustSpec)]
#[rust_spec(custom_niche)]
enum NicheFieldlessEnum {
    A,
    B,
}

#[derive(RustSpec)]
#[rust_spec(custom_niche)]
enum DataEnum {
    A(u8),
    B,
}

#[derive(RustSpec)]
#[repr(transparent)]
#[rust_spec(custom_niche)]
enum TransparentEnum {
    A(u8),
}

#[derive(RustSpec)]
#[repr(C)]
#[rust_spec(custom_niche)]
enum ReprCEnum {
    A(u8),
}

#[derive(RustSpec)]
#[rust_spec(custom_niche)]
union PlainUnion {
    value: u8,
}

#[derive(RustSpec)]
#[rust_spec(opaque, custom_niche)]
struct OpaqueAndCustomNiche(u8);

fn main() {}
