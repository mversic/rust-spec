use rust_spec::{Opaque, RustSpec};

#[derive(RustSpec)]
#[rust_spec(opaque)]
struct Handle {
    pointer: *mut u8,
}

fn require_opaque_spec<T: Opaque + RustSpec>() {}

fn main() {
    require_opaque_spec::<Handle>();
}
