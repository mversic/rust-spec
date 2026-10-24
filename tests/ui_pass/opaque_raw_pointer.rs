use rust_spec::{Opaque, RustSpec};

#[derive(RustSpec)]
#[rust_spec(opaque)]
struct Handle {
    pointer: *mut u8,
}

fn require_opaque_spec<T: RustSpec<Opacity = Opaque>>() {}

fn main() {
    require_opaque_spec::<Handle>();
}
