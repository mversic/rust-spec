#![allow(type_alias_bounds)]
use rust_spec::RustSpec;

trait Mapping { type Output; }
impl Mapping for u32 { type Output = std::boxed::Box<u32>; }
type Box<T: Mapping> = <T as Mapping>::Output;

#[derive(RustSpec)]
struct Node(Box<Self>);
impl Mapping for Node { type Output = *mut Self; }

mod shadowed_pin {
    use rust_spec::RustSpec;
    struct Pin<T>(T);
    #[derive(RustSpec)]
    struct Node(Pin<Box<Self>>);
}

mod shadowed_deque {
    use rust_spec::RustSpec;
    struct VecDeque<T>(*mut T);
    #[derive(RustSpec)]
    struct Node(VecDeque<Self>);
}

fn main() {}
