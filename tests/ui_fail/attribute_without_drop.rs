#![allow(dead_code)]

use rust_spec::RustSpec;

#[derive(RustSpec)]
#[rust_spec(with_custom_drop)]
struct NoDrop(u8);

#[derive(RustSpec)]
#[rust_spec(with_custom_drop)]
struct FieldNeedsDrop(String);

#[derive(RustSpec)]
#[rust_spec(with_custom_drop)]
struct Generic<T: RustSpec>(T);

#[derive(RustSpec)]
#[rust_spec(with_custom_drop)]
enum FieldlessEnum {
    Value,
}

#[derive(RustSpec)]
#[rust_spec(with_custom_drop)]
union ValueUnion {
    value: u8,
}

fn main() {}
