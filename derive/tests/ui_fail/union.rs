use rust_spec_derive::RustSpec;

#[derive(RustSpec)]
union Unsupported {
    value: u8,
}

fn main() {}
