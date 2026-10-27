use rust_spec::RustSpec;

#[derive(RustSpec)]
#[repr(C)]
struct Node {
    next: *mut Node,
}

fn main() {}
