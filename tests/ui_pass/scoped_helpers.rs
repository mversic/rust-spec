#![deny(private_bounds, private_interfaces)]

use rust_spec::{RustSpec, Gt, Zero, size::Sized as SpecSized};

// User items with these names must not collide with generated helpers.
pub struct CombineDrop;
pub struct AddSizedField;

#[derive(RustSpec)]
pub struct First<T>(pub (T, u32));

#[derive(RustSpec)]
pub struct Second<T>(pub [T; 2]);

fn nonzero<T: RustSpec<Size = SpecSized<Gt<Zero>>>>() {}

fn main() {
    nonzero::<First<()>>();
    nonzero::<Second<u8>>();
}
