use rust_spec::RustSpec;

#[derive(RustSpec)]
struct Direct(*mut u8);

#[derive(RustSpec)]
struct Nested([(*const u8, u8); 2]);

#[derive(RustSpec)]
enum Variant {
    Pointer(*mut u8),
}

#[derive(RustSpec)]
union Storage {
    pointer: *const u8,
}

fn main() {}
